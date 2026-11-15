use std::io::{self, BufRead, Write};
use std::sync::mpsc;
use std::time::Duration;
use tao::event_loop::EventLoopProxy;

use super::protocol::{AgentCommand, McpError, McpRequest, McpResponse};
use crate::types::UserEvent;

pub struct McpServer;

impl McpServer {
    /// Runs the stdio Model Context Protocol (MCP) server loop.
    pub fn run_stdio(proxy: EventLoopProxy<UserEvent>) {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let reader = stdin.lock();

        for line in reader.lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let request: McpRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = McpResponse {
                        jsonrpc: "2.0".into(),
                        id: None,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    Self::write_response(&mut stdout, &err_resp);
                    continue;
                }
            };

            // Notifications have no id and require no reply
            if request.id.is_none() {
                continue;
            }

            let response = Self::handle_request(&request, &proxy);
            Self::write_response(&mut stdout, &response);
        }
    }

    fn write_response(stdout: &mut io::Stdout, response: &McpResponse) {
        if let Ok(json) = serde_json::to_string(response) {
            let _ = writeln!(stdout, "{}", json);
            let _ = stdout.flush();
        }
    }

    fn handle_request(req: &McpRequest, proxy: &EventLoopProxy<UserEvent>) -> McpResponse {
        match req.method.as_str() {
            "initialize" => McpResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "aerolith-agent",
                        "version": "0.2.0"
                    }
                })),
                error: None,
            },

            "tools/list" => McpResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: Some(serde_json::json!({
                    "tools": Self::get_tool_definitions()
                })),
                error: None,
            },

            "tools/call" => {
                let params = req.params.as_ref().and_then(|p| p.as_object());
                let name = params.and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                let args = params.and_then(|p| p.get("arguments")).cloned().unwrap_or(serde_json::json!({}));

                Self::execute_tool(req.id.clone(), name, args, proxy)
            }

            _ => McpResponse {
                jsonrpc: "2.0".into(),
                id: req.id.clone(),
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", req.method),
                    data: None,
                }),
            },
        }
    }

    fn get_tool_definitions() -> Vec<serde_json::Value> {
        vec![
            serde_json::json!({
                "name": "browser_navigate",
                "description": "Navigate the browser's active tab to a URL",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string", "description": "The URL to navigate to" }
                    },
                    "required": ["url"]
                }
            }),
            serde_json::json!({
                "name": "browser_snapshot",
                "description": "Capture a compact, token-efficient semantic accessibility tree with numbered target IDs (@1, @2, ...) and coordinates",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_click",
                "description": "Click an element by target ID number (e.g. '1', '2') or CSS selector",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '1') or CSS selector" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_type",
                "description": "Type text into an input field by target ID number or CSS selector",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '2') or CSS selector" },
                        "text": { "type": "string", "description": "Text to type into the field" },
                        "clear": { "type": "boolean", "description": "Whether to clear existing field contents before typing" },
                        "submit": { "type": "boolean", "description": "Whether to press Enter/submit the form after typing" }
                    },
                    "required": ["target", "text"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_markdown",
                "description": "Extract clean, readable article markdown of the active web page, stripping navbars, ads, and cookie banners",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_screenshot",
                "description": "Capture a visual screenshot. Set 'som: true' to overlay Set-Of-Marks numbered bounding boxes for vision models",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "som": { "type": "boolean", "description": "Whether to render Set-Of-Marks visual badges over interactive elements" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_scroll",
                "description": "Scroll the active page up, down, top, or bottom",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "direction": { "type": "string", "enum": ["up", "down", "top", "bottom", "left", "right"], "description": "Scroll direction" },
                        "amount": { "type": "integer", "description": "Pixels to scroll (default 500)" }
                    },
                    "required": ["direction"]
                }
            }),
            serde_json::json!({
                "name": "browser_eval",
                "description": "Execute arbitrary JavaScript expression in the active tab context",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "expression": { "type": "string", "description": "JavaScript code to execute" }
                    },
                    "required": ["expression"]
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for",
                "description": "Wait for an element to appear in the DOM and for network requests to settle (network idle)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element ref ID or CSS selector to wait for" },
                        "timeout_ms": { "type": "integer", "description": "Timeout in milliseconds (default 5000)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_manage_tabs",
                "description": "Manage browser tabs: list open tabs, open a new tab, switch tabs, or close a tab",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "new", "switch", "close"], "description": "Tab action" },
                        "url": { "type": "string", "description": "URL for new tab" },
                        "index": { "type": "integer", "description": "Tab index for switch or close" }
                    },
                    "required": ["action"]
                }
            }),
        ]
    }

    fn execute_tool(
        req_id: Option<serde_json::Value>,
        name: &str,
        args: serde_json::Value,
        proxy: &EventLoopProxy<UserEvent>,
    ) -> McpResponse {
        let command = match name {
            "browser_navigate" => {
                let url = args["url"].as_str().unwrap_or("about:home").to_string();
                AgentCommand::Navigate { url }
            }
            "browser_snapshot" => AgentCommand::Snapshot,
            "browser_click" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::Click { target }
            }
            "browser_type" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                let text = args["text"].as_str().unwrap_or("").to_string();
                let clear = args["clear"].as_bool().unwrap_or(false);
                let submit = args["submit"].as_bool().unwrap_or(false);
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::Type { target, text, clear, submit }
            }
            "browser_get_markdown" => AgentCommand::Markdown,
            "browser_screenshot" => {
                let som = args["som"].as_bool().unwrap_or(false);
                AgentCommand::Screenshot { som }
            }
            "browser_scroll" => {
                let direction = args["direction"].as_str().unwrap_or("down").to_string();
                let amount = args["amount"].as_i64().unwrap_or(500) as i32;
                AgentCommand::Scroll { direction, amount }
            }
            "browser_eval" => {
                let expression = args["expression"].as_str().unwrap_or("").to_string();
                AgentCommand::Eval { expression }
            }
            "browser_wait_for" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                let timeout_ms = args["timeout_ms"].as_u64().unwrap_or(5000);
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::WaitFor { target, timeout_ms }
            }
            "browser_manage_tabs" => {
                let action = args["action"].as_str().unwrap_or("list");
                match action {
                    "new" => {
                        let url = args["url"].as_str().unwrap_or("about:home").to_string();
                        AgentCommand::TabNew { url }
                    }
                    "switch" => {
                        let index = args["index"].as_u64().unwrap_or(0) as usize;
                        AgentCommand::TabSwitch { index }
                    }
                    "close" => {
                        let index = args["index"].as_u64().unwrap_or(0) as usize;
                        AgentCommand::TabClose { index }
                    }
                    _ => AgentCommand::TabsList,
                }
            }
            _ => {
                return McpResponse {
                    jsonrpc: "2.0".into(),
                    id: req_id,
                    result: None,
                    error: Some(McpError {
                        code: -32601,
                        message: format!("Unknown tool: {}", name),
                        data: None,
                    }),
                };
            }
        };

        // Dispatch to event loop
        let (tx, rx) = mpsc::channel();
        if proxy.send_event(UserEvent::AgentAction { command, response_tx: tx }).is_err() {
            return Self::tool_error(req_id, "Browser event loop is unavailable");
        }

        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(result) => {
                if !result.success {
                    let err_msg = result.error.unwrap_or_else(|| "Command failed".into());
                    return Self::tool_error(req_id, err_msg);
                }

                // Check if result is screenshot with base64 image data
                if let Some(data) = result.data.as_ref() {
                    if let Some(b64) = data.get("screenshot_base64").and_then(|v| v.as_str()) {
                        return McpResponse {
                            jsonrpc: "2.0".into(),
                            id: req_id,
                            result: Some(serde_json::json!({
                                "content": [
                                    {
                                        "type": "image",
                                        "data": b64,
                                        "mimeType": "image/png"
                                    }
                                ]
                            })),
                            error: None,
                        };
                    }
                }

                // Normal text response
                let text = if let Some(data) = result.data {
                    if let Some(tree) = data.get("tree_text").and_then(|v| v.as_str()) {
                        tree.to_string()
                    } else if let Some(md) = data.get("markdown").and_then(|v| v.as_str()) {
                        md.to_string()
                    } else {
                        serde_json::to_string_pretty(&data).unwrap_or_default()
                    }
                } else {
                    "OK".to_string()
                };

                McpResponse {
                    jsonrpc: "2.0".into(),
                    id: req_id,
                    result: Some(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": text
                            }
                        ]
                    })),
                    error: None,
                }
            }
            Err(_) => Self::tool_error(req_id, "Tool execution timed out"),
        }
    }

    fn tool_error(id: Option<serde_json::Value>, msg: impl Into<String>) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(serde_json::json!({
                "isError": true,
                "content": [
                    {
                        "type": "text",
                        "text": msg.into()
                    }
                ]
            })),
            error: None,
        }
    }
}

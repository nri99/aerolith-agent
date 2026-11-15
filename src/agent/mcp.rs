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
                "name": "browser_back",
                "description": "Navigate active tab back in browser history",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            serde_json::json!({
                "name": "browser_forward",
                "description": "Navigate active tab forward in browser history",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            serde_json::json!({
                "name": "browser_reload",
                "description": "Reload the current page in the active tab",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            serde_json::json!({
                "name": "browser_get_text",
                "description": "Extract text content, input value, or specific HTML attribute of an element by ref ID or CSS selector",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '3') or CSS selector" },
                        "attribute": { "type": "string", "description": "Optional HTML attribute name (e.g. 'href', 'src', 'data-id'). If omitted, returns element text/value" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_snapshot",
                "description": "Capture a compact, token-efficient semantic accessibility tree with numbered target IDs (@1, @2, ...) and coordinates",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "viewport_only": { "type": "boolean", "description": "When true, returns only interactive elements currently visible within the viewport, saving tokens" },
                        "filter_role": { "type": "string", "description": "Filter snapshot elements by role or tag name (e.g. 'button', 'link', 'input')" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_highlight",
                "description": "Visually highlight an element with an animated focus ring to confirm visual grounding before clicking or typing",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '1') or CSS selector" }
                    },
                    "required": ["target"]
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
                "description": "Capture a visual screenshot. Set 'som: true' to overlay Set-Of-Marks numbered bounding boxes for vision models, or 'full_page: true' for entire scrollable page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "som": { "type": "boolean", "description": "Whether to render Set-Of-Marks visual badges over interactive elements" },
                        "full_page": { "type": "boolean", "description": "Whether to capture the entire scrollable height beyond the viewport" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_screenshot_element",
                "description": "Capture an exact cropped visual screenshot of a specific target element by ref ID or CSS selector. Ideal for inspecting captchas, icons, small buttons, or data tables with 10x token efficiency.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '3') or CSS selector" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_fill_form",
                "description": "Atomically fill multiple form fields in one turn to minimize latency and token overhead, with optional immediate form submit",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "fields": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "target": { "type": "string", "description": "Element numeric ref ID or CSS selector" },
                                    "value": { "type": "string", "description": "Value to populate" }
                                },
                                "required": ["target", "value"]
                            },
                            "description": "List of field targets and values to populate"
                        },
                        "submit": { "type": "boolean", "description": "Whether to submit the parent form after filling" }
                    },
                    "required": ["fields"]
                }
            }),
            serde_json::json!({
                "name": "browser_clipboard",
                "description": "Read or write the browser clipboard context",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["read", "write", "set"], "description": "Whether to read or write clipboard" },
                        "text": { "type": "string", "description": "Text to write into clipboard (required if action is 'write' or 'set')" }
                    },
                    "required": ["action"]
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
            serde_json::json!({
                "name": "browser_storage_state",
                "description": "Export or restore browser cookies, localStorage, and sessionStorage for reproducible agent sessions",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["get", "set"], "description": "Whether to get (export) or set (restore) storage state" },
                        "state": { "type": "object", "description": "State object containing cookies, local_storage, and session_storage (required if action is 'set')" }
                    },
                    "required": ["action"]
                }
            }),
            serde_json::json!({
                "name": "browser_cdp_info",
                "description": "Get Chrome DevTools Protocol (CDP) WebSocket endpoint and Playwright/Puppeteer connection code",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_playwright_info",
                "description": "Get complete Playwright connection configurations, Python/Node.js snippets, and extended Aerolith automation APIs",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_find_element",
                "description": "Find interactive elements by text, placeholder, aria-label, role, or value without scanning the full tree",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search text to match against elements" }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "browser_dialog_policy",
                "description": "Configure automated response policy for window.alert, window.confirm, and window.prompt modal dialogs to prevent browser hangs",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "alert": { "type": "string", "enum": ["accept"], "description": "Policy for alert() dialogs" },
                        "confirm": { "type": "string", "enum": ["accept", "dismiss"], "description": "Policy for confirm() dialogs" },
                        "prompt_value": { "type": "string", "description": "Default return string for prompt() dialogs" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_cdp_execute",
                "description": "Execute a raw Chrome DevTools Protocol (CDP) JSON-RPC method (e.g. 'Page.captureScreenshot', 'Network.getCookies')",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "method": { "type": "string", "description": "CDP method name (e.g. 'Page.captureScreenshot', 'Runtime.evaluate')" },
                        "params": { "type": "object", "description": "CDP method parameters" }
                    },
                    "required": ["method"]
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
            "browser_back" => AgentCommand::Back,
            "browser_forward" => AgentCommand::Forward,
            "browser_reload" => AgentCommand::Reload,
            "browser_get_text" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                let attribute = args["attribute"].as_str().map(|s| s.to_string());
                AgentCommand::GetText { target, attribute }
            }
            "browser_snapshot" => {
                let viewport_only = args["viewport_only"].as_bool().unwrap_or(false);
                let filter_role = args["filter_role"].as_str().map(|s| s.to_string());
                AgentCommand::Snapshot { viewport_only, filter_role }
            }
            "browser_highlight" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::Highlight { target }
            }
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
                let full_page = args["full_page"].as_bool().unwrap_or(false);
                AgentCommand::Screenshot { som, full_page }
            }
            "browser_screenshot_element" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::ScreenshotElement { target }
            }
            "browser_fill_form" => {
                let fields_val = args.get("fields").cloned().unwrap_or(serde_json::json!([]));
                let fields: Vec<crate::agent::protocol::FormField> = serde_json::from_value(fields_val).unwrap_or_default();
                let submit = args["submit"].as_bool().unwrap_or(false);
                AgentCommand::FillForm { fields, submit }
            }
            "browser_clipboard" => {
                let action = args["action"].as_str().unwrap_or("read").to_string();
                let text = args["text"].as_str().map(|s| s.to_string());
                AgentCommand::Clipboard { action, text }
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
            "browser_storage_state" => {
                let action = args["action"].as_str().unwrap_or("get").to_string();
                let state = args.get("state").cloned();
                AgentCommand::StorageState { action, state }
            }
            "browser_cdp_info" | "browser_playwright_info" => {
                let cdp_port = std::env::args()
                    .position(|a| a == "--cdp-port")
                    .and_then(|idx| std::env::args().nth(idx + 1))
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(9222);
                let info = serde_json::json!({
                    "status": "cdp_ready",
                    "cdp_port": cdp_port,
                    "http_endpoint": format!("http://127.0.0.1:{}", cdp_port),
                    "version_url": format!("http://127.0.0.1:{}/json/version", cdp_port),
                    "list_url": format!("http://127.0.0.1:{}/json/list", cdp_port),
                    "playwright_python_snippet": format!("browser = p.chromium.connect_over_cdp('http://127.0.0.1:{}')", cdp_port),
                    "playwright_node_snippet": format!("const browser = await chromium.connectOverCDP('http://127.0.0.1:{}');", cdp_port),
                    "puppeteer_snippet": format!("browser = await puppeteer.connect({{ browserURL: 'http://127.0.0.1:{}' }})", cdp_port),
                    "integration_modules": {
                        "python": "integrations/playwright-python/aerolith_playwright.py",
                        "node": "integrations/playwright-node/index.js"
                    }
                });
                return McpResponse {
                    jsonrpc: "2.0".into(),
                    id: req_id,
                    result: Some(serde_json::json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&info).unwrap_or_default()
                            }
                        ]
                    })),
                    error: None,
                };
            }
            "browser_find_element" => {
                let query = args["query"].as_str().unwrap_or("").to_string();
                if query.is_empty() {
                    return Self::tool_error(req_id, "Missing 'query' parameter");
                }
                AgentCommand::Find { query }
            }
            "browser_dialog_policy" => {
                let alert = args["alert"].as_str().map(|s| s.to_string());
                let confirm = args["confirm"].as_str().map(|s| s.to_string());
                let prompt_value = args["prompt_value"].as_str().map(|s| s.to_string());
                AgentCommand::DialogPolicy { alert, confirm, prompt_value }
            }
            "browser_cdp_execute" => {
                let cdp_port = std::env::args()
                    .position(|a| a == "--cdp-port")
                    .and_then(|idx| std::env::args().nth(idx + 1))
                    .and_then(|p| p.parse::<u16>().ok())
                    .unwrap_or(9222);
                let method = args["method"].as_str().unwrap_or("");
                if method.is_empty() {
                    return Self::tool_error(req_id, "Missing 'method' parameter");
                }
                let params = args.get("params").cloned();
                match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, method, params) {
                    Ok(res) => {
                        return McpResponse {
                            jsonrpc: "2.0".into(),
                            id: req_id,
                            result: Some(serde_json::json!({
                                "content": [
                                    {
                                        "type": "text",
                                        "text": serde_json::to_string_pretty(&res).unwrap_or_default()
                                    }
                                ]
                            })),
                            error: None,
                        };
                    }
                    Err(e) => return Self::tool_error(req_id, e),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_tool_definitions() {
        let tools = McpServer::get_tool_definitions();
        let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
        assert!(tool_names.contains(&"browser_navigate"));
        assert!(tool_names.contains(&"browser_back"));
        assert!(tool_names.contains(&"browser_forward"));
        assert!(tool_names.contains(&"browser_reload"));
        assert!(tool_names.contains(&"browser_get_text"));
        assert!(tool_names.contains(&"browser_snapshot"));
        assert!(tool_names.contains(&"browser_highlight"));
        assert!(tool_names.contains(&"browser_screenshot"));
        assert!(tool_names.contains(&"browser_storage_state"));
        assert!(tool_names.contains(&"browser_cdp_info"));
        assert!(tool_names.contains(&"browser_playwright_info"));
        assert!(tool_names.contains(&"browser_cdp_execute"));
        assert!(tool_names.contains(&"browser_screenshot_element"));
        assert!(tool_names.contains(&"browser_fill_form"));
        assert!(tool_names.contains(&"browser_clipboard"));

        let snap = tools.iter().find(|t| t["name"] == "browser_snapshot").unwrap();
        assert!(snap["inputSchema"]["properties"]["filter_role"].is_object());

        let shot = tools.iter().find(|t| t["name"] == "browser_screenshot").unwrap();
        assert!(shot["inputSchema"]["properties"]["full_page"].is_object());
    }

    #[test]
    fn test_mcp_initialize() {
        let req = McpRequest {
            jsonrpc: "2.0".into(),
            id: Some(serde_json::json!(1)),
            method: "initialize".into(),
            params: None,
        };
        #[cfg(target_os = "windows")]
        use tao::platform::windows::EventLoopBuilderExtWindows;
        let mut builder = tao::event_loop::EventLoopBuilder::<UserEvent>::with_user_event();
        #[cfg(target_os = "windows")]
        builder.with_any_thread(true);
        let dummy_loop = builder.build();
        let proxy = dummy_loop.create_proxy();
        let resp = McpServer::handle_request(&req, &proxy);
        assert_eq!(resp.jsonrpc, "2.0");
        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        assert_eq!(res["serverInfo"]["name"], "aerolith-agent");
    }
}

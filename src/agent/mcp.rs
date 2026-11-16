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
                "name": "browser_set_viewport",
                "description": "Dynamically emulate viewport metrics (width, height, mobile user agent & touch emulation)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "width": { "type": "integer", "description": "Viewport width in pixels (e.g. 375 for mobile, 1280 for desktop)" },
                        "height": { "type": "integer", "description": "Viewport height in pixels (e.g. 812 for mobile, 800 for desktop)" },
                        "dpr": { "type": "number", "description": "Device pixel ratio (default 1.0, 2.0 for retina)" },
                        "mobile": { "type": "boolean", "description": "Whether to emulate mobile device screen & touch interactions" }
                    },
                    "required": ["width", "height"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_console_logs",
                "description": "Retrieve captured JavaScript console logs (log, warn, error) and uncaught window errors from the page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear": { "type": "boolean", "description": "Whether to clear the console log buffer after reading (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_print_pdf",
                "description": "Render and export the current page to a high-fidelity vector PDF",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "landscape": { "type": "boolean", "description": "Whether to print in landscape orientation (default: false)" }
                    }
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
            serde_json::json!({
                "name": "browser_hover",
                "description": "Hover over an element to reveal tooltips, flyout submenus, or trigger hover CSS states",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element ref ID (e.g. '3') or CSS selector to hover over" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_press_key",
                "description": "Press a keyboard key (e.g. 'Escape' to dismiss modals, 'Enter', 'Tab', 'ArrowDown', 'Backspace')",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "key": { "type": "string", "description": "Key name (e.g. 'Escape', 'Enter', 'Tab', 'ArrowDown', 'ArrowUp', 'Backspace', 'Space')" },
                        "target": { "type": "string", "description": "Optional element ref ID or CSS selector to focus before pressing key" }
                    },
                    "required": ["key"]
                }
            }),
            serde_json::json!({
                "name": "browser_select_option",
                "description": "Select an option in an HTML <select> dropdown by value, label, or index",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element ref ID or CSS selector for the <select> element" },
                        "value": { "type": "string", "description": "Option value, text content, or zero-based index to select" }
                    },
                    "required": ["target", "value"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_network_activity",
                "description": "Retrieve recent network requests made by the page (fetch & XMLHttpRequest) and active in-flight request count",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear": { "type": "boolean", "description": "Whether to clear the recorded network request buffer after retrieval (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_drag",
                "description": "Drag an element to a target element or coordinate offset, supporting HTML5 drag-and-drop and range sliders",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "Source element ref ID or CSS selector" },
                        "target": { "type": "string", "description": "Optional destination element ref ID or CSS selector" },
                        "offset_x": { "type": "integer", "description": "Horizontal pixel offset" },
                        "offset_y": { "type": "integer", "description": "Vertical pixel offset" },
                        "steps": { "type": "integer", "description": "Number of intermediate mouse movement steps (default: 10)" }
                    },
                    "required": ["source"]
                }
            }),
            serde_json::json!({
                "name": "browser_cookies",
                "description": "Get, set, delete, or clear browser cookies via Chrome DevTools Protocol, including HttpOnly and secure cookies",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["get", "set", "delete", "clear"], "description": "Cookie operation action" },
                        "cookie": { "type": "object", "description": "Cookie object with name, value, domain, path, http_only, secure (required for action='set')" },
                        "name": { "type": "string", "description": "Cookie name (for action='delete')" },
                        "domain": { "type": "string", "description": "Cookie domain (for action='delete')" }
                    },
                    "required": ["action"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_headers",
                "description": "Set custom HTTP request headers applied to all subsequent network requests",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "headers": { "type": "object", "description": "Key-value map of HTTP headers" }
                    },
                    "required": ["headers"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_user_agent",
                "description": "Override the browser User-Agent string, accept-language header, and platform identifier",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "user_agent": { "type": "string", "description": "User-Agent string" },
                        "accept_language": { "type": "string", "description": "Accept-Language header (e.g. 'en-US,en;q=0.9')" },
                        "platform": { "type": "string", "description": "Platform string (e.g. 'Win32', 'Linux x86_64')" }
                    },
                    "required": ["user_agent"]
                }
            }),
            serde_json::json!({
                "name": "browser_upload_file",
                "description": "Upload one or more files to an <input type=\"file\"> element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element ref ID or CSS selector for the file input" },
                        "files": { "type": "array", "items": { "type": "string" }, "description": "File paths or file names to upload" }
                    },
                    "required": ["target", "files"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_location",
                "description": "Emulate geographical coordinates (latitude, longitude, accuracy) and timezone via CDP and geolocation API overrides",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "latitude": { "type": "number", "description": "Latitude in decimal degrees" },
                        "longitude": { "type": "number", "description": "Longitude in decimal degrees" },
                        "accuracy": { "type": "number", "description": "Position accuracy in meters (default: 1.0)" },
                        "timezone": { "type": "string", "description": "Timezone identifier (e.g. 'America/New_York', 'Europe/London')" }
                    },
                    "required": ["latitude", "longitude"]
                }
            }),
            serde_json::json!({
                "name": "browser_media_control",
                "description": "Inspect and control media playback (video and audio) on the page (status, pause, play, mute, unmute)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["status", "pause", "play", "mute", "unmute"], "description": "Media playback action" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_inspect_element",
                "description": "Inspect comprehensive attributes, computed CSS styles (visibility, display, opacity, pointer-events), viewport state, and exact bounding box of an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the element to inspect" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_route_rule",
                "description": "Dynamically intercept, abort, or mock network requests matching specific URL patterns (e.g. abort bloated ad/tracker scripts or mock backend APIs)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["add", "list", "clear", "remove"], "description": "Route rule action (default 'list')" },
                        "pattern": { "type": "string", "description": "URL glob or substring pattern (e.g. '*.png', '*/analytics/*', '*/api/*')" },
                        "rule_type": { "type": "string", "enum": ["abort", "mock"], "description": "Rule action type: 'abort' to cancel or 'mock' to stub" },
                        "status": { "type": "integer", "description": "HTTP status code for mock response (default 200)" },
                        "body": { "type": "string", "description": "Mock response body content" },
                        "headers": { "type": "object", "description": "Optional mock response headers" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_double_click",
                "description": "Simulate a human-like double-click on an interactive element (e.g. table row, file explorer item)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_right_click",
                "description": "Simulate a right-click / context menu event on an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_extract_table",
                "description": "Extract structured data rows and column headers from an HTML <table> into JSON records",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or ref ID of the table (defaults to largest table)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_downloads",
                "description": "Retrieve recorded browser file downloads or clear download history",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear": { "type": "boolean", "description": "Whether to clear download history" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_emulate_network",
                "description": "Throttle or emulate network connectivity (profiles: 'online', 'offline', 'slow-3g', 'fast-3g') via CDP",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "profile": { "type": "string", "enum": ["online", "offline", "slow-3g", "fast-3g"], "description": "Network profile name" },
                        "latency": { "type": "number", "description": "Additional latency in milliseconds" },
                        "download": { "type": "number", "description": "Max download throughput in bytes/second" },
                        "upload": { "type": "number", "description": "Max upload throughput in bytes/second" },
                        "offline": { "type": "boolean", "description": "Force complete offline disconnection" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_inject_css",
                "description": "Inject custom CSS stylesheets or remove previously injected styles to eliminate floating headers, banners, or occluding elements",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "css": { "type": "string", "description": "CSS rules string to inject" },
                        "remove": { "type": "boolean", "description": "Remove the injected custom stylesheet" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_query_elements",
                "description": "Batch query and inspect multiple DOM elements matching a CSS selector in a single turn",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "selector": { "type": "string", "description": "CSS selector to match (e.g. 'a.result', 'li.item')" },
                        "attribute": { "type": "string", "description": "Optional attribute name to extract (e.g. 'href', 'src')" },
                        "max_items": { "type": "integer", "description": "Maximum number of elements to return (default 50)" }
                    },
                    "required": ["selector"]
                }
            }),
            serde_json::json!({
                "name": "browser_focus",
                "description": "Explicitly focus an element to trigger framework validation, active styling, and input readiness",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_blur",
                "description": "Explicitly blur an element to trigger framework change events and input validation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_page_info",
                "description": "Retrieve comprehensive page layout metrics, scroll progress, document dimensions, and window metadata",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_html",
                "description": "Extract raw HTML source for the entire document or a specific DOM element by selector or ref",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID. If omitted, returns outerHTML of the entire document" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_scroll_into_view",
                "description": "Scroll a specific element into view with block alignment and automatic sticky-header occlusion compensation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "align": { "type": "string", "description": "Vertical alignment: 'start', 'center', 'end', or 'nearest' (default: 'center')" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_zoom",
                "description": "Set the browser rendering zoom factor (e.g., 1.2 for 120%, 0.8 for 80%) with dual-layer CSS and CDP scaling",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "zoom_factor": { "type": "number", "description": "Scale factor between 0.25 and 5.0 (default: 1.0)" }
                    },
                    "required": ["zoom_factor"]
                }
            }),
            serde_json::json!({
                "name": "browser_storage_item",
                "description": "Inspect or mutate granular localStorage or sessionStorage keys (get, set, delete, clear, keys)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "storage_type": { "type": "string", "description": "Storage backend: 'local' (default) or 'session'" },
                        "action": { "type": "string", "description": "Action to perform: 'get', 'set', 'delete', 'clear', 'keys'" },
                        "key": { "type": "string", "description": "Storage key name (required for get, set, delete)" },
                        "value": { "type": "string", "description": "Value to store (required for set)" }
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
            "browser_set_viewport" => {
                let width = args["width"].as_u64().unwrap_or(1280) as u32;
                let height = args["height"].as_u64().unwrap_or(800) as u32;
                let dpr = args["dpr"].as_f64();
                let mobile = args["mobile"].as_bool().unwrap_or(false);
                AgentCommand::Emulate { width, height, dpr, mobile }
            }
            "browser_get_console_logs" => {
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::ConsoleLogs { clear }
            }
            "browser_print_pdf" => {
                let landscape = args["landscape"].as_bool().unwrap_or(false);
                AgentCommand::PrintPdf { landscape }
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
            "browser_hover" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                AgentCommand::Hover { target }
            }
            "browser_press_key" => {
                let key = args["key"].as_str().unwrap_or("").to_string();
                if key.is_empty() {
                    return Self::tool_error(req_id, "Missing 'key' parameter");
                }
                let target = args["target"].as_str().map(|s| s.to_string());
                AgentCommand::PressKey { key, target }
            }
            "browser_select_option" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                let value = args["value"].as_str().unwrap_or("").to_string();
                if target.is_empty() || value.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' or 'value' parameter");
                }
                AgentCommand::SelectOption { target, value }
            }
            "browser_get_network_activity" => {
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::NetworkActivity { clear }
            }
            "browser_drag" => {
                let source = args["source"].as_str().unwrap_or("").to_string();
                if source.is_empty() {
                    return Self::tool_error(req_id, "Missing 'source' parameter");
                }
                let target = args["target"].as_str().map(|s| s.to_string());
                let offset_x = args["offset_x"].as_i64().map(|n| n as i32);
                let offset_y = args["offset_y"].as_i64().map(|n| n as i32);
                let steps = args["steps"].as_u64().unwrap_or(10) as u32;
                AgentCommand::Drag { source, target, offset_x, offset_y, steps }
            }
            "browser_cookies" => {
                let action = args["action"].as_str().unwrap_or("get").to_string();
                let cookie = args.get("cookie").cloned();
                let name = args["name"].as_str().map(|s| s.to_string());
                let domain = args["domain"].as_str().map(|s| s.to_string());
                AgentCommand::Cookies { action, cookie, name, domain }
            }
            "browser_set_headers" => {
                let headers = args.get("headers").cloned().unwrap_or(serde_json::json!({}));
                AgentCommand::ExtraHeaders { headers }
            }
            "browser_set_user_agent" => {
                let user_agent = args["user_agent"].as_str().unwrap_or("").to_string();
                if user_agent.is_empty() {
                    return Self::tool_error(req_id, "Missing 'user_agent' parameter");
                }
                let accept_language = args["accept_language"].as_str().map(|s| s.to_string());
                let platform = args["platform"].as_str().map(|s| s.to_string());
                AgentCommand::UserAgentOverride { user_agent, accept_language, platform }
            }
            "browser_upload_file" => {
                let target = args["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                let files: Vec<String> = args.get("files")
                    .and_then(|v| v.as_array())
                    .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|f| f.to_string())).collect())
                    .unwrap_or_default();
                AgentCommand::UploadFile { target, files }
            }
            "browser_set_location" => {
                let latitude = args["latitude"].as_f64().unwrap_or(0.0);
                let longitude = args["longitude"].as_f64().unwrap_or(0.0);
                let accuracy = args.get("accuracy").and_then(|v| v.as_f64());
                let timezone = args["timezone"].as_str().map(|s| s.to_string());
                AgentCommand::Location { latitude, longitude, accuracy, timezone }
            }
            "browser_media_control" => {
                let action = args["action"].as_str().unwrap_or("status").to_string();
                AgentCommand::MediaControl { action }
            }
            "browser_inspect_element" => {
                let target = args["target"]
                    .as_str()
                    .or_else(|| args["ref"].as_str())
                    .unwrap_or("")
                    .to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing required parameter 'target'");
                }
                AgentCommand::Inspect { target }
            }
            "browser_route_rule" => {
                let action = args["action"].as_str().unwrap_or("list").to_string();
                let pattern = args["pattern"].as_str().map(|s| s.to_string());
                let rule_type = args["rule_type"].as_str().or_else(|| args["type"].as_str()).map(|s| s.to_string());
                let status = args["status"].as_u64().map(|n| n as u16);
                let body = args["body"].as_str().map(|s| s.to_string());
                let headers = args.get("headers").cloned();
                AgentCommand::RouteRule { action, pattern, rule_type, status, body, headers }
            }
            "browser_double_click" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::DoubleClick { target }
            }
            "browser_right_click" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::RightClick { target }
            }
            "browser_extract_table" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::ExtractTable { target }
            }
            "browser_get_downloads" => {
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::Downloads { clear }
            }
            "browser_emulate_network" => {
                let profile = args["profile"].as_str().unwrap_or("online").to_string();
                let latency = args["latency"].as_f64().or_else(|| args["latency_ms"].as_f64());
                let download_throughput = args["download"].as_f64().or_else(|| args["download_throughput"].as_f64());
                let upload_throughput = args["upload"].as_f64().or_else(|| args["upload_throughput"].as_f64());
                let offline = args["offline"].as_bool();
                AgentCommand::NetworkConditions { profile, latency, download_throughput, upload_throughput, offline }
            }
            "browser_inject_css" => {
                let css = args["css"].as_str().unwrap_or("").to_string();
                let remove = args["remove"].as_bool().unwrap_or(false);
                AgentCommand::InjectCss { css, remove }
            }
            "browser_query_elements" => {
                let selector = args["selector"].as_str().or_else(|| args["sel"].as_str()).unwrap_or("").to_string();
                if selector.is_empty() { return Self::tool_error(req_id, "Missing 'selector' parameter"); }
                let attribute = args["attribute"].as_str().or_else(|| args["attr"].as_str()).map(|s| s.to_string());
                let max_items = args["max_items"].as_u64().or_else(|| args["max"].as_u64()).unwrap_or(50) as usize;
                AgentCommand::QueryElements { selector, attribute, max_items }
            }
            "browser_focus" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::Focus { target }
            }
            "browser_blur" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::Blur { target }
            }
            "browser_get_page_info" => {
                AgentCommand::PageInfo
            }
            "browser_get_html" => {
                let target = args["target"].as_str().or_else(|| args["selector"].as_str()).map(|s| s.to_string());
                AgentCommand::GetHtml { target }
            }
            "browser_scroll_into_view" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let align = args["align"].as_str().unwrap_or("center").to_string();
                AgentCommand::ScrollIntoView { target, align }
            }
            "browser_set_zoom" => {
                let zoom_factor = args["zoom_factor"].as_f64().or_else(|| args["factor"].as_f64()).unwrap_or(1.0);
                AgentCommand::SetZoom { zoom_factor }
            }
            "browser_storage_item" => {
                let storage_type = args["storage_type"].as_str().or_else(|| args["type"].as_str()).unwrap_or("local").to_string();
                let action = args["action"].as_str().unwrap_or("get").to_string();
                let key = args["key"].as_str().map(|s| s.to_string());
                let value = args["value"].as_str().map(|s| s.to_string());
                AgentCommand::StorageItem { storage_type, action, key, value }
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
        assert!(tool_names.contains(&"browser_set_viewport"));
        assert!(tool_names.contains(&"browser_get_console_logs"));
        assert!(tool_names.contains(&"browser_print_pdf"));
        assert!(tool_names.contains(&"browser_hover"));
        assert!(tool_names.contains(&"browser_press_key"));
        assert!(tool_names.contains(&"browser_select_option"));
        assert!(tool_names.contains(&"browser_get_network_activity"));
        assert!(tool_names.contains(&"browser_drag"));
        assert!(tool_names.contains(&"browser_cookies"));
        assert!(tool_names.contains(&"browser_set_headers"));
        assert!(tool_names.contains(&"browser_set_user_agent"));
        assert!(tool_names.contains(&"browser_upload_file"));
        assert!(tool_names.contains(&"browser_set_location"));
        assert!(tool_names.contains(&"browser_media_control"));
        assert!(tool_names.contains(&"browser_inspect_element"));
        assert!(tool_names.contains(&"browser_route_rule"));
        assert!(tool_names.contains(&"browser_double_click"));
        assert!(tool_names.contains(&"browser_right_click"));
        assert!(tool_names.contains(&"browser_extract_table"));
        assert!(tool_names.contains(&"browser_get_downloads"));
        assert!(tool_names.contains(&"browser_emulate_network"));
        assert!(tool_names.contains(&"browser_inject_css"));
        assert!(tool_names.contains(&"browser_query_elements"));
        assert!(tool_names.contains(&"browser_focus"));
        assert!(tool_names.contains(&"browser_blur"));
        assert!(tool_names.contains(&"browser_get_page_info"));
        assert!(tool_names.contains(&"browser_get_html"));
        assert!(tool_names.contains(&"browser_scroll_into_view"));
        assert!(tool_names.contains(&"browser_set_zoom"));
        assert!(tool_names.contains(&"browser_storage_item"));

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

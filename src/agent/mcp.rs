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
                "description": "Capture an exact cropped visual screenshot of a specific target element by ref ID or CSS selector with optional padding. Ideal for inspecting captchas, icons, small buttons, or data tables with 10x token efficiency.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Element numeric ref ID (e.g. '3') or CSS selector" },
                        "padding": { "type": "integer", "description": "Optional padding in pixels around the element bounding box (default: 0)" }
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
            serde_json::json!({
                "name": "browser_assert_element",
                "description": "Verify element state against declarative assertions (visible, hidden, disabled, checked, text_contains, value, has_class, attr_name/attr_value)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "visible": { "type": "boolean", "description": "Assert element is visible in layout" },
                        "hidden": { "type": "boolean", "description": "Assert element is absent or hidden" },
                        "disabled": { "type": "boolean", "description": "Assert element disabled state" },
                        "checked": { "type": "boolean", "description": "Assert checkbox/radio checked state" },
                        "text_contains": { "type": "string", "description": "Assert element text contains substring" },
                        "value": { "type": "string", "description": "Assert input value equals string" },
                        "has_class": { "type": "string", "description": "Assert element classList contains class" },
                        "attr_name": { "type": "string", "description": "Attribute name to check" },
                        "attr_value": { "type": "string", "description": "Expected attribute value" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for_selector",
                "description": "Wait for a DOM element to reach a target state: 'visible', 'hidden', 'attached', or 'detached' with configurable timeout",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "state": { "type": "string", "description": "Desired state: 'visible' (default), 'hidden', 'attached', or 'detached'" },
                        "timeout_ms": { "type": "integer", "description": "Maximum wait time in milliseconds (default: 5000)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_performance_metrics",
                "description": "Retrieve page performance metrics (TTFB, DOMContentLoaded, load time, resource breakdown, transfer bytes, JS heap memory)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_scroll_to_coords",
                "description": "Scroll the browser viewport directly to specific absolute (x, y) coordinates with instant or smooth behavior",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "integer", "description": "Horizontal scroll pixel offset (default: 0)" },
                        "y": { "type": "integer", "description": "Vertical scroll pixel offset (default: 0)" },
                        "smooth": { "type": "boolean", "description": "Smooth scrolling behavior (default: false)" }
                    },
                    "required": ["x", "y"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_page_outline",
                "description": "Retrieve compact semantic outline of the page: headings hierarchy, form structures, landmarks, interactive buttons, and links",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "max_links": { "type": "integer", "description": "Maximum links to return (default: 25)" },
                        "max_buttons": { "type": "integer", "description": "Maximum buttons to return (default: 25)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for_network_idle",
                "description": "Wait until active network requests remain at or below max_inflight for consecutive idle_time_ms",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "idle_time_ms": { "type": "integer", "description": "Consecutive idle duration required in milliseconds (default: 500)" },
                        "timeout_ms": { "type": "integer", "description": "Maximum timeout in milliseconds (default: 10000)" },
                        "max_inflight": { "type": "integer", "description": "Maximum tolerated inflight requests (default: 0)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_watch_mutations",
                "description": "Control and query DOM mutation records (start, stop, clear, get)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "description": "Action: 'start', 'stop', 'clear', or 'get' (default: 'get')" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container to observe" }
                    },
                    "required": ["action"]
                }
            }),
            serde_json::json!({
                "name": "browser_tab_eval",
                "description": "Evaluate a JavaScript expression directly in a specific tab by index without switching active GUI focus",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tab_index": { "type": "integer", "description": "Zero-based index of the target tab" },
                        "script": { "type": "string", "description": "JavaScript code or expression to evaluate" }
                    },
                    "required": ["tab_index", "script"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_response_body",
                "description": "Search recorded network activity for requests matching a URL pattern and return the captured response body",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "URL substring or path pattern to search (e.g. '/api/users')" },
                        "clear": { "type": "boolean", "description": "Whether to flush recorded network activity after reading (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_drop_file",
                "description": "Dispatch a synthetic file drag-and-drop or file input upload with custom file metadata and content",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target dropzone or file input" },
                        "file_name": { "type": "string", "description": "Name of the file to simulate (e.g. 'data.csv')" },
                        "mime_type": { "type": "string", "description": "MIME type (e.g. 'text/csv', 'image/png')" },
                        "content": { "type": "string", "description": "File payload (raw text or base64 encoded string)" }
                    },
                    "required": ["target", "file_name", "content"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_dom_tree",
                "description": "Serialize a nested hierarchical JSON tree of visible DOM nodes starting from root or target selector",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of root subtree (default: body)" },
                        "max_depth": { "type": "integer", "description": "Maximum tree depth to traverse (default: 4)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_set_offline",
                "description": "Instantly simulate network offline or online state via CDP Network emulation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "offline": { "type": "boolean", "description": "True to disconnect network, false to restore connectivity" }
                    },
                    "required": ["offline"]
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for_text",
                "description": "Wait until specific text substring appears in the document or a specified target element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Text substring to wait for" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of the container element" },
                        "timeout_ms": { "type": "integer", "description": "Maximum wait time in milliseconds (default: 5000)" }
                    },
                    "required": ["text"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_computed_styles",
                "description": "Read computed CSS style properties (e.g. display, visibility, opacity, color, z-index) for an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "properties": { "type": "array", "items": { "type": "string" }, "description": "List of CSS property names to query (defaults to common layout/visibility styles)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for_download",
                "description": "Wait for a new file download initiated by the page to appear, optionally matching a filename pattern",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "file_pattern": { "type": "string", "description": "Optional filename substring or pattern to match" },
                        "timeout_ms": { "type": "integer", "description": "Maximum wait time in milliseconds (default: 15000)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_set_content",
                "description": "Set the HTML content of the active page directly via document.write, replacing the current document",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "html": { "type": "string", "description": "HTML markup string to load into the active page" }
                    },
                    "required": ["html"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_forms",
                "description": "Inspect and structure all HTML forms and their controls (input, select, textarea, button) on the active page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container subtree (defaults to whole document)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_set_visibility",
                "description": "Force-emulate document.visibilityState ('visible' or 'hidden') and trigger visibilitychange event",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "state": { "type": "string", "description": "Desired visibility state: 'visible' (default) or 'hidden'" }
                    },
                    "required": ["state"]
                }
            }),
            serde_json::json!({
                "name": "browser_emulate_media",
                "description": "Emulate media color scheme ('dark', 'light', 'no-preference') and/or media type ('screen', 'print') via CDP",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "color_scheme": { "type": "string", "description": "Color scheme preference: 'dark', 'light', or 'no-preference'" },
                        "media_type": { "type": "string", "description": "CSS media type: 'screen' or 'print'" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_bounds",
                "description": "Extract exact pixel bounding box coordinates (x, y, width, height, center_x, center_y) and visibility metrics for an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_mock_route",
                "description": "Mock network responses for fetch/XHR requests matching a URL pattern (action: set, clear, list)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "description": "Mock action: 'set' (default), 'clear', or 'list'" },
                        "pattern": { "type": "string", "description": "URL substring or pattern to intercept" },
                        "status": { "type": "integer", "description": "HTTP status code to return (default: 200)" },
                        "content_type": { "type": "string", "description": "Response Content-Type header (default: application/json)" },
                        "body": { "type": "string", "description": "Mock response body payload string" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_viewport",
                "description": "Retrieve active viewport dimensions, scroll offsets, screen metrics, and device pixel ratio",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_set_element_attribute",
                "description": "Dynamically set or remove an HTML attribute on a DOM element for self-healing automation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "attribute": { "type": "string", "description": "Attribute name to set or remove (e.g. 'disabled', 'class')" },
                        "value": { "type": "string", "description": "Attribute value (omit or set null to remove attribute)" }
                    },
                    "required": ["target", "attribute"]
                }
            }),
            serde_json::json!({
                "name": "browser_search_page_text",
                "description": "Search all visible text content on the page and return matched snippets, element ref IDs, and bounding boxes",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Text substring to search for" },
                        "max_matches": { "type": "integer", "description": "Maximum number of matches to return (default: 20)" },
                        "case_sensitive": { "type": "boolean", "description": "Whether to perform case-sensitive search (default: false)" }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "browser_dump_storage",
                "description": "Dump all localStorage and/or sessionStorage key-value pairs in a single structured JSON payload",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "storage_type": { "type": "string", "description": "Storage partition to dump: 'all' (default), 'local', or 'session'" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_human_jitter",
                "description": "Simulate natural mouse micro-movements and subtle scrolling over a duration to satisfy anti-bot behavioral heuristics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "duration_ms": { "type": "integer", "description": "Duration of jitter simulation in milliseconds (default: 800)" },
                        "move_mouse": { "type": "boolean", "description": "Whether to generate random mouse movements (default: true)" },
                        "micro_scroll": { "type": "boolean", "description": "Whether to perform subtle scroll adjustments (default: true)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_type_keystrokes",
                "description": "Type text character-by-character into target element with realistic per-key delay and complete KeyboardEvent sequences",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "text": { "type": "string", "description": "Text string to type character-by-character" },
                        "delay_ms": { "type": "integer", "description": "Delay between keystrokes in milliseconds (default: 25)" },
                        "clear": { "type": "boolean", "description": "Whether to clear input before typing (default: false)" },
                        "submit": { "type": "boolean", "description": "Whether to press Enter and submit form after typing (default: false)" }
                    },
                    "required": ["target", "text"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_canvas_data",
                "description": "Extract base64 image data URL from <canvas>, <img>, or arbitrary DOM element via rasterization",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "mime_type": { "type": "string", "description": "Image MIME type ('image/png', 'image/jpeg', 'image/webp') (default: 'image/png')" },
                        "quality": { "type": "number", "description": "Image encoding quality from 0.0 to 1.0 (default: 0.92)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_resource_timing",
                "description": "Extract structured resource timing and transfer bandwidth metrics from window.performance",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "initiator_type": { "type": "string", "description": "Filter by resource initiator: 'script', 'img', 'css', 'fetch', 'xmlhttprequest'" },
                        "max_entries": { "type": "integer", "description": "Maximum number of resource records to return (default: 50)" },
                        "min_duration_ms": { "type": "number", "description": "Minimum duration threshold in milliseconds (default: 0.0)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_dom_stats",
                "description": "Inspect structural DOM complexity metrics (element count, tree depth, forms, buttons, links, inputs, media)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_scroll_by",
                "description": "Perform relative window scroll by horizontal (dx) and vertical (dy) pixel deltas",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "dx": { "type": "integer", "description": "Horizontal pixel scroll delta (default: 0)" },
                        "dy": { "type": "integer", "description": "Vertical pixel scroll delta (default: 0)" },
                        "smooth": { "type": "boolean", "description": "Whether to use smooth scrolling behavior (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_scroll_container",
                "description": "Scroll a specific inner scrollable container (e.g. div, modal, textarea, section) by horizontal (dx) and vertical (dy) pixels",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the container element" },
                        "dx": { "type": "integer", "description": "Horizontal pixel scroll delta (default: 0)" },
                        "dy": { "type": "integer", "description": "Vertical pixel scroll delta (default: 0)" },
                        "smooth": { "type": "boolean", "description": "Whether to use smooth scrolling behavior (default: false)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_select_element_text",
                "description": "Select text range inside an input, textarea, or contentEditable element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "start": { "type": "integer", "description": "Starting character offset (default: 0)" },
                        "length": { "type": "integer", "description": "Number of characters to select (omit to select to end)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_extract_page_links",
                "description": "Extract structured, deduplicated hyperlinks graph from active document with external origin detection",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "max_links": { "type": "integer", "description": "Maximum number of links to return (default: 100)" },
                        "external_only": { "type": "boolean", "description": "Whether to return only links pointing to external origins (default: false)" },
                        "filter_path": { "type": "string", "description": "Optional substring filter to match against URLs" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_state",
                "description": "Atomically inspect the complete interactive, disabled, focused, checked, readonly, and visibility state of an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_toggle_element_class",
                "description": "Add, remove, toggle, or check a CSS class name on the target DOM element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "class_name": { "type": "string", "description": "CSS class name to modify" },
                        "action": { "type": "string", "description": "Action to perform: 'toggle' (default), 'add', 'remove', 'contains'" }
                    },
                    "required": ["target", "class_name"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_element_text",
                "description": "Directly overwrite the textContent of target element and dispatch input/change events",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "text": { "type": "string", "description": "New text content to assign" }
                    },
                    "required": ["target", "text"]
                }
            }),
            serde_json::json!({
                "name": "browser_dispatch_custom_event",
                "description": "Construct and dispatch a synthetic CustomEvent on target element with optional detail payload",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "event_type": { "type": "string", "description": "Event type name (e.g. 'change', 'submit', 'cart:refresh')" },
                        "bubbles": { "type": "boolean", "description": "Whether the event bubbles up the DOM (default: true)" },
                        "cancelable": { "type": "boolean", "description": "Whether the event can be canceled (default: true)" },
                        "detail": { "type": "object", "description": "Custom JSON detail data attached to the event" }
                    },
                    "required": ["target", "event_type"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_active_element",
                "description": "Inspect the currently focused DOM element (document.activeElement)",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_check_element_clickable",
                "description": "Verify whether an element is physically clickable at its center coordinates, identifying any occluding DOM element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_dispatch_wheel_event",
                "description": "Dispatch a realistic WheelEvent (scroll/zoom/pan) on target element with deltaX, deltaY, and ctrlKey",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "delta_x": { "type": "number", "description": "Horizontal scroll delta (default: 0.0)" },
                        "delta_y": { "type": "number", "description": "Vertical scroll delta (default: 0.0)" },
                        "ctrl_key": { "type": "boolean", "description": "Whether Ctrl key is pressed for pinch-to-zoom (default: false)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_css_variables",
                "description": "Inspect computed CSS custom properties (variables) on target element or root documentElement",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID (omit to inspect root documentElement)" },
                        "variable_names": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Array of CSS variable names to query (e.g. ['--primary-color', '--bg-surface'])"
                        }
                    },
                    "required": ["variable_names"]
                }
            }),
            serde_json::json!({
                "name": "browser_clear_highlights",
                "description": "Remove all injected focus rings, Set-Of-Marks overlays, and visual debugging markers",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_simulate_pointer_click",
                "description": "Dispatch a full sequence of PointerEvents (pointerover, pointerenter, pointerdown, mousedown, pointerup, mouseup, click) at target center coordinates",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "click_count": { "type": "integer", "description": "Number of clicks to dispatch (default: 1)" },
                        "button": { "type": "integer", "description": "Mouse button (0 for left, 1 for middle, 2 for right) (default: 0)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_extract_meta_tags",
                "description": "Extract structured page metadata, OpenGraph tags, Twitter cards, links, and JSON-LD schemas from document",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "include_json_ld": { "type": "boolean", "description": "Whether to parse and include <script type=\"application/ld+json\"> blocks (default: true)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_document_geometry",
                "description": "Extract complete document dimensions, viewport metrics, scroll offsets, scroll percentages, and scrollable container count",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_set_device_orientation",
                "description": "Emulate mobile device orientation tilt angles (alpha, beta, gamma) and dispatch deviceorientation event",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "alpha": { "type": "number", "description": "Motion of the device around the z axis in degrees (0 to 360) (default: 0.0)" },
                        "beta": { "type": "number", "description": "Motion of the device around the x axis in degrees (-180 to 180) (default: 0.0)" },
                        "gamma": { "type": "number", "description": "Motion of the device around the y axis in degrees (-90 to 90) (default: 0.0)" },
                        "absolute": { "type": "boolean", "description": "Whether orientation is provided with absolute Earth coordinate frame (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_clear_browser_cache",
                "description": "Purge browser in-memory and disk caches via CDP, and optionally reset localStorage, sessionStorage, and cookies",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear_storage": { "type": "boolean", "description": "Whether to also clear localStorage and sessionStorage (default: false)" },
                        "clear_cookies": { "type": "boolean", "description": "Whether to also clear all document cookies (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_spacing",
                "description": "Inspect box-model computed spacing (margin, padding, border, dimensions, z-index, box-sizing) for an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_wait_for_url",
                "description": "Wait until the browser URL matches a target substring or regular expression pattern",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": { "type": "string", "description": "Substring or regex pattern to wait for in window.location.href" },
                        "timeout_ms": { "type": "integer", "description": "Maximum wait timeout in milliseconds (default: 5000)" }
                    },
                    "required": ["pattern"]
                }
            }),
            serde_json::json!({
                "name": "browser_dispatch_touch_event",
                "description": "Dispatch synthetic touch events (touchstart, touchmove, touchend, touchcancel) with multiple touch points",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "event_type": { "type": "string", "description": "Touch event type: 'touchstart', 'touchmove', 'touchend', 'touchcancel'" },
                        "touches": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "identifier": { "type": "integer" },
                                    "target": { "type": "string" },
                                    "x": { "type": "number" },
                                    "y": { "type": "number" },
                                    "radius_x": { "type": "number" },
                                    "radius_y": { "type": "number" },
                                    "rotation_angle": { "type": "number" },
                                    "force": { "type": "number" }
                                },
                                "required": ["target", "x", "y"]
                            },
                            "description": "List of active touch points"
                        }
                    },
                    "required": ["event_type"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_a11y_tree",
                "description": "Extract a hierarchical computed WAI-ARIA accessibility (a11y) tree for navigation and screen reader analysis",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of root subtree (default: body)" },
                        "max_depth": { "type": "integer", "description": "Maximum tree depth to traverse (default: 5)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_simulate_keyboard_chord",
                "description": "Dispatch a keyboard hotkey chord (e.g. 'Control+Shift+P', 'Meta+K', 'Alt+Tab') with realistic keydown/keyup sequence",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "chord": { "type": "string", "description": "Key combination (e.g. 'Control+Shift+P', 'Meta+A', 'Alt+Tab')" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID to target" }
                    },
                    "required": ["chord"]
                }
            }),
            serde_json::json!({
                "name": "browser_highlight_bounding_boxes",
                "description": "Render visual bounding boxes with label badges over page regions for grounding and multimodal verification",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "boxes": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "x": { "type": "number" },
                                    "y": { "type": "number" },
                                    "width": { "type": "number" },
                                    "height": { "type": "number" },
                                    "label": { "type": "string" },
                                    "color": { "type": "string" }
                                },
                                "required": ["x", "y", "width", "height"]
                            },
                            "description": "List of bounding box regions to annotate"
                        }
                    },
                    "required": ["boxes"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_attributes",
                "description": "Extract all HTML attributes of a DOM element as a complete key-value dictionary",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_filter",
                "description": "Inspect CSS filters, backdrop filters, opacity, pointer-events, and ancestor occlusion for an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_dismiss_all_modals",
                "description": "Dismiss modal overlays, cookie consent popups, backdrop dialogs, and restore body scroll",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_color_palette",
                "description": "Extract dominant computed colors and background colors used across visible elements on the page",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "description": "Maximum number of dominant colors to return (default: 15)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_drag_and_drop_element",
                "description": "Drag a source element and drop it onto a target element with multi-step pointer and drag event simulation",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source": { "type": "string", "description": "CSS selector or numeric ref ID of the source element to drag" },
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element to drop onto" },
                        "steps": { "type": "integer", "description": "Number of intermediate drag move steps (default: 10)" },
                        "hold_ms": { "type": "integer", "description": "Millisecond pause between dragstart and dragmove (default: 0)" }
                    },
                    "required": ["source", "target"]
                }
            }),
            serde_json::json!({
                "name": "browser_audit_forms",
                "description": "Audit form elements, input validation constraints (required, pattern, min/max), and HTML5 validity states",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of specific form or container" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_viewport_paging",
                "description": "Calculate viewport dimensions, multi-page scroll coordinates, total pages, and current scroll progress",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_computed_pseudo_element",
                "description": "Inspect computed CSS styles of pseudo-elements like ::before or ::after (e.g. icons, badges, content strings)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "pseudo": { "type": "string", "description": "Pseudo-element identifier (default: '::before', e.g. '::after', '::marker')" },
                        "properties": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of CSS property names to query (e.g. ['content', 'color', 'display'])"
                        }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_simulate_clipboard_paste",
                "description": "Simulate a clipboard paste event into a target or active element with DataTransfer text payload",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Text content to paste" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["text"]
                }
            }),
            serde_json::json!({
                "name": "browser_smart_fill",
                "description": "Smartly auto-fill form controls by fuzzy matching human-readable labels, names, or placeholders to values",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mapping": { "type": "object", "description": "Key-value mapping of field labels/names to input values" },
                        "submit": { "type": "boolean", "description": "Whether to auto-submit the form after filling" }
                    },
                    "required": ["mapping"]
                }
            }),
            serde_json::json!({
                "name": "browser_observe_text",
                "description": "Observe streaming text changes on a target element until updates settle or timeout expires",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" },
                        "timeout_ms": { "type": "integer", "description": "Maximum observation timeout in milliseconds (default: 10000)" },
                        "settle_ms": { "type": "integer", "description": "Quiet period with no changes before settling (default: 1000)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_set_visibility_state",
                "description": "Override document.visibilityState ('visible'/'hidden') to prevent bot pausing or idle detection",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "state": { "type": "string", "enum": ["visible", "hidden"], "description": "Target visibility state (default: 'visible')" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_computed_font",
                "description": "Extract computed font typography, font family, weight, line-height, and canvas measurement metrics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_action_map",
                "description": "Extract dense interactive element click-map with bounding box centroid coordinates and numeric ref bindings",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "viewport_only": { "type": "boolean", "description": "Only return elements currently inside the visible viewport (default: true)" },
                        "limit": { "type": "integer", "description": "Maximum number of interactive elements to return (default: 50)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_motion_state",
                "description": "Inspect active CSS animations and transitions on the page or target element to detect motion stability",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of specific element" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_listeners",
                "description": "Inspect event listeners, inline handlers, cursor styles, and interactive affordances of an element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_web_vitals",
                "description": "Extract Core Web Vitals, performance navigation timings (TTFB, FCP, DOMContentLoaded), and memory heap metrics",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_select_options",
                "description": "Inspect selectable options, option groups, values, text labels, and selected indices of a <select> element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target <select> element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_fuzzy_find_elements",
                "description": "Fuzzy search interactive elements by text content, ARIA labels, or attributes with similarity scoring",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search term or approximate text to match against elements" },
                        "limit": { "type": "integer", "description": "Maximum matching elements to return (default: 10)" },
                        "container": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container element to restrict search" }
                    },
                    "required": ["query"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_heading_hierarchy",
                "description": "Extract hierarchical heading structure (h1-h6) with levels, text content, IDs, and anchor links",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "container": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container element to restrict search" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_stacking",
                "description": "Inspect CSS stacking context, z-index, position, opacity, transform, and ancestor clipping bounds",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of the target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_query_shadow_elements",
                "description": "Recursively query elements across both light DOM and all open shadow roots",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "selector": { "type": "string", "description": "CSS selector to search for across shadow trees" },
                        "limit": { "type": "integer", "description": "Maximum matching elements to return (default: 50)" },
                        "container": { "type": "string", "description": "Optional CSS selector or numeric ref ID of root container" }
                    },
                    "required": ["selector"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_visibility_matrix",
                "description": "Perform 5-point occlusion testing and compute visibility ratio against viewport and ancestor clipping",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_error_digest",
                "description": "Retrieve aggregated diagnostics of runtime JavaScript errors, unhandled rejections, and console errors",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "max_errors": { "type": "integer", "description": "Maximum errors to return (default: 50)" },
                        "clear": { "type": "boolean", "description": "Whether to clear recorded error buffer after reading (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_extract_grid_data",
                "description": "Extract structured datasets from HTML tables and ARIA grids/treegrids into column-mapped row records",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of table or grid container" },
                        "max_rows": { "type": "integer", "description": "Maximum number of rows to extract (default: 100)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_diff_form_inputs",
                "description": "Inspect all form inputs and compare pristine default values with current values to identify dirty fields",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of form or container" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_unstick_scroll_locks",
                "description": "Neutralize overflow:hidden locks, fixed position body wrappers, and invisible click backdrops",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_diff_dom_text",
                "description": "Compute text mutations and diff between current visible content and a baseline text snapshot",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "baseline": { "type": "string", "description": "Optional baseline text content or previous checkpoint" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_frame_hierarchy",
                "description": "Inspect embedded frames and iframes with cross-origin detection and viewport coordinates",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_role_elements",
                "description": "Filter and extract interactive elements matching specific ARIA roles or semantics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "roles": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "List of ARIA roles to filter (e.g. ['button', 'link', 'textbox', 'combobox'])"
                        },
                        "viewport_only": { "type": "boolean", "description": "Only return elements currently inside visible viewport (default: true)" },
                        "limit": { "type": "integer", "description": "Maximum matching elements to return (default: 50)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_harvest_resources",
                "description": "Harvest all page resources (images, scripts, styles, media, and iframes) with resolved absolute URLs",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "resource_types": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Resource categories to harvest (e.g. ['images', 'scripts', 'styles', 'media', 'frames'])"
                        }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_detect_bot_challenges",
                "description": "Scan page for active bot challenges, CAPTCHAs (Turnstile, reCAPTCHA, hCaptcha), and WAF blocks",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            }),
            serde_json::json!({
                "name": "browser_get_scroll_state",
                "description": "Inspect scrollability metrics, directional scroll boundaries, and progress for an element or document",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of container element (default: document)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_proximity_elements",
                "description": "Find and rank nearby interactive elements sorted by Euclidean distance from an anchor point or element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "x": { "type": "number", "description": "Anchor viewport X coordinate" },
                        "y": { "type": "number", "description": "Anchor viewport Y coordinate" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of anchor element" },
                        "limit": { "type": "integer", "description": "Maximum neighboring elements to return (default: 10)" },
                        "radius": { "type": "number", "description": "Maximum Euclidean distance in pixels to search within" },
                        "interactive_only": { "type": "boolean", "description": "Only return actionable/interactive elements (default: true)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_emulate_screen_metrics",
                "description": "Emulate responsive viewport dimensions, scale factor, screen orientation, and color scheme",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "width": { "type": "integer", "description": "Viewport width in CSS pixels" },
                        "height": { "type": "integer", "description": "Viewport height in CSS pixels" },
                        "device_scale_factor": { "type": "number", "description": "Device pixel ratio (e.g. 2.0 or 3.0)" },
                        "mobile": { "type": "boolean", "description": "Enable mobile emulation and touch point support" },
                        "orientation": { "type": "string", "description": "Screen orientation (e.g. 'portraitPrimary', 'landscapePrimary')" },
                        "color_scheme": { "type": "string", "description": "Preferred color scheme ('light', 'dark', 'no-preference')" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_export_table_csv",
                "description": "Extract structured table or grid elements formatted directly as clean RFC 4180 CSV",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of table/grid element" },
                        "include_headers": { "type": "boolean", "description": "Include header row in CSV export (default: true)" },
                        "delimiter": { "type": "string", "description": "Delimiter character (default: ',')" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_reset_form",
                "description": "Reset target form or container form controls back to pristine default values",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of form or container" },
                        "trigger_events": { "type": "boolean", "description": "Dispatch reset, input, and change events (default: true)" },
                        "clear_custom_validations": { "type": "boolean", "description": "Clear custom validity and aria-invalid flags (default: true)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_datalist_options",
                "description": "Inspect selectable options in an HTML5 datalist or linked autocomplete input combobox",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of input or datalist" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_canvas_context",
                "description": "Inspect canvas rendering context (2D, WebGL, WebGL2), dimensions, and renderer hardware metrics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of canvas element" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_control_media_playback",
                "description": "Inspect and control HTML5 audio/video playback elements with playback state telemetry",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "description": "Action to perform ('status', 'play', 'pause', 'seek', 'volume', 'speed', 'mute', 'unmute')" },
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of media element" },
                        "value": { "type": "number", "description": "Numeric parameter for seek time (seconds), volume (0.0-1.0), or playback rate" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_element_ancestry",
                "description": "Inspect the full DOM ancestry tree and selector hierarchy path from an element up to document root",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of target element" },
                        "max_depth": { "type": "integer", "description": "Maximum tree depth to traverse upwards (default: 20)" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_layout_metrics",
                "description": "Inspect CSS Grid and Flexbox layout metrics, alignments, tracks, and child coordinates",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "Optional CSS selector or numeric ref ID of layout container" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_audit_contrast_ratio",
                "description": "Compute WCAG 2.1 relative luminance, contrast ratio, and accessibility compliance",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of target element" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_dispatch_advanced_event",
                "description": "Dispatch PointerEvent, MouseEvent, or KeyboardEvent with synthetic modifier keys",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or numeric ref ID of target element" },
                        "event_type": { "type": "string", "description": "Event type name (e.g. 'click', 'pointerdown', 'keydown')" },
                        "modifiers": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Modifier keys to activate (e.g. ['ctrl', 'shift', 'alt', 'meta'])"
                        },
                        "button": { "type": "integer", "description": "Mouse button index (0=left, 1=middle, 2=right)" },
                        "pressure": { "type": "number", "description": "Pointer pressure between 0.0 and 1.0 (default: 0.5)" },
                        "detail": { "type": "integer", "description": "Event detail click count (default: 1)" }
                    },
                    "required": ["target", "event_type"]
                }
            }),
            serde_json::json!({
                "name": "browser_get_network_har",
                "description": "Harvest intercepted network activity logs formatted as a standard HAR 1.2 archive",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "clear": { "type": "boolean", "description": "Clear the network log buffer after export (default: false)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_caret_position",
                "description": "Inspect or set the text caret position, selection start/end offsets, and highlighted text in an input, textarea, or contenteditable element",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": { "type": "string", "description": "CSS selector or element ref key" },
                        "set_start": { "type": "integer", "description": "Optional character index to move selection start to" },
                        "set_end": { "type": "integer", "description": "Optional character index to move selection end to" }
                    },
                    "required": ["target"]
                }
            }),
            serde_json::json!({
                "name": "browser_dismiss_cookie_banners",
                "description": "Auto-detect and click Accept/Agree/Dismiss on popular GDPR, CCPA, and CMP cookie consent dialogs and overlays (OneTrust, Cookiebot, Klaro, TrustArc, Quantcast)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "click_accept": { "type": "boolean", "description": "Click the accept/consent button if true, or reject/close button if false (default: true)" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_emulate_speech_api",
                "description": "Emulate Web Speech API (webkitSpeechRecognition/SpeechRecognition) to feed synthetic voice transcripts or control speech recognition state",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "description": "Action to perform: 'mock_recognition', 'feed_recognition', 'get_state', or 'reset'" },
                        "mock_transcript": { "type": "string", "description": "Synthetic voice transcript text to feed into onresult listener" }
                    }
                }
            }),
            serde_json::json!({
                "name": "browser_get_dom_mutations",
                "description": "Inspect and harvest real-time DOM mutation logs captured by the background MutationObserver (added/removed elements, attributes, target tags)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "limit": { "type": "integer", "description": "Maximum number of recent mutation records to retrieve (default: 50)" },
                        "clear": { "type": "boolean", "description": "Clear the mutation buffer after harvesting (default: true)" }
                    }
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
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'target' parameter");
                }
                let padding = args["padding"].as_i64().or_else(|| args["pad"].as_i64()).unwrap_or(0) as i32;
                AgentCommand::ScreenshotElement { target, padding }
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
            "browser_assert_element" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let assertions: crate::agent::protocol::ElementAssertion = serde_json::from_value(args.clone()).unwrap_or_default();
                AgentCommand::AssertElement { target, assertions }
            }
            "browser_wait_for_selector" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let state = args["state"].as_str().unwrap_or("visible").to_string();
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(5000);
                AgentCommand::WaitForSelector { target, state, timeout_ms }
            }
            "browser_get_performance_metrics" => {
                AgentCommand::GetPerformanceMetrics
            }
            "browser_scroll_to_coords" => {
                let x = args["x"].as_i64().or_else(|| args["left"].as_i64()).unwrap_or(0) as i32;
                let y = args["y"].as_i64().or_else(|| args["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = args["smooth"].as_bool().unwrap_or(false);
                AgentCommand::ScrollToCoords { x, y, smooth }
            }
            "browser_get_page_outline" => {
                let max_links = args["max_links"].as_u64().or_else(|| args["links"].as_u64()).unwrap_or(25) as usize;
                let max_buttons = args["max_buttons"].as_u64().or_else(|| args["buttons"].as_u64()).unwrap_or(25) as usize;
                AgentCommand::GetPageOutline { max_links, max_buttons }
            }
            "browser_wait_for_network_idle" => {
                let idle_time_ms = args["idle_time_ms"].as_u64().or_else(|| args["idle"].as_u64()).unwrap_or(500);
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(10000);
                let max_inflight = args["max_inflight"].as_u64().or_else(|| args["max"].as_u64()).unwrap_or(0) as usize;
                AgentCommand::WaitForNetworkIdle { idle_time_ms, timeout_ms, max_inflight }
            }
            "browser_watch_mutations" => {
                let action = args["action"].as_str().unwrap_or("get").to_string();
                let target = args["target"].as_str().map(|s| s.to_string());
                AgentCommand::WatchMutations { action, target }
            }
            "browser_tab_eval" => {
                let tab_index = args["tab_index"].as_u64().or_else(|| args["tab"].as_u64()).unwrap_or(0) as usize;
                let script = args["script"].as_str().or_else(|| args["expression"].as_str()).unwrap_or("").to_string();
                if script.is_empty() { return Self::tool_error(req_id, "Missing 'script' parameter"); }
                AgentCommand::TabEval { tab_index, script }
            }
            "browser_get_response_body" => {
                let pattern = args["pattern"].as_str().or_else(|| args["url"].as_str()).unwrap_or("").to_string();
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::GetResponseBody { pattern, clear }
            }
            "browser_drop_file" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let file_name = args["file_name"].as_str().or_else(|| args["name"].as_str()).unwrap_or("upload.bin").to_string();
                let mime_type = args["mime_type"].as_str().or_else(|| args["type"].as_str()).unwrap_or("application/octet-stream").to_string();
                let content = args["content"].as_str().or_else(|| args["data"].as_str()).unwrap_or("").to_string();
                AgentCommand::DropFile { target, file_name, mime_type, content }
            }
            "browser_get_dom_tree" => {
                let target = args["target"].as_str().map(|s| s.to_string());
                let max_depth = args["max_depth"].as_u64().or_else(|| args["depth"].as_u64()).unwrap_or(4) as usize;
                AgentCommand::GetDomTree { target, max_depth }
            }
            "browser_set_offline" => {
                let offline = args["offline"].as_bool().unwrap_or(true);
                AgentCommand::SetOffline { offline }
            }
            "browser_wait_for_text" => {
                let text = args["text"].as_str().unwrap_or("").to_string();
                if text.is_empty() { return Self::tool_error(req_id, "Missing 'text' parameter"); }
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(5000);
                AgentCommand::WaitForText { target, text, timeout_ms }
            }
            "browser_get_computed_styles" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let properties = if let Some(arr) = args["properties"].as_array().or_else(|| args["props"].as_array()) {
                    arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                } else {
                    vec![
                        "display".into(),
                        "visibility".into(),
                        "opacity".into(),
                        "color".into(),
                        "background-color".into(),
                        "z-index".into(),
                        "font-size".into(),
                        "cursor".into(),
                    ]
                };
                AgentCommand::GetComputedStyles { target, properties }
            }
            "browser_wait_for_download" => {
                let file_pattern = args["file_pattern"].as_str().or_else(|| args["pattern"].as_str()).or_else(|| args["file"].as_str()).map(|s| s.to_string());
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(15000);
                AgentCommand::WaitForDownload { file_pattern, timeout_ms }
            }
            "browser_set_content" => {
                let html = args["html"].as_str().or_else(|| args["content"].as_str()).unwrap_or("").to_string();
                if html.is_empty() { return Self::tool_error(req_id, "Missing 'html' parameter"); }
                AgentCommand::SetContent { html }
            }
            "browser_get_forms" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetForms { target }
            }
            "browser_set_visibility" => {
                let state = args["state"].as_str().unwrap_or("visible").to_string();
                AgentCommand::SetVisibility { state }
            }
            "browser_emulate_media" => {
                let media_type = args["media_type"].as_str().or_else(|| args["media"].as_str()).map(|s| s.to_string());
                let color_scheme = args["color_scheme"].as_str().or_else(|| args["scheme"].as_str()).map(|s| s.to_string());
                AgentCommand::EmulateMedia { media_type, color_scheme }
            }
            "browser_get_element_bounds" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementBounds { target }
            }
            "browser_mock_route" => {
                let action = args["action"].as_str().unwrap_or("set").to_string();
                let pattern = args["pattern"].as_str().or_else(|| args["url"].as_str()).map(|s| s.to_string());
                let status = args["status"].as_u64().unwrap_or(200) as u16;
                let content_type = args["content_type"].as_str().or_else(|| args["type"].as_str()).unwrap_or("application/json").to_string();
                let body = if let Some(b) = args.get("body") {
                    if let Some(s) = b.as_str() { Some(s.to_string()) } else { Some(b.to_string()) }
                } else {
                    None
                };
                AgentCommand::MockRoute { action, pattern, status, content_type, body }
            }
            "browser_get_viewport" => {
                AgentCommand::GetViewport
            }
            "browser_set_element_attribute" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let attribute = args["attribute"].as_str().or_else(|| args["attr"].as_str()).unwrap_or("").to_string();
                if attribute.is_empty() { return Self::tool_error(req_id, "Missing 'attribute' parameter"); }
                let value = args["value"].as_str().or_else(|| args["val"].as_str()).map(|s| s.to_string());
                AgentCommand::SetElementAttribute { target, attribute, value }
            }
            "browser_search_page_text" => {
                let query = args["query"].as_str().unwrap_or("").to_string();
                if query.is_empty() { return Self::tool_error(req_id, "Missing 'query' parameter"); }
                let max_matches = args["max_matches"].as_u64().map(|v| v as usize).unwrap_or(20);
                let case_sensitive = args["case_sensitive"].as_bool().unwrap_or(false);
                AgentCommand::SearchPageText { query, max_matches, case_sensitive }
            }
            "browser_dump_storage" => {
                let storage_type = args["storage_type"].as_str().unwrap_or("all").to_string();
                AgentCommand::DumpStorage { storage_type }
            }
            "browser_human_jitter" => {
                let duration_ms = args["duration_ms"].as_u64().unwrap_or(800);
                let move_mouse = args["move_mouse"].as_bool().unwrap_or(true);
                let micro_scroll = args["micro_scroll"].as_bool().unwrap_or(true);
                AgentCommand::HumanJitter { duration_ms, move_mouse, micro_scroll }
            }
            "browser_type_keystrokes" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let text = args["text"].as_str().unwrap_or("").to_string();
                let delay_ms = args["delay_ms"].as_u64().unwrap_or(25);
                let clear = args["clear"].as_bool().unwrap_or(false);
                let submit = args["submit"].as_bool().unwrap_or(false);
                AgentCommand::TypeKeystrokes { target, text, delay_ms, clear, submit }
            }
            "browser_get_canvas_data" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let mime_type = args["mime_type"].as_str().or_else(|| args["format"].as_str()).unwrap_or("image/png").to_string();
                let quality = args["quality"].as_f64().unwrap_or(0.92);
                AgentCommand::GetCanvasData { target, mime_type, quality }
            }
            "browser_get_resource_timing" => {
                let initiator_type = args["initiator_type"].as_str().or_else(|| args["initiator"].as_str()).map(|s| s.to_string());
                let max_entries = args["max_entries"].as_u64().or_else(|| args["max"].as_u64()).unwrap_or(50) as usize;
                let min_duration_ms = args["min_duration_ms"].as_f64().or_else(|| args["min_duration"].as_f64()).unwrap_or(0.0);
                AgentCommand::GetResourceTiming { initiator_type, max_entries, min_duration_ms }
            }
            "browser_get_dom_stats" => {
                AgentCommand::GetDomStats
            }
            "browser_scroll_by" => {
                let dx = args["dx"].as_i64().or_else(|| args["left"].as_i64()).unwrap_or(0) as i32;
                let dy = args["dy"].as_i64().or_else(|| args["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = args["smooth"].as_bool().unwrap_or(false);
                AgentCommand::ScrollBy { dx, dy, smooth }
            }
            "browser_scroll_container" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let dx = args["dx"].as_i64().or_else(|| args["left"].as_i64()).unwrap_or(0) as i32;
                let dy = args["dy"].as_i64().or_else(|| args["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = args["smooth"].as_bool().unwrap_or(false);
                AgentCommand::ScrollContainer { target, dx, dy, smooth }
            }
            "browser_select_element_text" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let start = args["start"].as_u64().map(|v| v as usize);
                let length = args["length"].as_u64().or_else(|| args["len"].as_u64()).map(|v| v as usize);
                AgentCommand::SelectElementText { target, start, length }
            }
            "browser_extract_page_links" => {
                let max_links = args["max_links"].as_u64().or_else(|| args["max"].as_u64()).unwrap_or(100) as usize;
                let external_only = args["external_only"].as_bool().unwrap_or(false);
                let filter_path = args["filter_path"].as_str().or_else(|| args["filter"].as_str()).map(|s| s.to_string());
                AgentCommand::ExtractPageLinks { max_links, external_only, filter_path }
            }
            "browser_get_element_state" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementState { target }
            }
            "browser_toggle_element_class" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let class_name = args["class_name"].as_str().or_else(|| args["class"].as_str()).unwrap_or("").to_string();
                if class_name.is_empty() { return Self::tool_error(req_id, "Missing 'class_name' parameter"); }
                let action = args["action"].as_str().unwrap_or("toggle").to_string();
                AgentCommand::ToggleElementClass { target, class_name, action }
            }
            "browser_set_element_text" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let text = args["text"].as_str().unwrap_or("").to_string();
                AgentCommand::SetElementText { target, text }
            }
            "browser_dispatch_custom_event" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let event_type = args["event_type"].as_str().or_else(|| args["event"].as_str()).unwrap_or("").to_string();
                if event_type.is_empty() { return Self::tool_error(req_id, "Missing 'event_type' parameter"); }
                let bubbles = args["bubbles"].as_bool().unwrap_or(true);
                let cancelable = args["cancelable"].as_bool().unwrap_or(true);
                let detail = args.get("detail").cloned();
                AgentCommand::DispatchCustomEvent { target, event_type, bubbles, cancelable, detail }
            }
            "browser_get_active_element" => {
                AgentCommand::GetActiveElement
            }
            "browser_check_element_clickable" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::CheckElementClickable { target }
            }
            "browser_dispatch_wheel_event" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let delta_x = args["delta_x"].as_f64().or_else(|| args["dx"].as_f64()).unwrap_or(0.0);
                let delta_y = args["delta_y"].as_f64().or_else(|| args["dy"].as_f64()).unwrap_or(0.0);
                let ctrl_key = args["ctrl_key"].as_bool().unwrap_or(false);
                AgentCommand::DispatchWheelEvent { target, delta_x, delta_y, ctrl_key }
            }
            "browser_get_css_variables" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let variable_names = args["variable_names"].as_array()
                    .or_else(|| args["variables"].as_array())
                    .or_else(|| args["vars"].as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                AgentCommand::GetCssVariables { target, variable_names }
            }
            "browser_clear_highlights" => {
                AgentCommand::ClearHighlights
            }
            "browser_simulate_pointer_click" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let click_count = args["click_count"].as_u64().or_else(|| args["count"].as_u64()).unwrap_or(1) as usize;
                let button = args["button"].as_i64().unwrap_or(0) as i32;
                AgentCommand::SimulatePointerClick { target, click_count, button }
            }
            "browser_extract_meta_tags" => {
                let include_json_ld = args["include_json_ld"].as_bool().unwrap_or(true);
                AgentCommand::ExtractMetaTags { include_json_ld }
            }
            "browser_get_document_geometry" => {
                AgentCommand::GetDocumentGeometry
            }
            "browser_set_device_orientation" => {
                let alpha = args["alpha"].as_f64().unwrap_or(0.0);
                let beta = args["beta"].as_f64().unwrap_or(0.0);
                let gamma = args["gamma"].as_f64().unwrap_or(0.0);
                let absolute = args["absolute"].as_bool().unwrap_or(false);
                AgentCommand::SetDeviceOrientation { alpha, beta, gamma, absolute }
            }
            "browser_clear_browser_cache" => {
                let clear_storage = args["clear_storage"].as_bool().unwrap_or(false);
                let clear_cookies = args["clear_cookies"].as_bool().unwrap_or(false);
                AgentCommand::ClearBrowserCache { clear_storage, clear_cookies }
            }
            "browser_get_element_spacing" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementSpacing { target }
            }
            "browser_wait_for_url" => {
                let pattern = args["pattern"].as_str().or_else(|| args["url"].as_str()).unwrap_or("").to_string();
                if pattern.is_empty() { return Self::tool_error(req_id, "Missing 'pattern' parameter"); }
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(5000);
                AgentCommand::WaitForUrl { pattern, timeout_ms }
            }
            "browser_dispatch_touch_event" => {
                let event_type = args["event_type"].as_str().or_else(|| args["event"].as_str()).unwrap_or("touchstart").to_string();
                let touches: Vec<crate::agent::protocol::TouchPoint> = args["touches"].as_array()
                    .map(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
                    .unwrap_or_default();
                AgentCommand::DispatchTouchEvent { event_type, touches }
            }
            "browser_get_a11y_tree" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let max_depth = args["max_depth"].as_u64().or_else(|| args["depth"].as_u64()).unwrap_or(5) as usize;
                AgentCommand::GetA11yTree { target, max_depth }
            }
            "browser_simulate_keyboard_chord" => {
                let chord = args["chord"].as_str().or_else(|| args["keys"].as_str()).unwrap_or("").to_string();
                if chord.is_empty() { return Self::tool_error(req_id, "Missing 'chord' parameter"); }
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::SimulateKeyboardChord { chord, target }
            }
            "browser_highlight_bounding_boxes" => {
                let boxes: Vec<crate::agent::protocol::BoundingBoxMarker> = args["boxes"].as_array()
                    .map(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
                    .unwrap_or_default();
                AgentCommand::HighlightBoundingBoxes { boxes }
            }
            "browser_get_element_attributes" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementAttributes { target }
            }
            "browser_get_element_filter" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementFilter { target }
            }
            "browser_dismiss_all_modals" => AgentCommand::DismissAllModals,
            "browser_get_color_palette" => {
                let limit = args["limit"].as_u64().map(|n| n as usize).unwrap_or(15);
                AgentCommand::GetColorPalette { limit }
            }
            "browser_drag_and_drop_element" => {
                let source = args["source"].as_str().or_else(|| args["from"].as_str()).unwrap_or("").to_string();
                let target = args["target"].as_str().or_else(|| args["to"].as_str()).unwrap_or("").to_string();
                if source.is_empty() || target.is_empty() {
                    return Self::tool_error(req_id, "Missing 'source' or 'target' parameter");
                }
                let steps = args["steps"].as_u64().map(|s| s as u32).unwrap_or(10);
                let hold_ms = args["hold_ms"].as_u64().unwrap_or(0);
                AgentCommand::DragAndDropElement { source, target, steps, hold_ms }
            }
            "browser_audit_forms" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::AuditForms { target }
            }
            "browser_get_viewport_paging" => AgentCommand::GetViewportPaging,
            "browser_get_computed_pseudo_element" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let pseudo = args["pseudo"].as_str().unwrap_or("::before").to_string();
                let props: Vec<String> = args["properties"].as_array()
                    .or_else(|| args["props"].as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                AgentCommand::GetComputedPseudoElement { target, pseudo, properties: props }
            }
            "browser_simulate_clipboard_paste" => {
                let text = args["text"].as_str().unwrap_or("").to_string();
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::SimulateClipboardPaste { target, text }
            }
            "browser_smart_fill" => {
                let mapping = if let Some(obj) = args["mapping"].as_object() {
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                } else {
                    std::collections::HashMap::new()
                };
                let submit = args["submit"].as_bool().unwrap_or(false);
                AgentCommand::SmartFill { mapping, submit }
            }
            "browser_observe_text" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                let timeout_ms = args["timeout_ms"].as_u64().or_else(|| args["timeout"].as_u64()).unwrap_or(10000);
                let settle_ms = args["settle_ms"].as_u64().or_else(|| args["settle"].as_u64()).unwrap_or(1000);
                AgentCommand::ObserveText { target, timeout_ms, settle_ms }
            }
            "browser_set_visibility_state" => {
                let state = args["state"].as_str().unwrap_or("visible").to_string();
                AgentCommand::SetVisibilityState { state }
            }
            "browser_get_computed_font" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetComputedFont { target }
            }
            "browser_get_action_map" => {
                let viewport_only = args["viewport_only"].as_bool().unwrap_or(true);
                let max_elements = args["limit"].as_u64().or_else(|| args["max_elements"].as_u64()).map(|n| n as usize).unwrap_or(50);
                AgentCommand::GetActionMap { viewport_only, max_elements }
            }
            "browser_get_motion_state" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetMotionState { target }
            }
            "browser_get_element_listeners" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementListeners { target }
            }
            "browser_get_web_vitals" => AgentCommand::GetWebVitals,
            "browser_get_select_options" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetSelectOptions { target }
            }
            "browser_fuzzy_find_elements" => {
                let query = args["query"].as_str().or_else(|| args["text"].as_str()).unwrap_or("").to_string();
                if query.is_empty() { return Self::tool_error(req_id, "Missing 'query' parameter"); }
                let limit = args["limit"].as_u64().or_else(|| args["max"].as_u64()).map(|n| n as usize).unwrap_or(10);
                let container = args["container"].as_str().or_else(|| args["parent"].as_str()).map(|s| s.to_string());
                AgentCommand::FuzzyFindElements { query, limit, container }
            }
            "browser_get_heading_hierarchy" => {
                let container = args["container"].as_str().or_else(|| args["parent"].as_str()).map(|s| s.to_string());
                AgentCommand::GetHeadingHierarchy { container }
            }
            "browser_get_element_stacking" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementStacking { target }
            }
            "browser_query_shadow_elements" => {
                let selector = args["selector"].as_str().or_else(|| args["sel"].as_str()).unwrap_or("").to_string();
                if selector.is_empty() { return Self::tool_error(req_id, "Missing 'selector' parameter"); }
                let limit = args["limit"].as_u64().or_else(|| args["max"].as_u64()).map(|n| n as usize).unwrap_or(50);
                let container = args["container"].as_str().map(|s| s.to_string());
                AgentCommand::QueryShadowElements { selector, limit, container }
            }
            "browser_get_element_visibility_matrix" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() { return Self::tool_error(req_id, "Missing 'target' parameter"); }
                AgentCommand::GetElementVisibilityMatrix { target }
            }
            "browser_get_error_digest" => {
                let max_errors = args["max_errors"].as_u64().or_else(|| args["limit"].as_u64()).map(|n| n as usize).unwrap_or(50);
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::GetErrorDigest { max_errors, clear }
            }
            "browser_extract_grid_data" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let max_rows = args["max_rows"].as_u64().or_else(|| args["limit"].as_u64()).map(|n| n as usize).unwrap_or(100);
                AgentCommand::ExtractGridData { target, max_rows }
            }
            "browser_diff_form_inputs" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::DiffFormInputs { target }
            }
            "browser_unstick_scroll_locks" => {
                AgentCommand::UnstickScrollLocks
            }
            "browser_diff_dom_text" => {
                let baseline = args["baseline"].as_str().map(|s| s.to_string());
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::DiffDomText { baseline, target }
            }
            "browser_get_frame_hierarchy" => {
                AgentCommand::GetFrameHierarchy
            }
            "browser_get_role_elements" => {
                let roles = args["roles"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let viewport_only = args["viewport_only"].as_bool().unwrap_or(true);
                let limit = args["limit"].as_u64().or_else(|| args["max"].as_u64()).map(|n| n as usize).unwrap_or(50);
                AgentCommand::GetRoleElements { roles, viewport_only, limit }
            }
            "browser_harvest_resources" => {
                let types = args["resource_types"].as_array()
                    .or_else(|| args["types"].as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                AgentCommand::HarvestResources { resource_types: types }
            }
            "browser_detect_bot_challenges" => {
                AgentCommand::DetectBotChallenges
            }
            "browser_get_scroll_state" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetScrollState { target }
            }
            "browser_get_proximity_elements" => {
                let x = args["x"].as_f64();
                let y = args["y"].as_f64();
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let limit = args["limit"].as_u64().or_else(|| args["max"].as_u64()).map(|n| n as usize).unwrap_or(10);
                let radius = args["radius"].as_f64();
                let interactive_only = args["interactive_only"].as_bool().unwrap_or(true);
                AgentCommand::GetProximityElements { x, y, target, limit, radius, interactive_only }
            }
            "browser_emulate_screen_metrics" => {
                let width = args["width"].as_u64().map(|n| n as u32);
                let height = args["height"].as_u64().map(|n| n as u32);
                let device_scale_factor = args["device_scale_factor"].as_f64().or_else(|| args["scale"].as_f64());
                let mobile = args["mobile"].as_bool().unwrap_or(false);
                let orientation = args["orientation"].as_str().map(|s| s.to_string());
                let color_scheme = args["color_scheme"].as_str().or_else(|| args["theme"].as_str()).map(|s| s.to_string());
                AgentCommand::EmulateScreenMetrics { width, height, device_scale_factor, mobile, orientation, color_scheme }
            }
            "browser_export_table_csv" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let include_headers = args["include_headers"].as_bool().or_else(|| args["headers"].as_bool()).unwrap_or(true);
                let delimiter = args["delimiter"].as_str().unwrap_or(",").to_string();
                AgentCommand::ExportTableCsv { target, include_headers, delimiter }
            }
            "browser_reset_form" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let trigger_events = args["trigger_events"].as_bool().or_else(|| args["events"].as_bool()).unwrap_or(true);
                let clear_custom_validations = args["clear_custom_validations"].as_bool().or_else(|| args["clear_validations"].as_bool()).unwrap_or(true);
                AgentCommand::ResetForm { target, trigger_events, clear_custom_validations }
            }
            "browser_get_datalist_options" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetDatalistOptions { target }
            }
            "browser_get_canvas_context" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetCanvasContext { target }
            }
            "browser_control_media_playback" => {
                let action = args["action"].as_str().unwrap_or("status").to_string();
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                let value = args["value"].as_f64().or_else(|| args["val"].as_f64());
                AgentCommand::ControlMediaPlayback { action, target, value }
            }
            "browser_get_element_ancestry" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                let max_depth = args["max_depth"].as_u64().or_else(|| args["depth"].as_u64()).map(|n| n as usize).unwrap_or(20);
                AgentCommand::GetElementAncestry { target, max_depth }
            }
            "browser_get_layout_metrics" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).map(|s| s.to_string());
                AgentCommand::GetLayoutMetrics { target }
            }
            "browser_audit_contrast_ratio" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                AgentCommand::AuditContrastRatio { target }
            }
            "browser_dispatch_advanced_event" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                let event_type = args["event_type"].as_str().or_else(|| args["event"].as_str()).unwrap_or("click").to_string();
                let modifiers = args["modifiers"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_lowercase())).collect())
                    .unwrap_or_default();
                let button = args["button"].as_i64().map(|n| n as i32).unwrap_or(0);
                let pressure = args["pressure"].as_f64().unwrap_or(0.5);
                let detail = args["detail"].as_i64().map(|n| n as i32).unwrap_or(1);
                AgentCommand::DispatchAdvancedEvent { target, event_type, modifiers, button, pressure, detail }
            }
            "browser_get_network_har" => {
                let clear = args["clear"].as_bool().unwrap_or(false);
                AgentCommand::GetNetworkHar { clear }
            }
            "browser_get_caret_position" => {
                let target = args["target"].as_str().or_else(|| args["ref"].as_str()).unwrap_or("").to_string();
                let set_start = args["set_start"].as_u64().map(|n| n as usize);
                let set_end = args["set_end"].as_u64().map(|n| n as usize);
                AgentCommand::GetCaretPosition { target, set_start, set_end }
            }
            "browser_dismiss_cookie_banners" => {
                let click_accept = args["click_accept"].as_bool().unwrap_or(true);
                AgentCommand::DismissCookieBanners { click_accept }
            }
            "browser_emulate_speech_api" => {
                let action = args["action"].as_str().map(|s| s.to_string());
                let mock_transcript = args["mock_transcript"].as_str().or_else(|| args["transcript"].as_str()).map(|s| s.to_string());
                AgentCommand::EmulateSpeechApi { action, mock_transcript }
            }
            "browser_get_dom_mutations" => {
                let limit = args["limit"].as_u64().map(|n| n as usize).unwrap_or(50);
                let clear = args["clear"].as_bool().unwrap_or(true);
                AgentCommand::GetDomMutations { limit, clear }
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
        assert!(tool_names.contains(&"browser_assert_element"));
        assert!(tool_names.contains(&"browser_wait_for_selector"));
        assert!(tool_names.contains(&"browser_get_performance_metrics"));
        assert!(tool_names.contains(&"browser_scroll_to_coords"));
        assert!(tool_names.contains(&"browser_get_page_outline"));
        assert!(tool_names.contains(&"browser_wait_for_network_idle"));
        assert!(tool_names.contains(&"browser_watch_mutations"));
        assert!(tool_names.contains(&"browser_tab_eval"));
        assert!(tool_names.contains(&"browser_get_response_body"));
        assert!(tool_names.contains(&"browser_drop_file"));
        assert!(tool_names.contains(&"browser_get_dom_tree"));
        assert!(tool_names.contains(&"browser_set_offline"));
        assert!(tool_names.contains(&"browser_wait_for_text"));
        assert!(tool_names.contains(&"browser_get_computed_styles"));
        assert!(tool_names.contains(&"browser_wait_for_download"));
        assert!(tool_names.contains(&"browser_set_content"));
        assert!(tool_names.contains(&"browser_get_forms"));
        assert!(tool_names.contains(&"browser_set_visibility"));
        assert!(tool_names.contains(&"browser_emulate_media"));
        assert!(tool_names.contains(&"browser_get_element_bounds"));
        assert!(tool_names.contains(&"browser_mock_route"));
        assert!(tool_names.contains(&"browser_get_viewport"));
        assert!(tool_names.contains(&"browser_set_element_attribute"));
        assert!(tool_names.contains(&"browser_search_page_text"));
        assert!(tool_names.contains(&"browser_dump_storage"));
        assert!(tool_names.contains(&"browser_human_jitter"));
        assert!(tool_names.contains(&"browser_type_keystrokes"));
        assert!(tool_names.contains(&"browser_get_canvas_data"));
        assert!(tool_names.contains(&"browser_get_resource_timing"));
        assert!(tool_names.contains(&"browser_get_dom_stats"));
        assert!(tool_names.contains(&"browser_scroll_by"));
        assert!(tool_names.contains(&"browser_scroll_container"));
        assert!(tool_names.contains(&"browser_select_element_text"));
        assert!(tool_names.contains(&"browser_extract_page_links"));
        assert!(tool_names.contains(&"browser_get_element_state"));
        assert!(tool_names.contains(&"browser_toggle_element_class"));
        assert!(tool_names.contains(&"browser_set_element_text"));
        assert!(tool_names.contains(&"browser_dispatch_custom_event"));
        assert!(tool_names.contains(&"browser_get_active_element"));
        assert!(tool_names.contains(&"browser_check_element_clickable"));
        assert!(tool_names.contains(&"browser_dispatch_wheel_event"));
        assert!(tool_names.contains(&"browser_get_css_variables"));
        assert!(tool_names.contains(&"browser_clear_highlights"));
        assert!(tool_names.contains(&"browser_simulate_pointer_click"));
        assert!(tool_names.contains(&"browser_extract_meta_tags"));
        assert!(tool_names.contains(&"browser_get_document_geometry"));
        assert!(tool_names.contains(&"browser_set_device_orientation"));
        assert!(tool_names.contains(&"browser_clear_browser_cache"));
        assert!(tool_names.contains(&"browser_get_element_spacing"));
        assert!(tool_names.contains(&"browser_wait_for_url"));
        assert!(tool_names.contains(&"browser_dispatch_touch_event"));
        assert!(tool_names.contains(&"browser_get_a11y_tree"));
        assert!(tool_names.contains(&"browser_simulate_keyboard_chord"));
        assert!(tool_names.contains(&"browser_highlight_bounding_boxes"));
        assert!(tool_names.contains(&"browser_get_element_attributes"));
        assert!(tool_names.contains(&"browser_get_element_filter"));
        assert!(tool_names.contains(&"browser_dismiss_all_modals"));
        assert!(tool_names.contains(&"browser_get_color_palette"));
        assert!(tool_names.contains(&"browser_drag_and_drop_element"));
        assert!(tool_names.contains(&"browser_audit_forms"));
        assert!(tool_names.contains(&"browser_get_viewport_paging"));
        assert!(tool_names.contains(&"browser_get_computed_pseudo_element"));
        assert!(tool_names.contains(&"browser_simulate_clipboard_paste"));
        assert!(tool_names.contains(&"browser_smart_fill"));
        assert!(tool_names.contains(&"browser_observe_text"));
        assert!(tool_names.contains(&"browser_set_visibility_state"));
        assert!(tool_names.contains(&"browser_get_computed_font"));
        assert!(tool_names.contains(&"browser_get_action_map"));
        assert!(tool_names.contains(&"browser_get_motion_state"));
        assert!(tool_names.contains(&"browser_get_element_listeners"));
        assert!(tool_names.contains(&"browser_get_web_vitals"));
        assert!(tool_names.contains(&"browser_get_select_options"));
        assert!(tool_names.contains(&"browser_fuzzy_find_elements"));
        assert!(tool_names.contains(&"browser_get_heading_hierarchy"));
        assert!(tool_names.contains(&"browser_get_element_stacking"));
        assert!(tool_names.contains(&"browser_query_shadow_elements"));
        assert!(tool_names.contains(&"browser_get_element_visibility_matrix"));
        assert!(tool_names.contains(&"browser_get_error_digest"));
        assert!(tool_names.contains(&"browser_extract_grid_data"));
        assert!(tool_names.contains(&"browser_diff_form_inputs"));
        assert!(tool_names.contains(&"browser_unstick_scroll_locks"));
        assert!(tool_names.contains(&"browser_diff_dom_text"));
        assert!(tool_names.contains(&"browser_get_frame_hierarchy"));
        assert!(tool_names.contains(&"browser_get_role_elements"));
        assert!(tool_names.contains(&"browser_harvest_resources"));
        assert!(tool_names.contains(&"browser_detect_bot_challenges"));
        assert!(tool_names.contains(&"browser_get_scroll_state"));
        assert!(tool_names.contains(&"browser_get_proximity_elements"));
        assert!(tool_names.contains(&"browser_emulate_screen_metrics"));
        assert!(tool_names.contains(&"browser_export_table_csv"));
        assert!(tool_names.contains(&"browser_reset_form"));
        assert!(tool_names.contains(&"browser_get_datalist_options"));
        assert!(tool_names.contains(&"browser_get_canvas_context"));
        assert!(tool_names.contains(&"browser_control_media_playback"));
        assert!(tool_names.contains(&"browser_get_element_ancestry"));
        assert!(tool_names.contains(&"browser_get_layout_metrics"));
        assert!(tool_names.contains(&"browser_audit_contrast_ratio"));
        assert!(tool_names.contains(&"browser_dispatch_advanced_event"));
        assert!(tool_names.contains(&"browser_get_network_har"));
        assert!(tool_names.contains(&"browser_get_caret_position"));
        assert!(tool_names.contains(&"browser_dismiss_cookie_banners"));
        assert!(tool_names.contains(&"browser_emulate_speech_api"));
        assert!(tool_names.contains(&"browser_get_dom_mutations"));

        let snap = tools.iter().find(|t| t["name"] == "browser_snapshot").unwrap();
        assert!(snap["inputSchema"]["properties"]["filter_role"].is_object());

        let shot = tools.iter().find(|t| t["name"] == "browser_screenshot").unwrap();
        assert!(shot["inputSchema"]["properties"]["full_page"].is_object());

        let shot_el = tools.iter().find(|t| t["name"] == "browser_screenshot_element").unwrap();
        assert!(shot_el["inputSchema"]["properties"]["padding"].is_object());
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

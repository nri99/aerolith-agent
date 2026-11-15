use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tao::event_loop::EventLoopProxy;

use super::protocol::{AgentCommand, AgentResultEnvelope};
use crate::types::UserEvent;

pub struct AgentServer;

impl AgentServer {
    /// Starts the background HTTP server for AI agent control on 127.0.0.1:port
    pub fn start(port: u16, cdp_port: u16, proxy: EventLoopProxy<UserEvent>) -> Result<u16, std::io::Error> {
        let listener = TcpListener::bind(("127.0.0.1", port))?;
        let bound_port = listener.local_addr()?.port();
        let proxy = Arc::new(proxy);

        thread::Builder::new()
            .name("aerolith-agent-server".into())
            .spawn(move || {
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let proxy_clone = Arc::clone(&proxy);
                            thread::spawn(move || {
                                Self::handle_connection(stream, cdp_port, &proxy_clone);
                            });
                        }
                        Err(e) => {
                            eprintln!("[AerolithAgentServer] Accept error: {}", e);
                        }
                    }
                }
            })?;

        Ok(bound_port)
    }

    fn handle_connection(mut stream: TcpStream, cdp_port: u16, proxy: &EventLoopProxy<UserEvent>) {
        let _ = stream.set_read_timeout(Some(Duration::from_secs(15)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(15)));

        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() || request_line.is_empty() {
            return;
        }

        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            Self::send_response(&mut stream, 400, "text/plain", b"Bad Request");
            return;
        }

        let method = parts[0];
        let path = parts[1];

        // Parse headers
        let mut content_length: usize = 0;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                if let Some(val) = line.split(':').nth(1) {
                    content_length = val.trim().parse().unwrap_or(0);
                }
            }
        }

        // Read body if present
        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            let _ = reader.read_exact(&mut body);
        }

        // Handle Health / Status
        if path == "/health" || path == "/" {
            let status = serde_json::json!({
                "status": "ok",
                "browser": "aerolith-agent",
                "version": "0.2.0",
                "capabilities": [
                    "semantic_dom",
                    "set_of_marks",
                    "markdown_extractor",
                    "mcp_server",
                    "cdp_bridge",
                    "cdp_execute",
                    "storage_state_sync",
                    "element_finder",
                    "dialog_interceptor",
                    "anti_bot_stealth"
                ],
                "cdp_port": cdp_port,
                "cdp_url": format!("http://127.0.0.1:{}", cdp_port)
            });
            Self::send_json(&mut stream, 200, &status);
            return;
        }

        // Handle direct CDP Method Execution
        if method == "POST" && path == "/cdp/execute" {
            let json: serde_json::Value = match serde_json::from_slice(&body) {
                Ok(v) => v,
                Err(_) => {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                    return;
                }
            };
            let method_name = json["method"].as_str().unwrap_or("");
            if method_name.is_empty() {
                Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'method' in CDP execute request"));
                return;
            }
            let params = json.get("params").cloned();
            match Self::execute_cdp_method(cdp_port, method_name, params) {
                Ok(result) => {
                    Self::send_json(&mut stream, 200, &AgentResultEnvelope::ok(result));
                }
                Err(e) => {
                    Self::send_json(&mut stream, 502, &AgentResultEnvelope::err(e));
                }
            }
            return;
        }

        // Handle CDP Discovery & Bridge
        if path == "/cdp" {
            let cdp_info = serde_json::json!({
                "status": "cdp_ready",
                "cdp_port": cdp_port,
                "http_endpoint": format!("http://127.0.0.1:{}", cdp_port),
                "version_url": format!("http://127.0.0.1:{}/json/version", cdp_port),
                "list_url": format!("http://127.0.0.1:{}/json/list", cdp_port),
                "execute_endpoint": "http://127.0.0.1:8765/cdp/execute",
                "playwright_snippet": format!("browser = await playwright.chromium.connect_over_cdp('http://127.0.0.1:{}')", cdp_port),
                "puppeteer_snippet": format!("browser = await puppeteer.connect({{ browserURL: 'http://127.0.0.1:{}' }})", cdp_port)
            });
            Self::send_json(&mut stream, 200, &cdp_info);
            return;
        }

        if path == "/json/version" {
            if let Some(raw) = Self::fetch_cdp_proxy(cdp_port, "/json/version") {
                Self::send_response(&mut stream, 200, "application/json", &raw);
                return;
            }
            let fallback = serde_json::json!({
                "Browser": "Chrome/120.0.6099.130 (AerolithAgent/0.2.0)",
                "Protocol-Version": "1.3",
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                "V8-Version": "12.0",
                "WebKit-Version": "537.36",
                "webSocketDebuggerUrl": format!("ws://127.0.0.1:{}/devtools/browser/aerolith", cdp_port)
            });
            Self::send_json(&mut stream, 200, &fallback);
            return;
        }

        if path == "/json" || path == "/json/list" {
            if let Some(raw) = Self::fetch_cdp_proxy(cdp_port, "/json/list") {
                Self::send_response(&mut stream, 200, "application/json", &raw);
                return;
            }
            let fallback = serde_json::json!([
                {
                    "description": "",
                    "devtoolsFrontendUrl": format!("/devtools/inspector.html?ws=127.0.0.1:{}/devtools/page/1", cdp_port),
                    "id": "1",
                    "title": "Aerolith Agent Tab",
                    "type": "page",
                    "url": "about:home",
                    "webSocketDebuggerUrl": format!("ws://127.0.0.1:{}/devtools/page/1", cdp_port)
                }
            ]);
            Self::send_json(&mut stream, 200, &fallback);
            return;
        }

        // Route to AgentCommand
        let command = match (method, path) {
            ("GET", p) if p.starts_with("/snapshot") => {
                let viewport_only = p.contains("viewport_only=1") || p.contains("viewport_only=true") || p.contains("visible=1");
                let filter_role = if let Some(idx) = p.find("role=") {
                    p[idx + 5..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("filter_role=") {
                    p[idx + 12..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::Snapshot { viewport_only, filter_role })
            }
            ("GET", p) if p.starts_with("/highlight") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref' parameter"));
                    return;
                }
                Some(AgentCommand::Highlight { target })
            }
            ("POST", "/highlight") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"]
                    .as_str()
                    .or_else(|| json["ref"].as_str())
                    .or_else(|| json["ref"].as_u64().map(|n| {
                        Box::leak(n.to_string().into_boxed_str()) as &str
                    }))
                    .unwrap_or("")
                    .to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref' parameter"));
                    return;
                }
                Some(AgentCommand::Highlight { target })
            }
            ("GET", p) if p.starts_with("/find") => {
                let query = if let Some(idx) = p.find("q=") {
                    p[idx + 2..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("query=") {
                    p[idx + 6..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                Some(AgentCommand::Find { query })
            }
            ("POST", "/find") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let query = json["query"].as_str().unwrap_or("").to_string();
                Some(AgentCommand::Find { query })
            }
            ("GET", "/dialog") | ("GET", "/dialogs") => {
                Some(AgentCommand::DialogPolicy { alert: None, confirm: None, prompt_value: None })
            }
            ("POST", "/dialog/policy") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let alert = json["alert"].as_str().map(|s| s.to_string());
                let confirm = json["confirm"].as_str().map(|s| s.to_string());
                let prompt_value = json["prompt_value"].as_str().map(|s| s.to_string());
                Some(AgentCommand::DialogPolicy { alert, confirm, prompt_value })
            }
            ("GET", p) if p.starts_with("/markdown") => Some(AgentCommand::Markdown),
            ("GET", p) if p.starts_with("/screenshot/element") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref' parameter"));
                    return;
                }
                Some(AgentCommand::ScreenshotElement { target })
            }
            ("POST", "/screenshot/element") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"]
                    .as_str()
                    .or_else(|| json["ref"].as_str())
                    .or_else(|| json["ref"].as_u64().map(|n| {
                        Box::leak(n.to_string().into_boxed_str()) as &str
                    }))
                    .unwrap_or("")
                    .to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref' parameter"));
                    return;
                }
                Some(AgentCommand::ScreenshotElement { target })
            }
            ("GET", p) if p.starts_with("/screenshot") => {
                let som = p.contains("som=1") || p.contains("som=true");
                let full_page = p.contains("full_page=1") || p.contains("full_page=true") || p.contains("full=1") || p.contains("full=true");
                Some(AgentCommand::Screenshot { som, full_page })
            }
            ("POST", "/fill") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let fields_val = json.get("fields").cloned().unwrap_or(serde_json::json!([]));
                let fields: Vec<crate::agent::protocol::FormField> = serde_json::from_value(fields_val).unwrap_or_default();
                let submit = json["submit"].as_bool().unwrap_or(false);
                Some(AgentCommand::FillForm { fields, submit })
            }
            ("GET", p) if p.starts_with("/clipboard") => {
                Some(AgentCommand::Clipboard { action: "read".into(), text: None })
            }
            ("POST", "/clipboard") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let action = json["action"].as_str().unwrap_or("read").to_string();
                let text = json["text"].as_str().map(|s| s.to_string());
                Some(AgentCommand::Clipboard { action, text })
            }
            ("GET", "/tabs") => Some(AgentCommand::TabsList),
            ("GET", "/storage/state") => Some(AgentCommand::StorageState {
                action: "get".into(),
                state: None,
            }),

            ("POST", "/storage/state") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                Some(AgentCommand::StorageState {
                    action: "set".into(),
                    state: Some(json),
                })
            }

            ("POST", "/navigate") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let url = json["url"].as_str().unwrap_or("about:home").to_string();
                Some(AgentCommand::Navigate { url })
            }

            ("POST", "/click") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"]
                    .as_str()
                    .or_else(|| json["ref"].as_str())
                    .or_else(|| json["ref"].as_u64().map(|n| {
                        // allocate formatted string
                        Box::leak(n.to_string().into_boxed_str()) as &str
                    }))
                    .unwrap_or("")
                    .to_string();

                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::Click { target })
            }

            ("POST", "/type") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"]
                    .as_str()
                    .or_else(|| json["ref"].as_str())
                    .or_else(|| json["ref"].as_u64().map(|n| {
                        Box::leak(n.to_string().into_boxed_str()) as &str
                    }))
                    .unwrap_or("")
                    .to_string();

                let text = json["text"].as_str().unwrap_or("").to_string();
                let clear = json["clear"].as_bool().unwrap_or(false);
                let submit = json["submit"].as_bool().unwrap_or(false);

                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::Type { target, text, clear, submit })
            }

            ("POST", "/scroll") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let direction = json["direction"].as_str().unwrap_or("down").to_string();
                let amount = json["amount"].as_i64().unwrap_or(500) as i32;
                Some(AgentCommand::Scroll { direction, amount })
            }

            ("POST", "/wait") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"]
                    .as_str()
                    .or_else(|| json["ref"].as_str())
                    .or_else(|| json["ref"].as_u64().map(|n| {
                        Box::leak(n.to_string().into_boxed_str()) as &str
                    }))
                    .unwrap_or("")
                    .to_string();

                let timeout_ms = json["timeout_ms"].as_u64().unwrap_or(5000);
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::WaitFor { target, timeout_ms })
            }

            ("POST", "/eval") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let expression = json["expression"].as_str().unwrap_or("").to_string();
                Some(AgentCommand::Eval { expression })
            }

            ("POST", "/tabs/new") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let url = json["url"].as_str().unwrap_or("about:home").to_string();
                Some(AgentCommand::TabNew { url })
            }

            ("POST", "/tabs/switch") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let index = json["index"].as_u64().unwrap_or(0) as usize;
                Some(AgentCommand::TabSwitch { index })
            }

            ("POST", "/tabs/close") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let index = json["index"].as_u64().unwrap_or(0) as usize;
                Some(AgentCommand::TabClose { index })
            }

            _ => None,
        };

        let command = match command {
            Some(cmd) => cmd,
            None => {
                Self::send_json(&mut stream, 404, &AgentResultEnvelope::err(format!("Unknown endpoint: {} {}", method, path)));
                return;
            }
        };

        // Dispatch to GUI thread via EventLoopProxy
        let (tx, rx) = mpsc::channel();
        if proxy.send_event(UserEvent::AgentAction { command, response_tx: tx }).is_err() {
            Self::send_json(&mut stream, 500, &AgentResultEnvelope::err("Browser event loop unavailable"));
            return;
        }

        // Wait for response
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(result) => {
                Self::send_json(&mut stream, 200, &result);
            }
            Err(_) => {
                Self::send_json(&mut stream, 504, &AgentResultEnvelope::err("Action timed out"));
            }
        }
    }

    fn send_json<T: serde::Serialize>(stream: &mut TcpStream, status: u16, data: &T) {
        let json_bytes = match serde_json::to_vec_pretty(data) {
            Ok(b) => b,
            Err(_) => b"{\"error\":\"Failed to serialize JSON\"}".to_vec(),
        };
        Self::send_response(stream, status, "application/json", &json_bytes);
    }

    fn send_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) {
        let status_text = match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            504 => "Gateway Timeout",
            _ => "Status",
        };

        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
            status, status_text, content_type, body.len()
        );

        let _ = stream.write_all(response.as_bytes());
        let _ = stream.write_all(body);
        let _ = stream.flush();
    }

    fn fetch_cdp_proxy(cdp_port: u16, path: &str) -> Option<Vec<u8>> {
        let mut stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], cdp_port)),
            Duration::from_millis(500),
        ).ok()?;
        let _ = stream.set_read_timeout(Some(Duration::from_millis(1000)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(1000)));

        let req = format!("GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n", path, cdp_port);
        let _ = stream.write_all(req.as_bytes());
        let _ = stream.flush();

        let mut resp = Vec::new();
        let _ = stream.read_to_end(&mut resp);
        if resp.is_empty() {
            return None;
        }
        if let Some(pos) = resp.windows(4).position(|w| w == b"\r\n\r\n") {
            Some(resp[pos + 4..].to_vec())
        } else {
            Some(resp)
        }
    }

    pub fn execute_cdp_method(cdp_port: u16, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value, String> {
        let list_bytes = Self::fetch_cdp_proxy(cdp_port, "/json/list")
            .ok_or_else(|| "CDP endpoint unavailable or not responding".to_string())?;
        let targets: serde_json::Value = serde_json::from_slice(&list_bytes)
            .map_err(|e| format!("Invalid JSON from CDP /json/list: {}", e))?;
        
        let ws_url = targets.as_array()
            .and_then(|arr| arr.iter().find(|t| t["type"].as_str() == Some("page") || t["type"].as_str() == Some("webview")))
            .and_then(|t| t["webSocketDebuggerUrl"].as_str())
            .or_else(|| targets.as_array().and_then(|arr| arr.first()).and_then(|t| t["webSocketDebuggerUrl"].as_str()))
            .ok_or_else(|| "No active CDP page target found".to_string())?;

        let path = if let Some(idx) = ws_url.find("/devtools/") {
            &ws_url[idx..]
        } else {
            return Err("Unexpected WebSocketDebuggerUrl format".into());
        };

        let mut stream = TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], cdp_port)),
            Duration::from_millis(1000),
        ).map_err(|e| format!("Failed to connect to CDP port {}: {}", cdp_port, e))?;
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        let handshake = format!(
            "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n",
            path, cdp_port
        );
        stream.write_all(handshake.as_bytes()).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut reader = BufReader::new(stream);
        let mut response_headers = Vec::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).map_err(|e| e.to_string())?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
            response_headers.push(line);
        }

        if response_headers.is_empty() || !response_headers[0].contains("101") {
            return Err("CDP WebSocket upgrade handshake failed".into());
        }

        let cdp_req = serde_json::json!({
            "id": 1,
            "method": method,
            "params": params.unwrap_or_else(|| serde_json::json!({}))
        });
        let payload_bytes = serde_json::to_vec(&cdp_req).map_err(|e| e.to_string())?;

        let mut frame = Vec::new();
        frame.push(0x81);
        let len = payload_bytes.len();
        if len < 126 {
            frame.push(0x80 | (len as u8));
        } else if len <= 65535 {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        } else {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
        let mask = [0x12, 0x34, 0x56, 0x78];
        frame.extend_from_slice(&mask);
        for (i, b) in payload_bytes.iter().enumerate() {
            frame.push(b ^ mask[i % 4]);
        }

        let mut stream = reader.into_inner();
        stream.write_all(&frame).map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;

        let mut header = [0u8; 2];
        stream.read_exact(&mut header).map_err(|e| e.to_string())?;
        let is_masked = (header[1] & 0x80) != 0;
        let mut payload_len = (header[1] & 0x7F) as usize;
        if payload_len == 126 {
            let mut ext = [0u8; 2];
            stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
            payload_len = u16::from_be_bytes(ext) as usize;
        } else if payload_len == 127 {
            let mut ext = [0u8; 8];
            stream.read_exact(&mut ext).map_err(|e| e.to_string())?;
            payload_len = u64::from_be_bytes(ext) as usize;
        }

        let mask_key = if is_masked {
            let mut m = [0u8; 4];
            stream.read_exact(&mut m).map_err(|e| e.to_string())?;
            Some(m)
        } else {
            None
        };

        let mut body = vec![0u8; payload_len];
        stream.read_exact(&mut body).map_err(|e| e.to_string())?;
        if let Some(m) = mask_key {
            for (i, b) in body.iter_mut().enumerate() {
                *b ^= m[i % 4];
            }
        }

        let resp_json: serde_json::Value = serde_json::from_slice(&body)
            .map_err(|e| format!("Invalid JSON-RPC response from CDP: {}", e))?;
        Ok(resp_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_proxy_offline_returns_none() {
        let res = AgentServer::fetch_cdp_proxy(59999, "/json/version");
        assert!(res.is_none());
    }

    #[test]
    fn test_cdp_execute_offline_returns_err() {
        let res = AgentServer::execute_cdp_method(59999, "Page.navigate", None);
        assert!(res.is_err());
    }
}

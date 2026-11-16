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
                let padding = if let Some(idx) = p.find("padding=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(0)
                } else if let Some(idx) = p.find("pad=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(0)
                } else {
                    0
                };
                Some(AgentCommand::ScreenshotElement { target, padding })
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
                let padding = json["padding"].as_i64().or_else(|| json["pad"].as_i64()).unwrap_or(0) as i32;
                Some(AgentCommand::ScreenshotElement { target, padding })
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

            ("POST", "/emulate") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let width = json["width"].as_u64().unwrap_or(1280) as u32;
                let height = json["height"].as_u64().unwrap_or(800) as u32;
                let dpr = json["dpr"].as_f64();
                let mobile = json["mobile"].as_bool().unwrap_or(false);
                Some(AgentCommand::Emulate { width, height, dpr, mobile })
            }
            ("GET", p) if p.starts_with("/emulate") => {
                let width = if let Some(idx) = p.find("width=") {
                    p[idx + 6..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(1280)
                } else if let Some(idx) = p.find("w=") {
                    p[idx + 2..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(1280)
                } else {
                    1280
                };
                let height = if let Some(idx) = p.find("height=") {
                    p[idx + 7..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(800)
                } else if let Some(idx) = p.find("h=") {
                    p[idx + 2..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(800)
                } else {
                    800
                };
                let mobile = p.contains("mobile=1") || p.contains("mobile=true");
                Some(AgentCommand::Emulate { width, height, dpr: None, mobile })
            }
            ("GET", p) if p.starts_with("/console") => {
                let clear = p.contains("clear=1") || p.contains("clear=true");
                Some(AgentCommand::ConsoleLogs { clear })
            }
            ("POST", "/console/clear") => {
                Some(AgentCommand::ConsoleLogs { clear: true })
            }
            ("POST", "/pdf") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let landscape = json["landscape"].as_bool().unwrap_or(false);
                Some(AgentCommand::PrintPdf { landscape })
            }
            ("GET", p) if p.starts_with("/pdf") => {
                let landscape = p.contains("landscape=1") || p.contains("landscape=true");
                Some(AgentCommand::PrintPdf { landscape })
            }
            ("POST", "/hover") => {
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
                    .or_else(|| json["ref"].as_u64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                    .unwrap_or("")
                    .to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::Hover { target })
            }
            ("GET", p) if p.starts_with("/hover") => {
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
                Some(AgentCommand::Hover { target })
            }
            ("POST", "/key") | ("POST", "/press_key") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let key = json["key"].as_str().unwrap_or("").to_string();
                if key.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'key' parameter"));
                    return;
                }
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::PressKey { key, target })
            }
            ("POST", "/select") | ("POST", "/select_option") => {
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
                    .or_else(|| json["ref"].as_u64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                    .unwrap_or("")
                    .to_string();
                let value = json["value"].as_str().unwrap_or("").to_string();
                if target.is_empty() || value.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'value' parameter"));
                    return;
                }
                Some(AgentCommand::SelectOption { target, value })
            }
            ("GET", p) if p.starts_with("/network") => {
                let clear = p.contains("clear=1") || p.contains("clear=true");
                Some(AgentCommand::NetworkActivity { clear })
            }
            ("POST", "/network/clear") => {
                Some(AgentCommand::NetworkActivity { clear: true })
            }
            ("POST", "/drag") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let source = json["source"]
                    .as_str()
                    .or_else(|| json["from"].as_str())
                    .or_else(|| json["source"].as_u64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                    .unwrap_or("")
                    .to_string();
                if source.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'source' parameter"));
                    return;
                }
                let target = json["target"].as_str().or_else(|| json["to"].as_str()).map(|s| s.to_string());
                let offset_x = json["offset_x"].as_i64().or_else(|| json["dx"].as_i64()).map(|n| n as i32);
                let offset_y = json["offset_y"].as_i64().or_else(|| json["dy"].as_i64()).map(|n| n as i32);
                let steps = json["steps"].as_u64().unwrap_or(10) as u32;
                Some(AgentCommand::Drag { source, target, offset_x, offset_y, steps })
            }
            ("GET", "/cookies") => {
                Some(AgentCommand::Cookies { action: "get".into(), cookie: None, name: None, domain: None })
            }
            ("POST", "/cookies") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                Some(AgentCommand::Cookies { action: "set".into(), cookie: Some(json), name: None, domain: None })
            }
            ("POST", "/cookies/delete") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let name = json["name"].as_str().map(|s| s.to_string());
                let domain = json["domain"].as_str().map(|s| s.to_string());
                Some(AgentCommand::Cookies { action: "delete".into(), cookie: None, name, domain })
            }
            ("POST", "/cookies/clear") => {
                Some(AgentCommand::Cookies { action: "clear".into(), cookie: None, name: None, domain: None })
            }
            ("POST", "/headers") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let headers = if json.get("headers").is_some() { json["headers"].clone() } else { json };
                Some(AgentCommand::ExtraHeaders { headers })
            }
            ("POST", "/user_agent") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let user_agent = json["user_agent"].as_str().or_else(|| json["userAgent"].as_str()).unwrap_or("").to_string();
                if user_agent.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'user_agent' parameter"));
                    return;
                }
                let accept_language = json["accept_language"].as_str().or_else(|| json["acceptLanguage"].as_str()).map(|s| s.to_string());
                let platform = json["platform"].as_str().map(|s| s.to_string());
                Some(AgentCommand::UserAgentOverride { user_agent, accept_language, platform })
            }
            ("POST", "/upload") => {
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
                    .or_else(|| json["ref"].as_u64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str))
                    .unwrap_or("")
                    .to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let files: Vec<String> = if let Some(arr) = json["files"].as_array() {
                    arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
                } else if let Some(f) = json["file"].as_str().or_else(|| json["path"].as_str()) {
                    vec![f.to_string()]
                } else {
                    vec![]
                };
                Some(AgentCommand::UploadFile { target, files })
            }
            ("POST", "/location") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let latitude = json["latitude"].as_f64().or_else(|| json["lat"].as_f64()).unwrap_or(0.0);
                let longitude = json["longitude"].as_f64().or_else(|| json["lng"].as_f64()).unwrap_or(0.0);
                let accuracy = json["accuracy"].as_f64();
                let timezone = json["timezone"].as_str().map(|s| s.to_string());
                Some(AgentCommand::Location { latitude, longitude, accuracy, timezone })
            }
            ("GET", "/media") => {
                Some(AgentCommand::MediaControl { action: "status".into() })
            }
            ("POST", "/media") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let action = json["action"].as_str().unwrap_or("status").to_string();
                Some(AgentCommand::MediaControl { action })
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

            ("POST", "/back") | ("GET", "/back") => Some(AgentCommand::Back),
            ("POST", "/forward") | ("GET", "/forward") => Some(AgentCommand::Forward),
            ("POST", "/reload") | ("GET", "/reload") => Some(AgentCommand::Reload),

            ("GET", p) if p.starts_with("/text") => {
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
                let attribute = if let Some(idx) = p.find("attr=") {
                    p[idx + 5..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("attribute=") {
                    p[idx + 10..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::GetText { target, attribute })
            }

            ("POST", "/text") => {
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
                let attribute = json["attribute"].as_str().or_else(|| json["attr"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::GetText { target, attribute })
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

            ("GET", p) if p.starts_with("/inspect") => {
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
                Some(AgentCommand::Inspect { target })
            }

            ("POST", "/inspect") => {
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
                Some(AgentCommand::Inspect { target })
            }

            ("GET", "/route") | ("GET", "/routes") => {
                Some(AgentCommand::RouteRule {
                    action: "list".into(),
                    pattern: None,
                    rule_type: None,
                    status: None,
                    body: None,
                    headers: None,
                })
            }

            ("DELETE", "/route") | ("DELETE", "/routes") => {
                Some(AgentCommand::RouteRule {
                    action: "clear".into(),
                    pattern: None,
                    rule_type: None,
                    status: None,
                    body: None,
                    headers: None,
                })
            }

            ("POST", "/route") | ("POST", "/routes") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let action = json["action"].as_str().unwrap_or("add").to_string();
                let pattern = json["pattern"].as_str().map(|s| s.to_string());
                let rule_type = json["rule_type"].as_str().or_else(|| json["type"].as_str()).map(|s| s.to_string());
                let status = json["status"].as_u64().map(|n| n as u16);
                let body = json["body"].as_str().map(|s| s.to_string());
                let headers = json.get("headers").cloned();
                Some(AgentCommand::RouteRule { action, pattern, rule_type, status, body, headers })
            }

            ("POST", "/dblclick") | ("POST", "/double_click") => {
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
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::DoubleClick { target })
            }

            ("POST", "/right_click") | ("POST", "/contextmenu") => {
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
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' or 'ref'"));
                    return;
                }
                Some(AgentCommand::RightClick { target })
            }

            ("GET", p) if p.starts_with("/table") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::ExtractTable { target })
            }

            ("POST", "/table") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::ExtractTable { target })
            }

            ("GET", "/downloads") => {
                Some(AgentCommand::Downloads { clear: false })
            }

            ("POST", "/downloads/clear") => {
                Some(AgentCommand::Downloads { clear: true })
            }

            ("POST", "/network/emulate") | ("POST", "/throttle") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let profile = json["profile"].as_str().unwrap_or("online").to_string();
                let latency = json["latency"].as_f64().or_else(|| json["latency_ms"].as_f64());
                let download_throughput = json["download"].as_f64().or_else(|| json["download_throughput"].as_f64());
                let upload_throughput = json["upload"].as_f64().or_else(|| json["upload_throughput"].as_f64());
                let offline = json["offline"].as_bool();
                Some(AgentCommand::NetworkConditions {
                    profile,
                    latency,
                    download_throughput,
                    upload_throughput,
                    offline,
                })
            }

            ("POST", "/css") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let css = json["css"].as_str().unwrap_or("").to_string();
                let remove = json["remove"].as_bool().unwrap_or(false);
                Some(AgentCommand::InjectCss { css, remove })
            }

            ("POST", "/css/clear") | ("DELETE", "/css") => {
                Some(AgentCommand::InjectCss { css: String::new(), remove: true })
            }

            ("GET", p) if p.starts_with("/query") => {
                let selector = if let Some(idx) = p.find("selector=") {
                    p[idx + 9..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("sel=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                let attribute = if let Some(idx) = p.find("attribute=") {
                    p[idx + 10..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("attr=") {
                    p[idx + 5..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let max_items = if let Some(idx) = p.find("max=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(50)
                } else {
                    50
                };
                if selector.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'selector' query parameter"));
                    return;
                }
                Some(AgentCommand::QueryElements { selector, attribute, max_items })
            }

            ("POST", "/query") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let selector = json["selector"].as_str().or_else(|| json["sel"].as_str()).unwrap_or("").to_string();
                if selector.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'selector' in body"));
                    return;
                }
                let attribute = json["attribute"].as_str().or_else(|| json["attr"].as_str()).map(|s| s.to_string());
                let max_items = json["max_items"].as_u64().or_else(|| json["max"].as_u64()).unwrap_or(50) as usize;
                Some(AgentCommand::QueryElements { selector, attribute, max_items })
            }

            ("POST", "/focus") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target'"));
                    return;
                }
                Some(AgentCommand::Focus { target })
            }

            ("POST", "/blur") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target'"));
                    return;
                }
                Some(AgentCommand::Blur { target })
            }

            ("GET", "/page_info") | ("GET", "/info") => {
                Some(AgentCommand::PageInfo)
            }

            ("GET", p) if p.starts_with("/html") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("selector=") {
                    p[idx + 9..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::GetHtml { target })
            }
            ("POST", "/html") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().or_else(|| json["selector"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::GetHtml { target })
            }

            ("GET", p) if p.starts_with("/scroll_into_view") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let align = if let Some(idx) = p.find("align=") {
                    p[idx + 6..].split('&').next().unwrap_or("center").to_string()
                } else {
                    "center".to_string()
                };
                Some(AgentCommand::ScrollIntoView { target, align })
            }
            ("POST", "/scroll_into_view") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target'"));
                    return;
                }
                let align = json["align"].as_str().unwrap_or("center").to_string();
                Some(AgentCommand::ScrollIntoView { target, align })
            }

            ("GET", p) if p.starts_with("/zoom") => {
                let zoom_factor = if let Some(idx) = p.find("factor=") {
                    p[idx + 7..].split('&').next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0)
                } else if let Some(idx) = p.find("zoom=") {
                    p[idx + 5..].split('&').next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0)
                } else {
                    1.0
                };
                Some(AgentCommand::SetZoom { zoom_factor })
            }
            ("POST", "/zoom") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let zoom_factor = json["zoom_factor"]
                    .as_f64()
                    .or_else(|| json["factor"].as_f64())
                    .unwrap_or(1.0);
                Some(AgentCommand::SetZoom { zoom_factor })
            }

            ("GET", p) if p.starts_with("/storage/item") => {
                let storage_type = if let Some(idx) = p.find("type=") {
                    p[idx + 5..].split('&').next().unwrap_or("local").to_string()
                } else {
                    "local".to_string()
                };
                let action = if let Some(idx) = p.find("action=") {
                    p[idx + 7..].split('&').next().unwrap_or("get").to_string()
                } else {
                    "get".to_string()
                };
                let key = if let Some(idx) = p.find("key=") {
                    p[idx + 4..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let value = if let Some(idx) = p.find("value=") {
                    p[idx + 6..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::StorageItem { storage_type, action, key, value })
            }
            ("POST", "/storage/item") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let storage_type = json["storage_type"].as_str().or_else(|| json["type"].as_str()).unwrap_or("local").to_string();
                let action = json["action"].as_str().unwrap_or("get").to_string();
                let key = json["key"].as_str().map(|s| s.to_string());
                let value = json["value"].as_str().map(|s| s.to_string());
                Some(AgentCommand::StorageItem { storage_type, action, key, value })
            }

            ("POST", "/assert_element") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let assertions: crate::agent::protocol::ElementAssertion = serde_json::from_value(
                    json.get("assertions").cloned().unwrap_or(json.clone())
                ).unwrap_or_default();
                Some(AgentCommand::AssertElement { target, assertions })
            }

            ("POST", "/wait_for_selector") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let state = json["state"].as_str().unwrap_or("visible").to_string();
                let timeout_ms = json["timeout_ms"].as_u64().or_else(|| json["timeout"].as_u64()).unwrap_or(5000);
                Some(AgentCommand::WaitForSelector { target, state, timeout_ms })
            }

            ("GET", "/performance") | ("GET", "/metrics/performance") => {
                Some(AgentCommand::GetPerformanceMetrics)
            }

            ("POST", "/scroll_to") | ("POST", "/scroll/coords") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let x = json["x"].as_i64().or_else(|| json["left"].as_i64()).unwrap_or(0) as i32;
                let y = json["y"].as_i64().or_else(|| json["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = json["smooth"].as_bool().unwrap_or(false);
                Some(AgentCommand::ScrollToCoords { x, y, smooth })
            }

            ("GET", p) if p.starts_with("/outline") => {
                let max_links = if let Some(idx) = p.find("links=") {
                    p[idx + 6..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(25)
                } else {
                    25
                };
                let max_buttons = if let Some(idx) = p.find("buttons=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(25)
                } else {
                    25
                };
                Some(AgentCommand::GetPageOutline { max_links, max_buttons })
            }
            ("POST", "/outline") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let max_links = json["max_links"].as_u64().or_else(|| json["links"].as_u64()).unwrap_or(25) as usize;
                let max_buttons = json["max_buttons"].as_u64().or_else(|| json["buttons"].as_u64()).unwrap_or(25) as usize;
                Some(AgentCommand::GetPageOutline { max_links, max_buttons })
            }

            ("GET", p) if p.starts_with("/wait_for_idle") => {
                let idle_time_ms = if let Some(idx) = p.find("idle=") {
                    p[idx + 5..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(500)
                } else {
                    500
                };
                let timeout_ms = if let Some(idx) = p.find("timeout=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(10000)
                } else {
                    10000
                };
                let max_inflight = if let Some(idx) = p.find("max=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(0)
                } else {
                    0
                };
                Some(AgentCommand::WaitForNetworkIdle { idle_time_ms, timeout_ms, max_inflight })
            }
            ("POST", "/wait_for_idle") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let idle_time_ms = json["idle_time_ms"].as_u64().or_else(|| json["idle"].as_u64()).unwrap_or(500);
                let timeout_ms = json["timeout_ms"].as_u64().or_else(|| json["timeout"].as_u64()).unwrap_or(10000);
                let max_inflight = json["max_inflight"].as_u64().or_else(|| json["max"].as_u64()).unwrap_or(0) as usize;
                Some(AgentCommand::WaitForNetworkIdle { idle_time_ms, timeout_ms, max_inflight })
            }

            ("GET", p) if p.starts_with("/mutations") => {
                let action = if let Some(idx) = p.find("action=") {
                    p[idx + 7..].split('&').next().unwrap_or("get").to_string()
                } else {
                    "get".to_string()
                };
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::WatchMutations { action, target })
            }
            ("POST", "/mutations") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let action = json["action"].as_str().unwrap_or("get").to_string();
                let target = json["target"].as_str().map(|s| s.to_string());
                Some(AgentCommand::WatchMutations { action, target })
            }

            ("POST", "/tabs/eval") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let tab_index = json["tab_index"].as_u64().or_else(|| json["tab"].as_u64()).unwrap_or(0) as usize;
                let script = json["script"].as_str().or_else(|| json["expression"].as_str()).unwrap_or("").to_string();
                if script.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'script' parameter"));
                    return;
                }
                Some(AgentCommand::TabEval { tab_index, script })
            }

            ("GET", p) if p.starts_with("/network/body") => {
                let pattern = if let Some(idx) = p.find("pattern=") {
                    p[idx + 8..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("url=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                let clear = p.contains("clear=1") || p.contains("clear=true");
                Some(AgentCommand::GetResponseBody { pattern, clear })
            }
            ("POST", "/network/body") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let pattern = json["pattern"].as_str().or_else(|| json["url"].as_str()).unwrap_or("").to_string();
                let clear = json["clear"].as_bool().unwrap_or(false);
                Some(AgentCommand::GetResponseBody { pattern, clear })
            }

            ("POST", "/drop_file") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let file_name = json["file_name"].as_str().or_else(|| json["name"].as_str()).unwrap_or("upload.bin").to_string();
                let mime_type = json["mime_type"].as_str().or_else(|| json["type"].as_str()).unwrap_or("application/octet-stream").to_string();
                let content = json["content"].as_str().or_else(|| json["data"].as_str()).unwrap_or("").to_string();
                Some(AgentCommand::DropFile { target, file_name, mime_type, content })
            }

            ("GET", p) if p.starts_with("/dom/tree") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let max_depth = if let Some(idx) = p.find("depth=") {
                    p[idx + 6..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(4)
                } else {
                    4
                };
                Some(AgentCommand::GetDomTree { target, max_depth })
            }
            ("POST", "/dom/tree") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().map(|s| s.to_string());
                let max_depth = json["max_depth"].as_u64().or_else(|| json["depth"].as_u64()).unwrap_or(4) as usize;
                Some(AgentCommand::GetDomTree { target, max_depth })
            }

            ("GET", p) if p.starts_with("/network/offline") => {
                let offline = p.contains("offline=1") || p.contains("offline=true");
                Some(AgentCommand::SetOffline { offline })
            }
            ("POST", "/network/offline") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let offline = json["offline"].as_bool().unwrap_or(true);
                Some(AgentCommand::SetOffline { offline })
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

            ("GET", p) if p.starts_with("/wait_for_text") => {
                let text = if let Some(idx) = p.find("text=") {
                    p[idx + 5..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let timeout_ms = if let Some(idx) = p.find("timeout=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(5000)
                } else {
                    5000
                };
                Some(AgentCommand::WaitForText { target, text, timeout_ms })
            }
            ("POST", "/wait_for_text") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let text = json["text"].as_str().unwrap_or("").to_string();
                if text.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'text' parameter"));
                    return;
                }
                let target = json["target"].as_str().map(|s| s.to_string());
                let timeout_ms = json["timeout_ms"].as_u64().or_else(|| json["timeout"].as_u64()).unwrap_or(5000);
                Some(AgentCommand::WaitForText { target, text, timeout_ms })
            }

            ("GET", p) if p.starts_with("/computed_styles") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let properties = if let Some(idx) = p.find("props=") {
                    p[idx + 6..].split('&').next().unwrap_or("")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    Vec::new()
                };
                let properties = if properties.is_empty() {
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
                } else {
                    properties
                };
                Some(AgentCommand::GetComputedStyles { target, properties })
            }
            ("POST", "/computed_styles") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let properties = if let Some(arr) = json["properties"].as_array().or_else(|| json["props"].as_array()) {
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
                Some(AgentCommand::GetComputedStyles { target, properties })
            }

            ("GET", p) if p.starts_with("/wait_for_download") => {
                let file_pattern = if let Some(idx) = p.find("pattern=") {
                    p[idx + 8..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("file=") {
                    p[idx + 5..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let timeout_ms = if let Some(idx) = p.find("timeout=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(15000)
                } else {
                    15000
                };
                Some(AgentCommand::WaitForDownload { file_pattern, timeout_ms })
            }
            ("POST", "/wait_for_download") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let file_pattern = json["file_pattern"].as_str().or_else(|| json["pattern"].as_str()).map(|s| s.to_string());
                let timeout_ms = json["timeout_ms"].as_u64().or_else(|| json["timeout"].as_u64()).unwrap_or(15000);
                Some(AgentCommand::WaitForDownload { file_pattern, timeout_ms })
            }

            ("POST", "/set_content") => {
                let html = match serde_json::from_slice::<serde_json::Value>(&body) {
                    Ok(json) => json["html"].as_str().or_else(|| json["content"].as_str()).unwrap_or("").to_string(),
                    Err(_) => String::from_utf8_lossy(&body).to_string(),
                };
                if html.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing HTML content in body"));
                    return;
                }
                Some(AgentCommand::SetContent { html })
            }

            ("GET", p) if p.starts_with("/forms") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::GetForms { target })
            }
            ("POST", "/forms") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().map(|s| s.to_string());
                Some(AgentCommand::GetForms { target })
            }

            ("GET", p) if p.starts_with("/visibility") => {
                let state = if let Some(idx) = p.find("state=") {
                    p[idx + 6..].split('&').next().unwrap_or("visible").to_string()
                } else {
                    "visible".to_string()
                };
                Some(AgentCommand::SetVisibility { state })
            }
            ("POST", "/visibility") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let state = json["state"].as_str().unwrap_or("visible").to_string();
                Some(AgentCommand::SetVisibility { state })
            }

            ("GET", p) if p.starts_with("/emulate/media") => {
                let media_type = if let Some(idx) = p.find("media=") {
                    p[idx + 6..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let color_scheme = if let Some(idx) = p.find("scheme=") {
                    p[idx + 7..].split('&').next().map(|s| s.to_string())
                } else if let Some(idx) = p.find("color_scheme=") {
                    p[idx + 13..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::EmulateMedia { media_type, color_scheme })
            }
            ("POST", "/emulate/media") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let media_type = json["media_type"].as_str().or_else(|| json["media"].as_str()).map(|s| s.to_string());
                let color_scheme = json["color_scheme"].as_str().or_else(|| json["scheme"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::EmulateMedia { media_type, color_scheme })
            }

            ("GET", p) if p.starts_with("/element/bounds") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementBounds { target })
            }
            ("POST", "/element/bounds") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementBounds { target })
            }

            ("GET", p) if p.starts_with("/network/mock") => {
                let action = if let Some(idx) = p.find("action=") {
                    p[idx + 7..].split('&').next().unwrap_or("list").to_string()
                } else {
                    "list".to_string()
                };
                let pattern = if let Some(idx) = p.find("pattern=") {
                    p[idx + 8..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                let status = if let Some(idx) = p.find("status=") {
                    p[idx + 7..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(200)
                } else {
                    200
                };
                let content_type = if let Some(idx) = p.find("type=") {
                    p[idx + 5..].split('&').next().unwrap_or("application/json").to_string()
                } else {
                    "application/json".to_string()
                };
                let body_str = if let Some(idx) = p.find("body=") {
                    p[idx + 5..].split('&').next().map(|s| s.to_string())
                } else {
                    None
                };
                Some(AgentCommand::MockRoute {
                    action,
                    pattern,
                    status,
                    content_type,
                    body: body_str,
                })
            }
            ("POST", "/network/mock") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let action = json["action"].as_str().unwrap_or("set").to_string();
                let pattern = json["pattern"].as_str().or_else(|| json["url"].as_str()).map(|s| s.to_string());
                let status = json["status"].as_u64().unwrap_or(200) as u16;
                let content_type = json["content_type"].as_str().or_else(|| json["type"].as_str()).unwrap_or("application/json").to_string();
                let body_val = if let Some(b) = json.get("body") {
                    if let Some(s) = b.as_str() {
                        Some(s.to_string())
                    } else {
                        Some(b.to_string())
                    }
                } else {
                    None
                };
                Some(AgentCommand::MockRoute {
                    action,
                    pattern,
                    status,
                    content_type,
                    body: body_val,
                })
            }

            ("GET", "/viewport") => {
                Some(AgentCommand::GetViewport)
            }

            ("POST", "/element/attribute") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let attribute = json["attribute"].as_str().or_else(|| json["attr"].as_str()).unwrap_or("").to_string();
                if attribute.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'attribute' parameter"));
                    return;
                }
                let value = json["value"].as_str().or_else(|| json["val"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::SetElementAttribute { target, attribute, value })
            }

            ("GET", p) if p.starts_with("/search/text") => {
                let query = if let Some(idx) = p.find("q=") {
                    p[idx + 2..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("query=") {
                    p[idx + 6..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if query.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'q' or 'query' parameter"));
                    return;
                }
                let max_matches = if let Some(idx) = p.find("max=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(20)
                } else {
                    20
                };
                let case_sensitive = p.contains("case=1") || p.contains("case=true");
                Some(AgentCommand::SearchPageText { query, max_matches, case_sensitive })
            }
            ("POST", "/search/text") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let query = json["query"].as_str().or_else(|| json["q"].as_str()).unwrap_or("").to_string();
                if query.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'query' parameter"));
                    return;
                }
                let max_matches = json["max_matches"].as_u64().or_else(|| json["max"].as_u64()).unwrap_or(20) as usize;
                let case_sensitive = json["case_sensitive"].as_bool().unwrap_or(false);
                Some(AgentCommand::SearchPageText { query, max_matches, case_sensitive })
            }

            ("GET", p) if p.starts_with("/storage/dump") => {
                let storage_type = if let Some(idx) = p.find("type=") {
                    p[idx + 5..].split('&').next().unwrap_or("all").to_string()
                } else {
                    "all".to_string()
                };
                Some(AgentCommand::DumpStorage { storage_type })
            }
            ("POST", "/storage/dump") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let storage_type = json["storage_type"].as_str().or_else(|| json["type"].as_str()).unwrap_or("all").to_string();
                Some(AgentCommand::DumpStorage { storage_type })
            }

            ("GET", p) if p.starts_with("/human_jitter") => {
                let duration_ms = if let Some(idx) = p.find("duration=") {
                    p[idx + 9..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(800)
                } else {
                    800
                };
                let move_mouse = !p.contains("mouse=0") && !p.contains("mouse=false");
                let micro_scroll = !p.contains("scroll=0") && !p.contains("scroll=false");
                Some(AgentCommand::HumanJitter { duration_ms, move_mouse, micro_scroll })
            }
            ("POST", "/human_jitter") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let duration_ms = json["duration_ms"].as_u64().or_else(|| json["duration"].as_u64()).unwrap_or(800);
                let move_mouse = json["move_mouse"].as_bool().unwrap_or(true);
                let micro_scroll = json["micro_scroll"].as_bool().unwrap_or(true);
                Some(AgentCommand::HumanJitter { duration_ms, move_mouse, micro_scroll })
            }

            ("POST", "/type/keystrokes") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let text = json["text"].as_str().unwrap_or("").to_string();
                let delay_ms = json["delay_ms"].as_u64().or_else(|| json["delay"].as_u64()).unwrap_or(25);
                let clear = json["clear"].as_bool().unwrap_or(false);
                let submit = json["submit"].as_bool().unwrap_or(false);
                Some(AgentCommand::TypeKeystrokes { target, text, delay_ms, clear, submit })
            }

            ("GET", p) if p.starts_with("/canvas/data") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let mime_type = if let Some(idx) = p.find("mime=") {
                    p[idx + 5..].split('&').next().unwrap_or("image/png").to_string()
                } else if let Some(idx) = p.find("format=") {
                    let fmt = p[idx + 7..].split('&').next().unwrap_or("png");
                    if fmt == "jpeg" || fmt == "jpg" { "image/jpeg".to_string() } else if fmt == "webp" { "image/webp".to_string() } else { "image/png".to_string() }
                } else {
                    "image/png".to_string()
                };
                let quality = if let Some(idx) = p.find("quality=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(0.92)
                } else {
                    0.92
                };
                Some(AgentCommand::GetCanvasData { target, mime_type, quality })
            }
            ("POST", "/canvas/data") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let mime_type = json["mime_type"].as_str().or_else(|| json["format"].as_str()).unwrap_or("image/png").to_string();
                let quality = json["quality"].as_f64().unwrap_or(0.92);
                Some(AgentCommand::GetCanvasData { target, mime_type, quality })
            }

            ("GET", p) if p.starts_with("/performance/resources") => {
                let initiator_type = if let Some(idx) = p.find("initiator=") {
                    Some(p[idx + 10..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else if let Some(idx) = p.find("type=") {
                    Some(p[idx + 5..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else {
                    None
                };
                let max_entries = if let Some(idx) = p.find("max=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(50)
                } else {
                    50
                };
                let min_duration_ms = if let Some(idx) = p.find("min_duration=") {
                    p[idx + 13..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(0.0)
                } else {
                    0.0
                };
                Some(AgentCommand::GetResourceTiming { initiator_type, max_entries, min_duration_ms })
            }
            ("POST", "/performance/resources") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let initiator_type = json["initiator_type"].as_str().or_else(|| json["initiator"].as_str()).map(|s| s.to_string());
                let max_entries = json["max_entries"].as_u64().or_else(|| json["max"].as_u64()).unwrap_or(50) as usize;
                let min_duration_ms = json["min_duration_ms"].as_f64().or_else(|| json["min_duration"].as_f64()).unwrap_or(0.0);
                Some(AgentCommand::GetResourceTiming { initiator_type, max_entries, min_duration_ms })
            }

            ("GET", p) if p.starts_with("/dom/stats") => {
                Some(AgentCommand::GetDomStats)
            }
            ("POST", "/dom/stats") => {
                Some(AgentCommand::GetDomStats)
            }

            ("POST", "/scroll/by") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let dx = json["dx"].as_i64().or_else(|| json["left"].as_i64()).unwrap_or(0) as i32;
                let dy = json["dy"].as_i64().or_else(|| json["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = json["smooth"].as_bool().unwrap_or(false);
                Some(AgentCommand::ScrollBy { dx, dy, smooth })
            }

            ("POST", "/scroll/container") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let dx = json["dx"].as_i64().or_else(|| json["left"].as_i64()).unwrap_or(0) as i32;
                let dy = json["dy"].as_i64().or_else(|| json["top"].as_i64()).unwrap_or(0) as i32;
                let smooth = json["smooth"].as_bool().unwrap_or(false);
                Some(AgentCommand::ScrollContainer { target, dx, dy, smooth })
            }

            ("POST", "/element/select_text") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let start = json["start"].as_u64().map(|v| v as usize);
                let length = json["length"].as_u64().or_else(|| json["len"].as_u64()).map(|v| v as usize);
                Some(AgentCommand::SelectElementText { target, start, length })
            }

            ("GET", p) if p.starts_with("/links") => {
                let max_links = if let Some(idx) = p.find("max=") {
                    p[idx + 4..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(100)
                } else {
                    100
                };
                let external_only = p.contains("external=1") || p.contains("external=true");
                let filter_path = if let Some(idx) = p.find("filter=") {
                    Some(p[idx + 7..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else {
                    None
                };
                Some(AgentCommand::ExtractPageLinks { max_links, external_only, filter_path })
            }
            ("POST", "/links") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let max_links = json["max_links"].as_u64().or_else(|| json["max"].as_u64()).unwrap_or(100) as usize;
                let external_only = json["external_only"].as_bool().unwrap_or(false);
                let filter_path = json["filter_path"].as_str().or_else(|| json["filter"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::ExtractPageLinks { max_links, external_only, filter_path })
            }

            ("GET", p) if p.starts_with("/element/state") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementState { target })
            }
            ("POST", "/element/state") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementState { target })
            }

            ("POST", "/element/class") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let class_name = json["class_name"].as_str().or_else(|| json["class"].as_str()).unwrap_or("").to_string();
                if class_name.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'class_name' parameter"));
                    return;
                }
                let action = json["action"].as_str().unwrap_or("toggle").to_string();
                Some(AgentCommand::ToggleElementClass { target, class_name, action })
            }

            ("POST", "/element/text/set") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let text = json["text"].as_str().unwrap_or("").to_string();
                Some(AgentCommand::SetElementText { target, text })
            }

            ("POST", "/element/dispatch_event") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let event_type = json["event_type"].as_str().or_else(|| json["event"].as_str()).unwrap_or("").to_string();
                if event_type.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'event_type' parameter"));
                    return;
                }
                let bubbles = json["bubbles"].as_bool().unwrap_or(true);
                let cancelable = json["cancelable"].as_bool().unwrap_or(true);
                let detail = json.get("detail").cloned();
                Some(AgentCommand::DispatchCustomEvent { target, event_type, bubbles, cancelable, detail })
            }

            ("GET", "/active_element") | ("POST", "/active_element") => {
                Some(AgentCommand::GetActiveElement)
            }

            ("GET", p) if p.starts_with("/element/clickable") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::CheckElementClickable { target })
            }
            ("POST", "/element/clickable") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::CheckElementClickable { target })
            }

            ("POST", "/element/wheel") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let delta_x = json["delta_x"].as_f64().or_else(|| json["dx"].as_f64()).unwrap_or(0.0);
                let delta_y = json["delta_y"].as_f64().or_else(|| json["dy"].as_f64()).unwrap_or(0.0);
                let ctrl_key = json["ctrl_key"].as_bool().unwrap_or(false);
                Some(AgentCommand::DispatchWheelEvent { target, delta_x, delta_y, ctrl_key })
            }

            ("GET", p) if p.starts_with("/css/variables") => {
                let target = if let Some(idx) = p.find("target=") {
                    Some(p[idx + 7..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else {
                    None
                };
                let vars = if let Some(idx) = p.find("vars=") {
                    p[idx + 5..].split('&').next().unwrap_or("")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    Vec::new()
                };
                Some(AgentCommand::GetCssVariables { target, variable_names: vars })
            }
            ("POST", "/css/variables") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().map(|s| s.to_string());
                let vars = json["variables"].as_array()
                    .or_else(|| json["vars"].as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                Some(AgentCommand::GetCssVariables { target, variable_names: vars })
            }

            ("POST", "/highlights/clear") | ("GET", "/highlights/clear") => {
                Some(AgentCommand::ClearHighlights)
            }

            ("POST", "/element/pointer_click") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let click_count = json["click_count"].as_u64().or_else(|| json["count"].as_u64()).unwrap_or(1) as usize;
                let button = json["button"].as_i64().unwrap_or(0) as i32;
                Some(AgentCommand::SimulatePointerClick { target, click_count, button })
            }

            ("GET", p) if p.starts_with("/meta/tags") => {
                let include_json_ld = !p.contains("json_ld=false") && !p.contains("json_ld=0");
                Some(AgentCommand::ExtractMetaTags { include_json_ld })
            }
            ("POST", "/meta/tags") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let include_json_ld = json["include_json_ld"].as_bool().unwrap_or(true);
                Some(AgentCommand::ExtractMetaTags { include_json_ld })
            }

            ("GET", "/document/geometry") | ("POST", "/document/geometry") | ("GET", "/geometry") => {
                Some(AgentCommand::GetDocumentGeometry)
            }

            ("POST", "/emulate/orientation") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let alpha = json["alpha"].as_f64().unwrap_or(0.0);
                let beta = json["beta"].as_f64().unwrap_or(0.0);
                let gamma = json["gamma"].as_f64().unwrap_or(0.0);
                let absolute = json["absolute"].as_bool().unwrap_or(false);
                Some(AgentCommand::SetDeviceOrientation { alpha, beta, gamma, absolute })
            }

            ("POST", "/cache/clear") | ("POST", "/clear_cache") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let clear_storage = json["clear_storage"].as_bool().or_else(|| json["storage"].as_bool()).unwrap_or(false);
                let clear_cookies = json["clear_cookies"].as_bool().or_else(|| json["cookies"].as_bool()).unwrap_or(false);
                Some(AgentCommand::ClearBrowserCache { clear_storage, clear_cookies })
            }

            ("GET", p) if p.starts_with("/element/spacing") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementSpacing { target })
            }
            ("POST", "/element/spacing") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementSpacing { target })
            }

            ("GET", p) if p.starts_with("/wait_for_url") => {
                let pattern = if let Some(idx) = p.find("pattern=") {
                    p[idx + 8..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("url=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if pattern.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'pattern' parameter"));
                    return;
                }
                let timeout_ms = if let Some(idx) = p.find("timeout=") {
                    p[idx + 8..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(5000)
                } else {
                    5000
                };
                Some(AgentCommand::WaitForUrl { pattern, timeout_ms })
            }
            ("POST", "/wait_for_url") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let pattern = json["pattern"].as_str().or_else(|| json["url"].as_str()).unwrap_or("").to_string();
                if pattern.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'pattern' parameter"));
                    return;
                }
                let timeout_ms = json["timeout_ms"].as_u64().or_else(|| json["timeout"].as_u64()).unwrap_or(5000);
                Some(AgentCommand::WaitForUrl { pattern, timeout_ms })
            }

            ("POST", "/touch") | ("POST", "/element/touch") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let event_type = json["event_type"].as_str().or_else(|| json["event"].as_str()).unwrap_or("touchstart").to_string();
                let touches: Vec<crate::agent::protocol::TouchPoint> = json["touches"].as_array()
                    .map(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
                    .unwrap_or_default();
                Some(AgentCommand::DispatchTouchEvent { event_type, touches })
            }

            ("GET", p) if p.starts_with("/a11y") => {
                let target = if let Some(idx) = p.find("target=") {
                    Some(p[idx + 7..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else if let Some(idx) = p.find("ref=") {
                    Some(p[idx + 4..].split('&').next().unwrap_or("").to_string()).filter(|s| !s.is_empty())
                } else {
                    None
                };
                let max_depth = if let Some(idx) = p.find("depth=") {
                    p[idx + 6..].split('&').next().and_then(|s| s.parse().ok()).unwrap_or(5)
                } else {
                    5
                };
                Some(AgentCommand::GetA11yTree { target, max_depth })
            }
            ("POST", "/a11y") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                let max_depth = json["max_depth"].as_u64().or_else(|| json["depth"].as_u64()).unwrap_or(5) as usize;
                Some(AgentCommand::GetA11yTree { target, max_depth })
            }

            ("POST", "/keyboard/chord") | ("POST", "/chord") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let chord = json["chord"].as_str().or_else(|| json["keys"].as_str()).unwrap_or("").to_string();
                if chord.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'chord' parameter"));
                    return;
                }
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::SimulateKeyboardChord { chord, target })
            }

            ("POST", "/highlights/boxes") | ("POST", "/boxes") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let boxes: Vec<crate::agent::protocol::BoundingBoxMarker> = json["boxes"].as_array()
                    .map(|arr| serde_json::from_value(serde_json::Value::Array(arr.clone())).unwrap_or_default())
                    .unwrap_or_default();
                Some(AgentCommand::HighlightBoundingBoxes { boxes })
            }

            ("GET", p) if p.starts_with("/element/attributes") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementAttributes { target })
            }
            ("POST", "/element/attributes") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementAttributes { target })
            }

            ("GET", p) if p.starts_with("/element/filter") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementFilter { target })
            }
            ("POST", "/element/filter") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                Some(AgentCommand::GetElementFilter { target })
            }

            ("POST", "/modals/dismiss") | ("POST", "/dismiss_modals") => {
                Some(AgentCommand::DismissAllModals)
            }

            ("GET", p) if p.starts_with("/palette") => {
                let limit = if let Some(idx) = p.find("limit=") {
                    p[idx + 6..].split('&').next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(15)
                } else {
                    15
                };
                Some(AgentCommand::GetColorPalette { limit })
            }
            ("POST", "/palette") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
                let limit = json["limit"].as_u64().map(|n| n as usize).unwrap_or(15);
                Some(AgentCommand::GetColorPalette { limit })
            }

            ("POST", "/drag_and_drop") | ("POST", "/element/drag_and_drop") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let source = json["source"].as_str().or_else(|| json["from"].as_str()).unwrap_or("").to_string();
                let target = json["target"].as_str().or_else(|| json["to"].as_str()).unwrap_or("").to_string();
                if source.is_empty() || target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'source' or 'target' parameter"));
                    return;
                }
                let steps = json["steps"].as_u64().map(|s| s as u32).unwrap_or(10);
                let hold_ms = json["hold_ms"].as_u64().unwrap_or(0);
                Some(AgentCommand::DragAndDropElement { source, target, steps, hold_ms })
            }

            ("GET", p) if p.starts_with("/forms/audit") => {
                let target = if let Some(idx) = p.find("target=") {
                    let t = p[idx + 7..].split('&').next().unwrap_or("").to_string();
                    if t.is_empty() { None } else { Some(t) }
                } else {
                    None
                };
                Some(AgentCommand::AuditForms { target })
            }
            ("POST", "/forms/audit") => {
                let json: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::AuditForms { target })
            }

            ("GET", "/viewport/paging") | ("POST", "/viewport/paging") => {
                Some(AgentCommand::GetViewportPaging)
            }

            ("GET", p) if p.starts_with("/element/pseudo") => {
                let target = if let Some(idx) = p.find("target=") {
                    p[idx + 7..].split('&').next().unwrap_or("").to_string()
                } else if let Some(idx) = p.find("ref=") {
                    p[idx + 4..].split('&').next().unwrap_or("").to_string()
                } else {
                    String::new()
                };
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let pseudo = if let Some(idx) = p.find("pseudo=") {
                    p[idx + 7..].split('&').next().unwrap_or("::before").to_string()
                } else {
                    "::before".to_string()
                };
                let props: Vec<String> = if let Some(idx) = p.find("props=") {
                    p[idx + 6..].split('&').next().unwrap_or("")
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    vec![]
                };
                Some(AgentCommand::GetComputedPseudoElement { target, pseudo, properties: props })
            }
            ("POST", "/element/pseudo") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).unwrap_or("").to_string();
                if target.is_empty() {
                    Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Missing 'target' parameter"));
                    return;
                }
                let pseudo = json["pseudo"].as_str().unwrap_or("::before").to_string();
                let props: Vec<String> = json["properties"].as_array()
                    .or_else(|| json["props"].as_array())
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                Some(AgentCommand::GetComputedPseudoElement { target, pseudo, properties: props })
            }

            ("POST", "/clipboard/paste") => {
                let json: serde_json::Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => {
                        Self::send_json(&mut stream, 400, &AgentResultEnvelope::err("Invalid JSON body"));
                        return;
                    }
                };
                let text = json["text"].as_str().unwrap_or("").to_string();
                let target = json["target"].as_str().or_else(|| json["ref"].as_str()).map(|s| s.to_string());
                Some(AgentCommand::SimulateClipboardPaste { target, text })
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

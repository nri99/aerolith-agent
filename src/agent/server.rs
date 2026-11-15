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
    pub fn start(port: u16, proxy: EventLoopProxy<UserEvent>) -> Result<u16, std::io::Error> {
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
                                Self::handle_connection(stream, &proxy_clone);
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

    fn handle_connection(mut stream: TcpStream, proxy: &EventLoopProxy<UserEvent>) {
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
                    "anti_bot_stealth"
                ]
            });
            Self::send_json(&mut stream, 200, &status);
            return;
        }

        // Route to AgentCommand
        let command = match (method, path) {
            ("GET", p) if p.starts_with("/snapshot") => Some(AgentCommand::Snapshot),
            ("GET", p) if p.starts_with("/markdown") => Some(AgentCommand::Markdown),
            ("GET", p) if p.starts_with("/screenshot") => {
                let som = p.contains("som=1") || p.contains("som=true");
                Some(AgentCommand::Screenshot { som })
            }
            ("GET", "/tabs") => Some(AgentCommand::TabsList),

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
}

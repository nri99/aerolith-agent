use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct AgentCli;

impl AgentCli {
    /// Attempts to execute an agent CLI command against a running aerolith-agent server.
    /// Returns true if the command was recognized and handled as a CLI client command.
    pub fn handle_client_subcommand(args: &[String]) -> bool {
        if args.len() < 2 {
            return false;
        }

        let cmd = args[1].to_lowercase();
        let port = Self::extract_port(args).unwrap_or(8765);

        match cmd.as_str() {
            "nav" | "navigate" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent nav <url> [--port <port>]");
                    std::process::exit(1);
                }
                let url = &args[2];
                let payload = serde_json::json!({ "url": url });
                let resp = Self::send_post(port, "/navigate", &payload);
                println!("{}", resp);
                true
            }

            "snapshot" => {
                let viewport_only = args.iter().any(|a| a == "--viewport-only" || a == "-v");
                let role = args.iter()
                    .position(|a| a == "--role" || a == "-r")
                    .and_then(|idx| args.get(idx + 1));

                let mut params = Vec::new();
                if viewport_only {
                    params.push("viewport_only=1".to_string());
                }
                if let Some(r) = role {
                    params.push(format!("role={}", crate::url_utils::urlencoding_simple(r)));
                }

                let endpoint = if params.is_empty() {
                    "/snapshot".to_string()
                } else {
                    format!("/snapshot?{}", params.join("&"))
                };
                let resp = Self::send_get(port, &endpoint);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(tree) = json["data"]["tree_text"].as_str() {
                        let filter_label = role.map(|r| format!(" [ROLE: {}]", r)).unwrap_or_default();
                        println!("=== SEMANTIC ACCESSIBILITY SNAPSHOT{}{filter_label} ===", if viewport_only { " (VIEWPORT ONLY)" } else { "" });
                        println!("URL: {}", json["data"]["url"].as_str().unwrap_or(""));
                        println!("Title: {}", json["data"]["title"].as_str().unwrap_or(""));
                        println!("---------------------------------------");
                        println!("{}", tree);
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "highlight" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent highlight <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/highlight", &payload);
                println!("{}", resp);
                true
            }

            "click" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent click <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/click", &payload);
                println!("{}", resp);
                true
            }

            "type" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent type <ref_or_selector> <text> [--clear] [--enter] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let text = &args[3];
                let clear = args.iter().any(|a| a == "--clear");
                let submit = args.iter().any(|a| a == "--enter" || a == "--submit");

                let payload = serde_json::json!({
                    "target": target,
                    "text": text,
                    "clear": clear,
                    "submit": submit
                });
                let resp = Self::send_post(port, "/type", &payload);
                println!("{}", resp);
                true
            }

            "markdown" | "md" => {
                let resp = Self::send_get(port, "/markdown");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(md) = json["data"]["markdown"].as_str() {
                        println!("{}", md);
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "screenshot" => {
                let som = args.iter().any(|a| a == "--som");
                let full_page = args.iter().any(|a| a == "--full-page" || a == "-f");
                let path = args.iter().find(|a| a.ends_with(".png")).cloned();

                let mut params = Vec::new();
                if som { params.push("som=1"); }
                if full_page { params.push("full_page=1"); }

                let endpoint = if params.is_empty() {
                    "/screenshot".to_string()
                } else {
                    format!("/screenshot?{}", params.join("&"))
                };
                let resp = Self::send_get(port, &endpoint);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(b64) = json["data"]["screenshot_base64"].as_str() {
                        if let Some(out_path) = path {
                            // Decode base64
                            let bytes = Self::base64_decode(b64);
                            if let Ok(b) = bytes {
                                if std::fs::write(&out_path, b).is_ok() {
                                    println!("Screenshot saved to {}", out_path);
                                    return true;
                                }
                            }
                        }
                    }
                }
                println!("{}", resp);
                true
            }

            "scroll" => {
                let direction = args.get(2).map(|s| s.as_str()).unwrap_or("down");
                let amount = args.get(3).and_then(|s| s.parse::<i32>().ok()).unwrap_or(500);
                let payload = serde_json::json!({
                    "direction": direction,
                    "amount": amount
                });
                let resp = Self::send_post(port, "/scroll", &payload);
                println!("{}", resp);
                true
            }

            "wait" | "wait_for" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent wait <ref_or_selector> [--timeout <ms>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let timeout_ms = args.iter()
                    .position(|a| a == "--timeout" || a == "-t")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|t| t.parse::<u64>().ok())
                    .unwrap_or(5000);

                let payload = serde_json::json!({
                    "target": target,
                    "timeout_ms": timeout_ms
                });
                let resp = Self::send_post(port, "/wait", &payload);
                println!("{}", resp);
                true
            }

            "tabs" => {
                let resp = Self::send_get(port, "/tabs");
                println!("{}", resp);
                true
            }

            "storage" => {
                let is_export = args.iter().any(|a| a == "--export" || a == "export" || a == "-e");
                let is_import = args.iter().any(|a| a == "--import" || a == "import" || a == "-i");

                if is_import {
                    let input_arg = args.iter()
                        .position(|a| a == "--import" || a == "import" || a == "-i")
                        .and_then(|idx| args.get(idx + 1));

                    let raw_json = match input_arg {
                        Some(path_or_str) => {
                            if std::path::Path::new(path_or_str).exists() {
                                std::fs::read_to_string(path_or_str).unwrap_or_else(|_| path_or_str.to_string())
                            } else {
                                path_or_str.to_string()
                            }
                        }
                        None => {
                            eprintln!("Usage: aerolith-agent storage --import <file_or_json> [--port <port>]");
                            std::process::exit(1);
                        }
                    };

                    let val: serde_json::Value = match serde_json::from_str(&raw_json) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Invalid storage state JSON: {}", e);
                            std::process::exit(1);
                        }
                    };

                    let resp = Self::send_post(port, "/storage/state", &val);
                    println!("{}", resp);
                    return true;
                }

                if is_export || args.len() == 2 {
                    let resp = Self::send_get(port, "/storage/state");
                    let out_file = args.iter()
                        .position(|a| a == "--out" || a == "-o")
                        .and_then(|idx| args.get(idx + 1));

                    if let Some(path) = out_file {
                        if std::fs::write(path, &resp).is_ok() {
                            println!("Storage state exported to {}", path);
                            return true;
                        }
                    }
                    println!("{}", resp);
                    return true;
                }

                eprintln!("Usage: aerolith-agent storage [--export [--out <file>]] [--import <file_or_json>] [--port <port>]");
                std::process::exit(1);
            }

            "cdp" => {
                let resp = Self::send_get(port, "/cdp");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    println!("=== CHROME DEVTOOLS PROTOCOL (CDP) BRIDGE ===");
                    println!("CDP Endpoint: {}", json["http_endpoint"].as_str().unwrap_or(""));
                    println!("Version URL:  {}", json["version_url"].as_str().unwrap_or(""));
                    println!("List URL:     {}", json["list_url"].as_str().unwrap_or(""));
                    println!("----------------------------------------------");
                    println!("Playwright Connect:\n  {}", json["playwright_snippet"].as_str().unwrap_or(""));
                    println!("Puppeteer Connect:\n  {}", json["puppeteer_snippet"].as_str().unwrap_or(""));
                    return true;
                }
                println!("{}", resp);
                true
            }

            "find" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent find <query> [--port <port>]");
                    std::process::exit(1);
                }
                let query = &args[2];
                let endpoint = format!("/find?q={}", crate::url_utils::urlencoding_simple(query));
                let resp = Self::send_get(port, &endpoint);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(matches) = json["data"]["matches"].as_array() {
                        println!("Found {} matches for \"{}\":", matches.len(), query);
                        for m in matches {
                            let r = m["ref"].as_u64().unwrap_or(0);
                            let tag = m["tag"].as_str().unwrap_or("");
                            let text = m["text"].as_str().unwrap_or("");
                            let b = &m["bounds"];
                            let cx = b["center_x"].as_i64().unwrap_or(0);
                            let cy = b["center_y"].as_i64().unwrap_or(0);
                            let inv = b["in_viewport"].as_bool().unwrap_or(false);
                            println!("  [@{}] <{}> \"{}\" center={},{} in_viewport={}", r, tag, text, cx, cy, inv);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "profile" => {
                let is_clear = args.iter().any(|a| a == "--clear");
                if is_clear {
                    let name = args.iter()
                        .position(|a| a == "--clear")
                        .and_then(|idx| args.get(idx + 1));
                    match name {
                        Some(n) => {
                            match crate::paths::clear_profile(n) {
                                Ok(_) => println!("Profile '{}' cleared successfully.", n),
                                Err(e) => eprintln!("Failed to clear profile '{}': {}", n, e),
                            }
                            return true;
                        }
                        None => {
                            eprintln!("Usage: aerolith-agent profile --clear <name>");
                            std::process::exit(1);
                        }
                    }
                }

                let profiles = crate::paths::list_profiles();
                println!("=== AEROLITH PROFILES ({}) ===", profiles.len());
                if profiles.is_empty() {
                    println!("(No named profiles created yet)");
                } else {
                    for p in profiles {
                        println!("  - {}", p);
                    }
                }
                true
            }

            "dialog" => {
                let alert = args.iter().position(|a| a == "--alert").and_then(|idx| args.get(idx + 1));
                let confirm = args.iter().position(|a| a == "--confirm").and_then(|idx| args.get(idx + 1));
                let prompt_val = args.iter().position(|a| a == "--prompt").and_then(|idx| args.get(idx + 1));

                if alert.is_some() || confirm.is_some() || prompt_val.is_some() {
                    let payload = serde_json::json!({
                        "alert": alert,
                        "confirm": confirm,
                        "prompt_value": prompt_val
                    });
                    let resp = Self::send_post(port, "/dialog/policy", &payload);
                    println!("{}", resp);
                } else {
                    let resp = Self::send_get(port, "/dialog");
                    println!("{}", resp);
                }
                true
            }

            "cdp-eval" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent cdp-eval <method> [params_json] [--port <port>]");
                    std::process::exit(1);
                }
                let method = &args[2];
                let params = args.get(3).and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
                let payload = serde_json::json!({
                    "method": method,
                    "params": params
                });
                let resp = Self::send_post(port, "/cdp/execute", &payload);
                println!("{}", resp);
                true
            }

            _ => false,
        }
    }

    fn extract_port(args: &[String]) -> Option<u16> {
        for i in 0..args.len() {
            if args[i] == "--port" || args[i] == "-p" {
                if let Some(p) = args.get(i + 1) {
                    return p.parse().ok();
                }
            }
            if args[i].starts_with("--port=") {
                return args[i].trim_start_matches("--port=").parse().ok();
            }
        }
        None
    }

    fn send_get(port: u16, path: &str) -> String {
        Self::send_http(port, "GET", path, None)
    }

    fn send_post(port: u16, path: &str, body: &serde_json::Value) -> String {
        let json_str = serde_json::to_string(body).unwrap_or_default();
        Self::send_http(port, "POST", path, Some(&json_str))
    }

    fn send_http(port: u16, method: &str, path: &str, body: Option<&str>) -> String {
        let mut stream = match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[AerolithAgent] Cannot connect to browser server on port {}: {}", port, e);
                eprintln!("[AerolithAgent] Is the browser running with agent mode? Start it with: aerolith-agent serve");
                std::process::exit(1);
            }
        };

        let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));

        let content_len = body.map(|b| b.len()).unwrap_or(0);
        let req = format!(
            "{} {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            method, path, port, content_len
        );

        let _ = stream.write_all(req.as_bytes());
        if let Some(b) = body {
            let _ = stream.write_all(b.as_bytes());
        }
        let _ = stream.flush();

        let mut reader = BufReader::new(stream);
        let mut status_line = String::new();
        let _ = reader.read_line(&mut status_line);

        // Read headers until empty line
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
                break;
            }
        }

        // Read body
        let mut resp_body = String::new();
        let _ = reader.read_to_string(&mut resp_body);
        resp_body
    }

    fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
        const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut inv = [255u8; 256];
        for (i, &b) in TABLE.iter().enumerate() {
            inv[b as usize] = i as u8;
        }

        let clean: Vec<u8> = input.bytes().filter(|&b| b != b'=' && b != b'\r' && b != b'\n' && b != b' ').collect();
        let mut out = Vec::new();

        for chunk in clean.chunks(4) {
            let b0 = inv[chunk[0] as usize] as u32;
            let b1 = if chunk.len() > 1 { inv[chunk[1] as usize] as u32 } else { 0 };
            let b2 = if chunk.len() > 2 { inv[chunk[2] as usize] as u32 } else { 0 };
            let b3 = if chunk.len() > 3 { inv[chunk[3] as usize] as u32 } else { 0 };

            let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;

            out.push(((triple >> 16) & 0xFF) as u8);
            if chunk.len() > 2 {
                out.push(((triple >> 8) & 0xFF) as u8);
            }
            if chunk.len() > 3 {
                out.push((triple & 0xFF) as u8);
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_port() {
        let args1 = vec!["aerolith-agent".into(), "nav".into(), "https://example.com".into(), "--port".into(), "9000".into()];
        assert_eq!(AgentCli::extract_port(&args1), Some(9000));

        let args2 = vec!["aerolith-agent".into(), "snapshot".into(), "--port=8888".into()];
        assert_eq!(AgentCli::extract_port(&args2), Some(8888));

        let args3 = vec!["aerolith-agent".into(), "snapshot".into()];
        assert_eq!(AgentCli::extract_port(&args3), None);
    }

    #[test]
    fn test_base64_decode() {
        // "Hello World" in base64 is "SGVsbG8gV29ybGQ="
        let decoded = AgentCli::base64_decode("SGVsbG8gV29ybGQ=").unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");
    }

    #[test]
    fn test_handle_client_subcommand_non_client() {
        let args = vec!["aerolith-agent".into()];
        assert!(!AgentCli::handle_client_subcommand(&args));

        let args_unknown = vec!["aerolith-agent".into(), "unknown_action".into()];
        assert!(!AgentCli::handle_client_subcommand(&args_unknown));
    }
}

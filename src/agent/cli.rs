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

            "back" => {
                let resp = Self::send_post(port, "/back", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "forward" | "fwd" => {
                let resp = Self::send_post(port, "/forward", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "reload" | "refresh" => {
                let resp = Self::send_post(port, "/reload", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "text" | "get-text" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent text <ref_or_selector> [--attr <attribute>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let attr = args.iter()
                    .position(|a| a == "--attr" || a == "-a" || a == "--attribute")
                    .and_then(|idx| args.get(idx + 1));

                let mut params = vec![format!("target={}", crate::url_utils::urlencoding_simple(target))];
                if let Some(a) = attr {
                    params.push(format!("attr={}", crate::url_utils::urlencoding_simple(a)));
                }

                let endpoint = format!("/text?{}", params.join("&"));
                let resp = Self::send_get(port, &endpoint);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(txt) = json["data"]["text"].as_str() {
                        println!("{}", txt);
                        return true;
                    }
                }
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

            "screenshot-element" | "crop" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent screenshot-element <ref_or_selector> [out.png] [--padding <px>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let path = args.iter().skip(3).find(|a| a.ends_with(".png")).cloned();
                let mut padding = 0i32;
                let mut i = 3;
                while i < args.len() {
                    if (args[i] == "--padding" || args[i] == "--pad") && i + 1 < args.len() {
                        padding = args[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let endpoint = format!("/screenshot/element?target={}&padding={}", crate::url_utils::urlencoding_simple(target), padding);
                let resp = Self::send_get(port, &endpoint);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(b64) = json["data"]["screenshot_base64"].as_str() {
                        if let Some(out_path) = path {
                            let bytes = Self::base64_decode(b64);
                            if let Ok(b) = bytes {
                                if std::fs::write(&out_path, b).is_ok() {
                                    println!("Element screenshot saved to {}", out_path);
                                    return true;
                                }
                            }
                        }
                    }
                }
                println!("{}", resp);
                true
            }

            "fill" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent fill <json_or_file> [--submit] [--port <port>]");
                    std::process::exit(1);
                }
                let input_arg = &args[2];
                let raw_json = if std::path::Path::new(input_arg).exists() {
                    std::fs::read_to_string(input_arg).unwrap_or_else(|_| input_arg.to_string())
                } else {
                    input_arg.to_string()
                };

                let fields: serde_json::Value = match serde_json::from_str(&raw_json) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Invalid JSON fields: {}", e);
                        std::process::exit(1);
                    }
                };

                let submit = args.iter().any(|a| a == "--submit" || a == "--enter");
                let payload = serde_json::json!({
                    "fields": fields,
                    "submit": submit
                });
                let resp = Self::send_post(port, "/fill", &payload);
                println!("{}", resp);
                true
            }

            "clipboard" | "clip" => {
                let set_val = args.iter()
                    .position(|a| a == "--set" || a == "-s")
                    .and_then(|idx| args.get(idx + 1));

                if let Some(txt) = set_val {
                    let payload = serde_json::json!({
                        "action": "set",
                        "text": txt
                    });
                    let resp = Self::send_post(port, "/clipboard", &payload);
                    println!("{}", resp);
                } else {
                    let resp = Self::send_get(port, "/clipboard");
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                        if let Some(txt) = json["data"]["text"].as_str() {
                            println!("{}", txt);
                            return true;
                        }
                    }
                    println!("{}", resp);
                }
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

            "tabs" | "tab" => {
                let action = args.get(2).map(|s| s.as_str()).unwrap_or("list");
                match action {
                    "new" | "create" => {
                        let url = args.get(3).map(|s| s.as_str()).unwrap_or("about:home");
                        let payload = serde_json::json!({ "url": url });
                        let resp = Self::send_post(port, "/tabs/new", &payload);
                        println!("{}", resp);
                    }
                    "switch" => {
                        let idx: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let payload = serde_json::json!({ "index": idx });
                        let resp = Self::send_post(port, "/tabs/switch", &payload);
                        println!("{}", resp);
                    }
                    "close" => {
                        let idx: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);
                        let payload = serde_json::json!({ "index": idx });
                        let resp = Self::send_post(port, "/tabs/close", &payload);
                        println!("{}", resp);
                    }
                    _ => {
                        let resp = Self::send_get(port, "/tabs");
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                            if let Some(tabs) = json["data"]["tabs"].as_array() {
                                let active = json["data"]["active_index"].as_u64().unwrap_or(0);
                                println!("=== BROWSER TABS ({}, Active Tab: {}) ===", tabs.len(), active);
                                for t in tabs {
                                    let idx = t["index"].as_u64().unwrap_or(0);
                                    let is_act = t["is_active"].as_bool().unwrap_or(false);
                                    let is_sleep = t["is_sleeping"].as_bool().unwrap_or(false);
                                    let mark = if is_act { "*" } else { " " };
                                    let sleep_label = if is_sleep { " [sleeping]" } else { "" };
                                    println!(" {} [{}] {} ({}){}", mark, idx, t["title"].as_str().unwrap_or(""), t["url"].as_str().unwrap_or(""), sleep_label);
                                }
                                return true;
                            }
                        }
                        println!("{}", resp);
                    }
                }
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

            "playwright" | "pw" => {
                let is_python = args.iter().any(|a| a == "python" || a == "py" || a == "--python");
                let is_node = args.iter().any(|a| a == "node" || a == "js" || a == "ts" || a == "--node");
                let is_test = args.iter().any(|a| a == "test" || a == "--test");

                let resp = Self::send_get(port, "/cdp");
                let json = serde_json::from_str::<serde_json::Value>(&resp).unwrap_or(serde_json::json!({}));
                let cdp_port = json["cdp_port"].as_u64().unwrap_or(9222) as u16;
                let cdp_url = format!("http://127.0.0.1:{}", cdp_port);

                if is_test {
                    println!("Testing Playwright CDP connection to {}...", cdp_url);
                    match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Browser.getVersion", None) {
                        Ok(v) => {
                            println!("✅ CDP Connection Successful!");
                            println!("   Product:       {}", v["product"].as_str().unwrap_or("WebView2"));
                            println!("   User Agent:    {}", v["userAgent"].as_str().unwrap_or(""));
                            println!("   Protocol Ver:  {}", v["protocolVersion"].as_str().unwrap_or("1.3"));
                            println!("Playwright is ready to attach via:");
                            println!("   playwright.chromium.connect_over_cdp('{}')", cdp_url);
                        }
                        Err(e) => {
                            eprintln!("❌ CDP Connection Failed: {}", e);
                            eprintln!("Ensure Aerolith Agent is running and CDP port {} is open.", cdp_port);
                        }
                    }
                    return true;
                }

                if is_python {
                    println!("# Aerolith Agent - Python Playwright Quickstart");
                    println!("from playwright.sync_api import sync_playwright");
                    println!();
                    println!("with sync_playwright() as p:");
                    println!("    browser = p.chromium.connect_over_cdp(\"{}\")", cdp_url);
                    println!("    page = browser.contexts[0].pages[0]");
                    println!("    print(\"Connected to page:\", page.title())");
                    println!("    page.goto(\"https://example.com\")");
                    println!();
                    println!("# Extended API via integrations/playwright-python/aerolith_playwright.py:");
                    println!("# page.semantic_snapshot(viewport_only=True)");
                    println!("# page.highlight(\"3\")");
                    println!("# page.screenshot_element(\"3\", \"crop.png\")");
                    return true;
                }

                if is_node {
                    println!("// Aerolith Agent - Node.js Playwright Quickstart");
                    println!("const {{ chromium }} = require('playwright');");
                    println!();
                    println!("(async () => {{");
                    println!("    const browser = await chromium.connectOverCDP('{}');", cdp_url);
                    println!("    const page = browser.contexts()[0].pages()[0];");
                    println!("    console.log('Connected to page:', await page.title());");
                    println!("    await page.goto('https://example.com');");
                    println!("}})();");
                    return true;
                }

                println!("=== AEROLITH AGENT PLAYWRIGHT BRIDGE ===");
                println!("CDP Endpoint:        {}", cdp_url);
                println!("Browser Version URL: {}/json/version", cdp_url);
                println!("Page Targets URL:    {}/json/list", cdp_url);
                println!("--------------------------------------------------");
                println!("Quick Connect:");
                println!("  Python:  browser = playwright.chromium.connect_over_cdp('{}')", cdp_url);
                println!("  Node.js: const browser = await chromium.connectOverCDP('{}');", cdp_url);
                println!();
                println!("Commands:");
                println!("  aerolith-agent playwright --test     # Test live CDP WebSocket link");
                println!("  aerolith-agent playwright --python   # Print Python sample script");
                println!("  aerolith-agent playwright --node     # Print Node.js sample script");
                println!("--------------------------------------------------");
                println!("SDK Modules:");
                println!("  Python:  integrations/playwright-python/aerolith_playwright.py");
                println!("  Node.js: integrations/playwright-node/index.js");
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

            "emulate" | "set-viewport" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent emulate <width> <height> [--dpr <scale>] [--mobile] [--port <port>]");
                    std::process::exit(1);
                }
                let width = args[2].parse::<u32>().unwrap_or(1280);
                let height = args[3].parse::<u32>().unwrap_or(720);
                let dpr = args.iter()
                    .position(|a| a == "--dpr" || a == "-d")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<f64>().ok());
                let mobile = args.iter().any(|a| a == "--mobile" || a == "-m");

                let mut payload = serde_json::json!({
                    "width": width,
                    "height": height,
                    "mobile": mobile
                });
                if let Some(d) = dpr {
                    payload["dpr"] = serde_json::json!(d);
                }

                let resp = Self::send_post(port, "/emulate", &payload);
                println!("{}", resp);
                true
            }

            "console" | "logs" => {
                let clear = args.iter().any(|a| a == "--clear" || a == "-c");
                let endpoint = if clear { "/console/clear" } else { "/console" };
                let resp = Self::send_get(port, endpoint);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(logs) = json["data"]["logs"].as_array() {
                        println!("=== BROWSER CONSOLE LOGS ({}) ===", logs.len());
                        for entry in logs {
                            let level = entry["level"].as_str().unwrap_or("log").to_uppercase();
                            let msg = entry["message"].as_str().unwrap_or("");
                            let ts = entry["timestamp"].as_f64().unwrap_or(0.0);
                            println!("[{}] {:.0}ms: {}", level, ts, msg);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "pdf" | "print-pdf" => {
                let landscape = args.iter().any(|a| a == "--landscape" || a == "-l");
                let path = args.iter().skip(2).find(|a| a.ends_with(".pdf")).cloned();

                let endpoint = if landscape { "/pdf?landscape=1" } else { "/pdf" };
                let resp = Self::send_get(port, endpoint);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(b64) = json["data"]["pdf_base64"].as_str() {
                        if let Some(out_path) = path {
                            let bytes = Self::base64_decode(b64);
                            if let Ok(b) = bytes {
                                if std::fs::write(&out_path, b).is_ok() {
                                    println!("PDF saved to {}", out_path);
                                    return true;
                                }
                            }
                        }
                    }
                }
                println!("{}", resp);
                true
            }

            "hover" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent hover <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/hover", &payload);
                println!("{}", resp);
                true
            }

            "key" | "press-key" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent key <key_name> [--target <ref_or_selector>] [--port <port>]");
                    std::process::exit(1);
                }
                let key = &args[2];
                let target = args.iter()
                    .position(|a| a == "--target" || a == "-t")
                    .and_then(|idx| args.get(idx + 1));
                let mut payload = serde_json::json!({ "key": key });
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/key", &payload);
                println!("{}", resp);
                true
            }

            "select" | "select-option" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent select <ref_or_selector> <value> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let value = &args[3];
                let payload = serde_json::json!({
                    "target": target,
                    "value": value
                });
                let resp = Self::send_post(port, "/select", &payload);
                println!("{}", resp);
                true
            }

            "network" | "net" => {
                let clear = args.iter().any(|a| a == "--clear" || a == "-c");
                let endpoint = if clear { "/network/clear" } else { "/network" };
                let resp = if clear {
                    Self::send_post(port, endpoint, &serde_json::json!({}))
                } else {
                    Self::send_get(port, endpoint)
                };
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(requests) = json["data"]["requests"].as_array() {
                        let inflight = json["data"]["inflight_count"].as_u64().unwrap_or(0);
                        println!("=== RECORDED NETWORK REQUESTS ({}, {} In-Flight) ===", requests.len(), inflight);
                        for r in requests {
                            let method = r["method"].as_str().unwrap_or("GET");
                            let status = r["status"].as_u64().unwrap_or(0);
                            let url = r["url"].as_str().unwrap_or("");
                            let duration = r["duration_ms"].as_u64().unwrap_or(0);
                            println!("[{}] {} ({}ms): {}", status, method, duration, url);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "drag" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent drag <source> [--to <target>] [--dx <offset_x>] [--dy <offset_y>] [--steps <n>] [--port <port>]");
                    std::process::exit(1);
                }
                let source = &args[2];
                let target = args.iter().position(|a| a == "--to" || a == "-t").and_then(|idx| args.get(idx + 1));
                let dx = args.iter().position(|a| a == "--dx" || a == "-x").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<i32>().ok());
                let dy = args.iter().position(|a| a == "--dy" || a == "-y").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<i32>().ok());
                let steps = args.iter().position(|a| a == "--steps" || a == "-s").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<u32>().ok()).unwrap_or(10);

                let mut payload = serde_json::json!({
                    "source": source,
                    "steps": steps
                });
                if let Some(t) = target { payload["target"] = serde_json::json!(t); }
                if let Some(x) = dx { payload["offset_x"] = serde_json::json!(x); }
                if let Some(y) = dy { payload["offset_y"] = serde_json::json!(y); }

                let resp = Self::send_post(port, "/drag", &payload);
                println!("{}", resp);
                true
            }

            "cookies" | "cookie" => {
                let is_delete = args.iter().any(|a| a == "--delete" || a == "-d");
                let is_clear = args.iter().any(|a| a == "--clear" || a == "-c");
                let is_set = args.iter().any(|a| a == "--set" || a == "-s");

                if is_clear {
                    let resp = Self::send_post(port, "/cookies/clear", &serde_json::json!({}));
                    println!("{}", resp);
                    return true;
                }

                if is_delete {
                    let name = args.iter().position(|a| a == "--delete" || a == "-d").and_then(|idx| args.get(idx + 1));
                    let domain = args.iter().position(|a| a == "--domain").and_then(|idx| args.get(idx + 1));
                    let payload = serde_json::json!({
                        "name": name,
                        "domain": domain
                    });
                    let resp = Self::send_post(port, "/cookies/delete", &payload);
                    println!("{}", resp);
                    return true;
                }

                if is_set {
                    let cookie_arg = args.iter().position(|a| a == "--set" || a == "-s").and_then(|idx| args.get(idx + 1));
                    if let Some(raw) = cookie_arg {
                        let json: serde_json::Value = serde_json::from_str(raw).unwrap_or(serde_json::json!({ "name": raw, "value": "" }));
                        let resp = Self::send_post(port, "/cookies", &json);
                        println!("{}", resp);
                        return true;
                    }
                }

                let resp = Self::send_get(port, "/cookies");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(cookies) = json["data"]["cookies"].as_array() {
                        println!("=== BROWSER COOKIES ({}) ===", cookies.len());
                        for c in cookies {
                            let name = c["name"].as_str().unwrap_or("");
                            let val = c["value"].as_str().unwrap_or("");
                            let dom = c["domain"].as_str().unwrap_or("");
                            let http_only = c["httpOnly"].as_bool().unwrap_or(false);
                            let secure = c["secure"].as_bool().unwrap_or(false);
                            println!("  {}={} (domain={}, httpOnly={}, secure={})", name, val, dom, http_only, secure);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "headers" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent headers <json_or_key_val> [--port <port>]");
                    std::process::exit(1);
                }
                let raw = &args[2];
                let payload: serde_json::Value = serde_json::from_str(raw).unwrap_or_else(|_| {
                    if let Some((k, v)) = raw.split_once(':') {
                        serde_json::json!({ k.trim(): v.trim() })
                    } else {
                        serde_json::json!({})
                    }
                });
                let resp = Self::send_post(port, "/headers", &payload);
                println!("{}", resp);
                true
            }

            "user-agent" | "ua" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent user-agent <user_agent_string> [--lang <accept_language>] [--platform <platform>] [--port <port>]");
                    std::process::exit(1);
                }
                let ua = &args[2];
                let lang = args.iter().position(|a| a == "--lang" || a == "-l").and_then(|idx| args.get(idx + 1));
                let plat = args.iter().position(|a| a == "--platform").and_then(|idx| args.get(idx + 1));
                let mut payload = serde_json::json!({ "user_agent": ua });
                if let Some(l) = lang { payload["accept_language"] = serde_json::json!(l); }
                if let Some(p) = plat { payload["platform"] = serde_json::json!(p); }

                let resp = Self::send_post(port, "/user_agent", &payload);
                println!("{}", resp);
                true
            }

            "upload" | "upload-file" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent upload <target> <file_path...> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let files: Vec<String> = args[3..].iter()
                    .filter(|a| !a.starts_with("--port"))
                    .cloned()
                    .collect();
                let payload = serde_json::json!({
                    "target": target,
                    "files": files
                });
                let resp = Self::send_post(port, "/upload", &payload);
                println!("{}", resp);
                true
            }

            "location" | "geo" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent location <lat> <lng> [--accuracy <m>] [--timezone <tz>] [--port <port>]");
                    std::process::exit(1);
                }
                let lat = args[2].parse::<f64>().unwrap_or(0.0);
                let lng = args[3].parse::<f64>().unwrap_or(0.0);
                let acc = args.iter().position(|a| a == "--accuracy" || a == "-a").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<f64>().ok());
                let tz = args.iter().position(|a| a == "--timezone" || a == "-t").and_then(|idx| args.get(idx + 1));
                let mut payload = serde_json::json!({
                    "latitude": lat,
                    "longitude": lng
                });
                if let Some(a) = acc { payload["accuracy"] = serde_json::json!(a); }
                if let Some(t) = tz { payload["timezone"] = serde_json::json!(t); }
                let resp = Self::send_post(port, "/location", &payload);
                println!("{}", resp);
                true
            }

            "media" => {
                let action = if args.iter().any(|a| a == "--pause") {
                    "pause"
                } else if args.iter().any(|a| a == "--play") {
                    "play"
                } else if args.iter().any(|a| a == "--mute") {
                    "mute"
                } else if args.iter().any(|a| a == "--unmute") {
                    "unmute"
                } else {
                    args.get(2).map(|s| s.as_str()).unwrap_or("status")
                };

                let resp = if action == "status" {
                    Self::send_get(port, "/media")
                } else {
                    Self::send_post(port, "/media", &serde_json::json!({ "action": action }))
                };
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(media) = json["data"]["media"].as_array() {
                        println!("=== MEDIA PLAYBACK STATUS ({}) ===", media.len());
                        for m in media {
                            let tag = m["tag"].as_str().unwrap_or("media");
                            let paused = m["paused"].as_bool().unwrap_or(true);
                            let muted = m["muted"].as_bool().unwrap_or(false);
                            let cur = m["current_time"].as_i64().unwrap_or(0);
                            let dur = m["duration"].as_i64().unwrap_or(0);
                            let src = m["src"].as_str().unwrap_or("");
                            println!("  <{}> paused={}, muted={}, time={}/{}s src={}", tag, paused, muted, cur, dur, src);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "inspect" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent inspect <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/inspect", &payload);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        let data = &json["data"];
                        println!("=== ELEMENT INSPECTION: {} ===", data["target"].as_str().unwrap_or(target));
                        println!("  Tag: <{}>", data["tag_name"].as_str().unwrap_or(""));
                        println!("  Text: \"{}\"", data["text"].as_str().unwrap_or(""));
                        if !data["value"].is_null() {
                            println!("  Value: {}", data["value"]);
                        }
                        let state = &data["state"];
                        println!("  State: visible={}, in_viewport={}, focused={}, clickable={}, disabled={}",
                            state["is_visible"], state["is_in_viewport"], state["is_focused"], state["is_clickable"], data["disabled"]);
                        let rect = &data["rect"];
                        println!("  Bounds: {}x{} at ({}, {})", rect["width"], rect["height"], rect["x"], rect["y"]);
                        let style = &data["computed_style"];
                        println!("  Styles: display={}, visibility={}, opacity={}, pointer-events={}, z-index={}",
                            style["display"], style["visibility"], style["opacity"], style["pointer_events"], style["z_index"]);
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "route" => {
                let is_clear = args.iter().any(|a| a == "--clear" || a == "-c");
                if is_clear {
                    let resp = Self::send_delete(port, "/route");
                    println!("{}", resp);
                    return true;
                }

                let block_pattern = args.iter()
                    .position(|a| a == "--block" || a == "--abort" || a == "-b")
                    .and_then(|idx| args.get(idx + 1));
                let mock_pattern = args.iter()
                    .position(|a| a == "--mock" || a == "-m")
                    .and_then(|idx| args.get(idx + 1));

                if let Some(pat) = block_pattern {
                    let payload = serde_json::json!({
                        "action": "abort",
                        "pattern": pat,
                        "rule_type": "abort"
                    });
                    let resp = Self::send_post(port, "/route", &payload);
                    println!("{}", resp);
                    return true;
                }

                if let Some(pat) = mock_pattern {
                    let body_str = args.iter()
                        .position(|a| a == "--body")
                        .and_then(|idx| args.get(idx + 1))
                        .map(|s| s.as_str())
                        .unwrap_or("{}");
                    let status = args.iter()
                        .position(|a| a == "--status")
                        .and_then(|idx| args.get(idx + 1))
                        .and_then(|s| s.parse::<u16>().ok())
                        .unwrap_or(200);
                    let payload = serde_json::json!({
                        "action": "mock",
                        "pattern": pat,
                        "rule_type": "mock",
                        "status": status,
                        "body": body_str
                    });
                    let resp = Self::send_post(port, "/route", &payload);
                    println!("{}", resp);
                    return true;
                }

                let resp = Self::send_get(port, "/route");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(routes) = json["data"]["routes"].as_array() {
                        println!("=== ACTIVE ROUTE INTERCEPTION RULES ({}) ===", routes.len());
                        for r in routes {
                            let pat = r["pattern"].as_str().unwrap_or("*");
                            let rtype = r["rule_type"].as_str().unwrap_or("abort");
                            let st = r["status"].as_u64().unwrap_or(200);
                            println!("  [{}] {} (status: {})", rtype.to_uppercase(), pat, st);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "dblclick" | "double-click" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent dblclick <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/dblclick", &payload);
                println!("{}", resp);
                true
            }

            "right-click" | "contextmenu" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent right-click <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/right_click", &payload);
                println!("{}", resp);
                true
            }

            "table" | "extract-table" => {
                let target = args.get(2).filter(|a| !a.starts_with("--"));
                let mut path = "/table".to_string();
                if let Some(t) = target {
                    path = format!("/table?target={}", t);
                }
                let resp = Self::send_get(port, &path);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        let data = &json["data"];
                        let title = data["title"].as_str().unwrap_or("Untitled Table");
                        let rows = data["rows"].as_array();
                        let count = data["row_count"].as_u64().unwrap_or(0);
                        let cols = data["col_count"].as_u64().unwrap_or(0);
                        println!("=== EXTRACTED TABLE: {} ({} cols, {} rows) ===", title, cols, count);
                        if let Some(r_list) = rows {
                            for (idx, r) in r_list.iter().take(10).enumerate() {
                                println!("  [{}] {}", idx + 1, r);
                            }
                            if r_list.len() > 10 {
                                println!("  ... and {} more rows", r_list.len() - 10);
                            }
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "downloads" | "dls" => {
                let is_clear = args.iter().any(|a| a == "--clear" || a == "-c");
                let resp = if is_clear {
                    Self::send_post(port, "/downloads/clear", &serde_json::json!({}))
                } else {
                    Self::send_get(port, "/downloads")
                };
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if let Some(dls) = json["data"]["downloads"].as_array() {
                        println!("=== RECORDED FILE DOWNLOADS ({}) ===", dls.len());
                        for (idx, d) in dls.iter().enumerate() {
                            let fn_str = d["filename"].as_str().unwrap_or("unnamed");
                            let url_str = d["url"].as_str().unwrap_or("");
                            println!("  [{}] {} ({})", idx + 1, fn_str, url_str);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "throttle" | "network-emulate" => {
                let profile = args.iter().position(|a| a == "--profile" || a == "-p").and_then(|idx| args.get(idx + 1)).map(|s| s.as_str()).unwrap_or("online");
                let offline = args.iter().any(|a| a == "--offline");
                let latency = args.iter().position(|a| a == "--latency" || a == "-l").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<f64>().ok());
                let download = args.iter().position(|a| a == "--download" || a == "-d").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<f64>().ok());
                let upload = args.iter().position(|a| a == "--upload" || a == "-u").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<f64>().ok());

                let mut payload = serde_json::json!({
                    "profile": profile,
                    "offline": offline
                });
                if let Some(lat) = latency { payload["latency"] = serde_json::json!(lat); }
                if let Some(dl) = download { payload["download"] = serde_json::json!(dl); }
                if let Some(ul) = upload { payload["upload"] = serde_json::json!(ul); }

                let resp = Self::send_post(port, "/network/emulate", &payload);
                println!("{}", resp);
                true
            }

            "css" | "inject-css" => {
                let is_clear = args.iter().any(|a| a == "--clear" || a == "-c");
                if is_clear {
                    let resp = Self::send_post(port, "/css/clear", &serde_json::json!({}));
                    println!("{}", resp);
                    return true;
                }
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent css \"<css_rules>\" [--port <port>] or --clear");
                    std::process::exit(1);
                }
                let css = &args[2];
                let payload = serde_json::json!({ "css": css });
                let resp = Self::send_post(port, "/css", &payload);
                println!("{}", resp);
                true
            }

            "query" | "query-elements" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent query <selector> [--attr <name>] [--limit <n>] [--port <port>]");
                    std::process::exit(1);
                }
                let selector = &args[2];
                let attr = args.iter().position(|a| a == "--attr" || a == "-a").and_then(|idx| args.get(idx + 1));
                let limit = args.iter().position(|a| a == "--limit" || a == "-n").and_then(|idx| args.get(idx + 1)).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
                let mut payload = serde_json::json!({
                    "selector": selector,
                    "max_items": limit
                });
                if let Some(a) = attr { payload["attribute"] = serde_json::json!(a); }
                let resp = Self::send_post(port, "/query", &payload);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        let data = &json["data"];
                        let count = data["count"].as_u64().unwrap_or(0);
                        let total = data["total_matched"].as_u64().unwrap_or(0);
                        let items = data["items"].as_array();
                        println!("=== MATCHED ELEMENTS: {} (showing {} of {}) ===", selector, count, total);
                        if let Some(i_list) = items {
                            for item in i_list {
                                let ref_id = item["ref"].as_str().unwrap_or("");
                                let tag = item["tag_name"].as_str().unwrap_or("");
                                let text = item["text"].as_str().unwrap_or("");
                                let attr_str = if !item["attribute"].is_null() {
                                    format!(" attr={}", item["attribute"])
                                } else {
                                    String::new()
                                };
                                println!("  [@{}] <{}> \"{}\"{}", ref_id, tag, text, attr_str);
                            }
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "focus" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent focus <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/focus", &payload);
                println!("{}", resp);
                true
            }

            "blur" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent blur <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/blur", &payload);
                println!("{}", resp);
                true
            }

            "info" | "page-info" => {
                let resp = Self::send_get(port, "/page_info");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        let data = &json["data"];
                        println!("=== PAGE METRICS & STATE ===");
                        println!("  URL: {}", data["url"].as_str().unwrap_or(""));
                        println!("  Title: {}", data["title"].as_str().unwrap_or(""));
                        println!("  Ready State: {}", data["ready_state"].as_str().unwrap_or(""));
                        println!("  Viewport: {}x{}", data["viewport_width"], data["viewport_height"]);
                        println!("  Document: {}x{}", data["document_width"], data["document_height"]);
                        println!("  Scroll: ({}, {}) - Progress: {}%",
                            data["scroll_x"], data["scroll_y"], (data["scroll_progress"].as_f64().unwrap_or(0.0) * 100.0).round());
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "html" | "get-html" => {
                let target = if args.len() >= 3 && !args[2].starts_with('-') {
                    Some(args[2].as_str())
                } else {
                    None
                };
                let endpoint = if let Some(t) = target {
                    format!("/html?target={}", crate::url_utils::urlencoding_simple(t))
                } else {
                    "/html".to_string()
                };
                let resp = Self::send_get(port, &endpoint);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        if let Some(html) = json["data"]["html"].as_str() {
                            println!("{}", html);
                            return true;
                        }
                    }
                }
                println!("{}", resp);
                true
            }

            "scroll-into-view" | "scroll-in-view" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent scroll-into-view <ref_or_selector> [--align <start|center|end>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let mut align = "center".to_string();
                for i in 3..args.len() {
                    if args[i] == "--align" && i + 1 < args.len() {
                        align = args[i + 1].clone();
                    }
                }
                let payload = serde_json::json!({ "target": target, "align": align });
                let resp = Self::send_post(port, "/scroll_into_view", &payload);
                println!("{}", resp);
                true
            }

            "zoom" | "set-zoom" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent zoom <factor> [--port <port>]");
                    std::process::exit(1);
                }
                let factor: f64 = args[2].parse().unwrap_or(1.0);
                let payload = serde_json::json!({ "zoom_factor": factor });
                let resp = Self::send_post(port, "/zoom", &payload);
                println!("{}", resp);
                true
            }

            "storage-item" | "storage-key" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent storage-item <get|set|delete|clear|keys> [--type <local|session>] [--key <name>] [--val <value>] [--port <port>]");
                    std::process::exit(1);
                }
                let action = &args[2];
                let mut storage_type = "local".to_string();
                let mut key: Option<String> = None;
                let mut val: Option<String> = None;

                let mut i = 3;
                while i < args.len() {
                    if (args[i] == "--type" || args[i] == "-t") && i + 1 < args.len() {
                        storage_type = args[i + 1].clone();
                        i += 2;
                    } else if (args[i] == "--key" || args[i] == "-k") && i + 1 < args.len() {
                        key = Some(args[i + 1].clone());
                        i += 2;
                    } else if (args[i] == "--val" || args[i] == "--value" || args[i] == "-v") && i + 1 < args.len() {
                        val = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }

                let payload = serde_json::json!({
                    "storage_type": storage_type,
                    "action": action,
                    "key": key,
                    "value": val
                });
                let resp = Self::send_post(port, "/storage/item", &payload);
                println!("{}", resp);
                true
            }

            "assert-element" | "assert" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent assert-element <target> [--visible] [--hidden] [--disabled] [--checked] [--text <str>] [--val <str>] [--class <name>] [--attr <name>] [--attr-val <str>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let mut assertions = serde_json::Map::new();

                let mut i = 3;
                while i < args.len() {
                    match args[i].as_str() {
                        "--visible" => { assertions.insert("visible".into(), serde_json::Value::Bool(true)); i += 1; }
                        "--hidden" => { assertions.insert("hidden".into(), serde_json::Value::Bool(true)); i += 1; }
                        "--disabled" => { assertions.insert("disabled".into(), serde_json::Value::Bool(true)); i += 1; }
                        "--enabled" => { assertions.insert("disabled".into(), serde_json::Value::Bool(false)); i += 1; }
                        "--checked" => { assertions.insert("checked".into(), serde_json::Value::Bool(true)); i += 1; }
                        "--text" if i + 1 < args.len() => { assertions.insert("text_contains".into(), serde_json::Value::String(args[i + 1].clone())); i += 2; }
                        "--val" | "--value" if i + 1 < args.len() => { assertions.insert("value".into(), serde_json::Value::String(args[i + 1].clone())); i += 2; }
                        "--class" if i + 1 < args.len() => { assertions.insert("has_class".into(), serde_json::Value::String(args[i + 1].clone())); i += 2; }
                        "--attr" if i + 1 < args.len() => { assertions.insert("attr_name".into(), serde_json::Value::String(args[i + 1].clone())); i += 2; }
                        "--attr-val" if i + 1 < args.len() => { assertions.insert("attr_value".into(), serde_json::Value::String(args[i + 1].clone())); i += 2; }
                        _ => { i += 1; }
                    }
                }

                let payload = serde_json::json!({
                    "target": target,
                    "assertions": assertions
                });
                let resp = Self::send_post(port, "/assert_element", &payload);
                println!("{}", resp);
                true
            }

            "wait-selector" | "wait-state" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent wait-selector <target> [--state <visible|hidden|attached|detached>] [--timeout <ms>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let mut state = "visible".to_string();
                let mut timeout_ms = 5000u64;

                let mut i = 3;
                while i < args.len() {
                    if args[i] == "--state" && i + 1 < args.len() {
                        state = args[i + 1].clone();
                        i += 2;
                    } else if (args[i] == "--timeout" || args[i] == "-t") && i + 1 < args.len() {
                        timeout_ms = args[i + 1].parse().unwrap_or(5000);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }

                let payload = serde_json::json!({
                    "target": target,
                    "state": state,
                    "timeout_ms": timeout_ms
                });
                let resp = Self::send_post(port, "/wait_for_selector", &payload);
                println!("{}", resp);
                true
            }

            "perf" | "performance" => {
                let resp = Self::send_get(port, "/performance");
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                    if json["status"] == "ok" {
                        let m = &json["data"]["metrics"];
                        println!("=== PERFORMANCE METRICS ===");
                        println!("  URL: {}", m["url"].as_str().unwrap_or(""));
                        println!("  TTFB: {} ms", m["ttfb_ms"]);
                        println!("  DOMContentLoaded: {} ms", m["dom_content_loaded_ms"]);
                        println!("  Page Load: {} ms", m["load_ms"]);
                        println!("  Total Resources: {}", m["total_resources"]);
                        println!("  Transfer Size: {} KB", (m["total_transfer_bytes"].as_f64().unwrap_or(0.0) / 1024.0).round());
                        if let Some(mem) = m["js_heap_used_mb"].as_f64() {
                            println!("  JS Heap Used: {} MB", mem);
                        }
                        return true;
                    }
                }
                println!("{}", resp);
                true
            }

            "scroll-to" | "scroll-coords" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent scroll-to <x> <y> [--smooth] [--port <port>]");
                    std::process::exit(1);
                }
                let x: i32 = args[2].parse().unwrap_or(0);
                let y: i32 = args[3].parse().unwrap_or(0);
                let mut smooth = false;
                for i in 4..args.len() {
                    if args[i] == "--smooth" {
                        smooth = true;
                    }
                }
                let payload = serde_json::json!({ "x": x, "y": y, "smooth": smooth });
                let resp = Self::send_post(port, "/scroll_to", &payload);
                println!("{}", resp);
                true
            }

            "outline" | "page-outline" => {
                let mut max_links = 25usize;
                let mut max_buttons = 25usize;
                let mut i = 2;
                while i < args.len() {
                    if (args[i] == "--links" || args[i] == "-l") && i + 1 < args.len() {
                        max_links = args[i + 1].parse().unwrap_or(25);
                        i += 2;
                    } else if (args[i] == "--buttons" || args[i] == "-b") && i + 1 < args.len() {
                        max_buttons = args[i + 1].parse().unwrap_or(25);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({ "max_links": max_links, "max_buttons": max_buttons });
                let resp = Self::send_post(port, "/outline", &payload);
                println!("{}", resp);
                true
            }

            "wait-idle" | "wait-network-idle" => {
                let mut idle_ms = 500u64;
                let mut timeout_ms = 10000u64;
                let mut max_inflight = 0usize;
                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--idle" && i + 1 < args.len() {
                        idle_ms = args[i + 1].parse().unwrap_or(500);
                        i += 2;
                    } else if (args[i] == "--timeout" || args[i] == "-t") && i + 1 < args.len() {
                        timeout_ms = args[i + 1].parse().unwrap_or(10000);
                        i += 2;
                    } else if args[i] == "--max" && i + 1 < args.len() {
                        max_inflight = args[i + 1].parse().unwrap_or(0);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({
                    "idle_time_ms": idle_ms,
                    "timeout_ms": timeout_ms,
                    "max_inflight": max_inflight
                });
                let resp = Self::send_post(port, "/wait_for_idle", &payload);
                println!("{}", resp);
                true
            }

            "mutations" | "watch-mutations" => {
                let action = if args.len() >= 3 && !args[2].starts_with('-') {
                    args[2].clone()
                } else {
                    "get".to_string()
                };
                let mut target: Option<String> = None;
                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--target" && i + 1 < args.len() {
                        target = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({ "action": action, "target": target });
                let resp = Self::send_post(port, "/mutations", &payload);
                println!("{}", resp);
                true
            }

            "tab-eval" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent tab-eval <tab_index> <expression> [--port <port>]");
                    std::process::exit(1);
                }
                let tab_index: usize = args[2].parse().unwrap_or(0);
                let expression = &args[3];
                let payload = serde_json::json!({ "tab_index": tab_index, "script": expression });
                let resp = Self::send_post(port, "/tabs/eval", &payload);
                println!("{}", resp);
                true
            }

            "response-body" | "network-body" => {
                let pattern = if args.len() >= 3 && !args[2].starts_with('-') {
                    args[2].clone()
                } else {
                    String::new()
                };
                let clear = args.iter().any(|a| a == "--clear");
                let payload = serde_json::json!({ "pattern": pattern, "clear": clear });
                let resp = Self::send_post(port, "/network/body", &payload);
                println!("{}", resp);
                true
            }

            "drop-file" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent drop-file <target> <file_path_or_content> [--name <name>] [--mime <type>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let input = &args[3];
                let mut file_name = "upload.bin".to_string();
                let mut mime_type = "application/octet-stream".to_string();
                let content = if let Ok(bytes) = std::fs::read(input) {
                    file_name = std::path::Path::new(input)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("upload.bin")
                        .to_string();
                    if file_name.ends_with(".csv") { mime_type = "text/csv".to_string(); }
                    else if file_name.ends_with(".json") { mime_type = "application/json".to_string(); }
                    else if file_name.ends_with(".png") { mime_type = "image/png".to_string(); }
                    else if file_name.ends_with(".jpg") || file_name.ends_with(".jpeg") { mime_type = "image/jpeg".to_string(); }
                    else if file_name.ends_with(".txt") { mime_type = "text/plain".to_string(); }
                    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
                    let mut b64 = String::new();
                    for chunk in bytes.chunks(3) {
                        let b0 = chunk[0] as u32;
                        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
                        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
                        let triple = (b0 << 16) | (b1 << 8) | b2;
                        b64.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
                        b64.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
                        b64.push(if chunk.len() > 1 { TABLE[((triple >> 6) & 0x3F) as usize] as char } else { '=' });
                        b64.push(if chunk.len() > 2 { TABLE[(triple & 0x3F) as usize] as char } else { '=' });
                    }
                    b64
                } else {
                    input.clone()
                };

                let mut i = 4;
                while i < args.len() {
                    if (args[i] == "--name" || args[i] == "-n") && i + 1 < args.len() {
                        file_name = args[i + 1].clone();
                        i += 2;
                    } else if (args[i] == "--mime" || args[i] == "-m") && i + 1 < args.len() {
                        mime_type = args[i + 1].clone();
                        i += 2;
                    } else {
                        i += 1;
                    }
                }

                let payload = serde_json::json!({
                    "target": target,
                    "file_name": file_name,
                    "mime_type": mime_type,
                    "content": content
                });
                let resp = Self::send_post(port, "/drop_file", &payload);
                println!("{}", resp);
                true
            }

            "dom-tree" | "tree" => {
                let target = if args.len() >= 3 && !args[2].starts_with('-') {
                    Some(args[2].clone())
                } else {
                    None
                };
                let mut max_depth = 4usize;
                let mut i = 2;
                while i < args.len() {
                    if (args[i] == "--depth" || args[i] == "-d") && i + 1 < args.len() {
                        max_depth = args[i + 1].parse().unwrap_or(4);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({ "target": target, "max_depth": max_depth });
                let resp = Self::send_post(port, "/dom/tree", &payload);
                println!("{}", resp);
                true
            }

            "offline" | "set-offline" => {
                let offline = if args.len() >= 3 {
                    args[2] != "false" && args[2] != "0" && args[2] != "off"
                } else {
                    true
                };
                let payload = serde_json::json!({ "offline": offline });
                let resp = Self::send_post(port, "/network/offline", &payload);
                println!("{}", resp);
                true
            }

            "wait-text" | "wait-for-text" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent wait-text <text> [--target <ref_or_selector>] [--timeout <ms>] [--port <port>]");
                    std::process::exit(1);
                }
                let text = &args[2];
                let mut target: Option<String> = None;
                let mut timeout_ms = 5000u64;
                let mut i = 3;
                while i < args.len() {
                    if (args[i] == "--target" || args[i] == "--selector") && i + 1 < args.len() {
                        target = Some(args[i + 1].clone());
                        i += 2;
                    } else if (args[i] == "--timeout" || args[i] == "-t") && i + 1 < args.len() {
                        timeout_ms = args[i + 1].parse().unwrap_or(5000);
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({
                    "text": text,
                    "target": target,
                    "timeout_ms": timeout_ms
                });
                let resp = Self::send_post(port, "/wait_for_text", &payload);
                println!("{}", resp);
                true
            }

            "computed-styles" | "styles" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent computed-styles <target> [prop1 prop2...] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let mut props: Vec<String> = Vec::new();
                let mut i = 3;
                while i < args.len() {
                    if args[i] == "--port" || args[i] == "-p" {
                        i += 2;
                    } else if !args[i].starts_with('-') {
                        props.push(args[i].clone());
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                let payload = if props.is_empty() {
                    serde_json::json!({ "target": target })
                } else {
                    serde_json::json!({ "target": target, "properties": props })
                };
                let resp = Self::send_post(port, "/computed_styles", &payload);
                println!("{}", resp);
                true
            }

            "wait-download" | "wait-for-download" => {
                let mut pattern: Option<String> = None;
                let mut timeout_ms = 15000u64;
                let mut i = 2;
                while i < args.len() {
                    if (args[i] == "--timeout" || args[i] == "-t") && i + 1 < args.len() {
                        timeout_ms = args[i + 1].parse().unwrap_or(15000);
                        i += 2;
                    } else if (args[i] == "--pattern" || args[i] == "-p") && i + 1 < args.len() {
                        pattern = Some(args[i + 1].clone());
                        i += 2;
                    } else if !args[i].starts_with('-') && pattern.is_none() {
                        pattern = Some(args[i].clone());
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({
                    "file_pattern": pattern,
                    "timeout_ms": timeout_ms
                });
                let resp = Self::send_post(port, "/wait_for_download", &payload);
                println!("{}", resp);
                true
            }

            "set-content" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent set-content <html_or_file> [--port <port>]");
                    std::process::exit(1);
                }
                let input = &args[2];
                let html = if let Ok(content) = std::fs::read_to_string(input) {
                    content
                } else {
                    input.clone()
                };
                let payload = serde_json::json!({ "html": html });
                let resp = Self::send_post(port, "/set_content", &payload);
                println!("{}", resp);
                true
            }

            "forms" | "get-forms" => {
                let target = if args.len() >= 3 && !args[2].starts_with('-') {
                    Some(args[2].clone())
                } else {
                    None
                };
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/forms", &payload);
                println!("{}", resp);
                true
            }

            "visibility" | "set-visibility" => {
                let state = if args.len() >= 3 && !args[2].starts_with('-') {
                    &args[2]
                } else {
                    "visible"
                };
                let payload = serde_json::json!({ "state": state });
                let resp = Self::send_post(port, "/visibility", &payload);
                println!("{}", resp);
                true
            }

            "emulate-media" | "media-emulate" => {
                let mut media_type: Option<String> = None;
                let mut color_scheme: Option<String> = None;
                let mut i = 2;
                while i < args.len() {
                    if (args[i] == "--media" || args[i] == "-m") && i + 1 < args.len() {
                        media_type = Some(args[i + 1].clone());
                        i += 2;
                    } else if (args[i] == "--scheme" || args[i] == "-s" || args[i] == "--color-scheme") && i + 1 < args.len() {
                        color_scheme = Some(args[i + 1].clone());
                        i += 2;
                    } else if !args[i].starts_with('-') && color_scheme.is_none() {
                        color_scheme = Some(args[i].clone());
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                let payload = serde_json::json!({
                    "media_type": media_type,
                    "color_scheme": color_scheme
                });
                let resp = Self::send_post(port, "/emulate/media", &payload);
                println!("{}", resp);
                true
            }

            "bounds" | "element-bounds" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent bounds <ref_or_selector> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/element/bounds", &payload);
                println!("{}", resp);
                true
            }

            "mock-route" | "mock" => {
                let mut action = "set".to_string();
                let mut pattern: Option<String> = None;
                let mut body: Option<String> = None;
                let mut status = 200u16;
                let mut content_type = "application/json".to_string();

                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--clear" {
                        action = "clear".to_string();
                        i += 1;
                    } else if args[i] == "--list" {
                        action = "list".to_string();
                        i += 1;
                    } else if (args[i] == "--status" || args[i] == "-s") && i + 1 < args.len() {
                        status = args[i + 1].parse().unwrap_or(200);
                        i += 2;
                    } else if (args[i] == "--type" || args[i] == "-t") && i + 1 < args.len() {
                        content_type = args[i + 1].clone();
                        i += 2;
                    } else if args[i] == "--port" || args[i] == "-p" {
                        i += 2;
                    } else if !args[i].starts_with('-') {
                        if pattern.is_none() {
                            pattern = Some(args[i].clone());
                        } else if body.is_none() {
                            body = Some(args[i].clone());
                        }
                        i += 1;
                    } else {
                        i += 1;
                    }
                }

                let payload = serde_json::json!({
                    "action": action,
                    "pattern": pattern,
                    "status": status,
                    "content_type": content_type,
                    "body": body
                });
                let resp = Self::send_post(port, "/network/mock", &payload);
                println!("{}", resp);
                true
            }

            "viewport" | "get-viewport" => {
                let resp = Self::send_get(port, "/viewport");
                println!("{}", resp);
                true
            }

            "attr-set" | "set-attr" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent attr-set <target> <attribute> [value] [--remove] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let attribute = &args[3];
                let remove = args.iter().any(|a| a == "--remove");
                let value = if remove {
                    None
                } else if args.len() >= 5 && !args[4].starts_with('-') {
                    Some(args[4].clone())
                } else {
                    Some("".to_string())
                };

                let payload = serde_json::json!({
                    "target": target,
                    "attribute": attribute,
                    "value": value
                });
                let resp = Self::send_post(port, "/element/attribute", &payload);
                println!("{}", resp);
                true
            }

            "search-text" | "search-page-text" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent search-text <query> [--max <n>] [--case] [--port <port>]");
                    std::process::exit(1);
                }
                let query = &args[2];
                let max_matches = args.iter()
                    .position(|a| a == "--max" || a == "-m")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());
                let case_sensitive = args.iter().any(|a| a == "--case" || a == "-c");

                let mut payload = serde_json::json!({
                    "query": query,
                    "case_sensitive": case_sensitive
                });
                if let Some(m) = max_matches {
                    payload["max_matches"] = serde_json::json!(m);
                }
                let resp = Self::send_post(port, "/search/text", &payload);
                println!("{}", resp);
                true
            }

            "storage-dump" | "dump-storage" => {
                let storage_type = args.iter()
                    .position(|a| a == "--type" || a == "-t")
                    .and_then(|idx| args.get(idx + 1))
                    .map(|s| s.as_str())
                    .unwrap_or("all");

                let endpoint = format!("/storage/dump?type={}", crate::url_utils::urlencoding_simple(storage_type));
                let resp = Self::send_get(port, &endpoint);
                println!("{}", resp);
                true
            }

            "human-jitter" | "jitter" => {
                let duration_ms = args.iter()
                    .position(|a| a == "--duration" || a == "-d")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<u64>().ok());
                let no_mouse = args.iter().any(|a| a == "--no-mouse");
                let no_scroll = args.iter().any(|a| a == "--no-scroll");

                let mut payload = serde_json::json!({
                    "move_mouse": !no_mouse,
                    "micro_scroll": !no_scroll
                });
                if let Some(d) = duration_ms {
                    payload["duration_ms"] = serde_json::json!(d);
                }
                let resp = Self::send_post(port, "/human_jitter", &payload);
                println!("{}", resp);
                true
            }

            "type-keystrokes" | "keystrokes" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent type-keystrokes <target> <text> [--delay <ms>] [--clear] [--submit] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let text = &args[3];
                let delay_ms = args.iter()
                    .position(|a| a == "--delay" || a == "-d")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<u64>().ok());
                let clear = args.iter().any(|a| a == "--clear" || a == "-c");
                let submit = args.iter().any(|a| a == "--submit" || a == "--enter");

                let mut payload = serde_json::json!({
                    "target": target,
                    "text": text,
                    "clear": clear,
                    "submit": submit
                });
                if let Some(d) = delay_ms {
                    payload["delay_ms"] = serde_json::json!(d);
                }
                let resp = Self::send_post(port, "/type/keystrokes", &payload);
                println!("{}", resp);
                true
            }

            "canvas-data" | "get-canvas" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent canvas-data <target> [--format <png|jpeg|webp>] [--quality <0-1>] [--out <file.png>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let format = args.iter()
                    .position(|a| a == "--format" || a == "-f")
                    .and_then(|idx| args.get(idx + 1))
                    .map(|s| s.as_str())
                    .unwrap_or("image/png");
                let quality = args.iter()
                    .position(|a| a == "--quality" || a == "-q")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.92);
                let out_file = args.iter()
                    .position(|a| a == "--out" || a == "-o")
                    .and_then(|idx| args.get(idx + 1))
                    .cloned();

                let mime_type = if format == "jpeg" || format == "jpg" {
                    "image/jpeg"
                } else if format == "webp" {
                    "image/webp"
                } else {
                    "image/png"
                };

                let payload = serde_json::json!({
                    "target": target,
                    "mime_type": mime_type,
                    "quality": quality
                });
                let resp = Self::send_post(port, "/canvas/data", &payload);

                if let Some(out_path) = out_file {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                        if let Some(data_url) = json["data"]["data_url"].as_str() {
                            if let Some(b64) = data_url.split(',').nth(1) {
                                if let Ok(bytes) = Self::base64_decode(b64) {
                                    if std::fs::write(&out_path, bytes).is_ok() {
                                        println!("Canvas image saved to {}", out_path);
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }

                println!("{}", resp);
                true
            }

            "resource-timing" | "perf-resources" => {
                let initiator = args.iter()
                    .position(|a| a == "--initiator" || a == "-i")
                    .and_then(|idx| args.get(idx + 1));
                let max = args.iter()
                    .position(|a| a == "--max" || a == "-m")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());
                let min_dur = args.iter()
                    .position(|a| a == "--min-duration" || a == "-d")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<f64>().ok());

                let mut params = Vec::new();
                if let Some(init) = initiator {
                    params.push(format!("initiator={}", crate::url_utils::urlencoding_simple(init)));
                }
                if let Some(m) = max {
                    params.push(format!("max={}", m));
                }
                if let Some(d) = min_dur {
                    params.push(format!("min_duration={}", d));
                }

                let endpoint = if params.is_empty() {
                    "/performance/resources".to_string()
                } else {
                    format!("/performance/resources?{}", params.join("&"))
                };

                let resp = Self::send_get(port, &endpoint);
                println!("{}", resp);
                true
            }

            "dom-stats" | "stats-dom" => {
                let resp = Self::send_get(port, "/dom/stats");
                println!("{}", resp);
                true
            }

            "scroll-by" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent scroll-by <dx> <dy> [--smooth] [--port <port>]");
                    std::process::exit(1);
                }
                let dx = args[2].parse::<i32>().unwrap_or(0);
                let dy = args[3].parse::<i32>().unwrap_or(0);
                let smooth = args.iter().any(|a| a == "--smooth" || a == "-s");

                let payload = serde_json::json!({
                    "dx": dx,
                    "dy": dy,
                    "smooth": smooth
                });
                let resp = Self::send_post(port, "/scroll/by", &payload);
                println!("{}", resp);
                true
            }

            "scroll-container" => {
                if args.len() < 5 {
                    eprintln!("Usage: aerolith-agent scroll-container <target> <dx> <dy> [--smooth] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let dx = args[3].parse::<i32>().unwrap_or(0);
                let dy = args[4].parse::<i32>().unwrap_or(0);
                let smooth = args.iter().any(|a| a == "--smooth" || a == "-s");

                let payload = serde_json::json!({
                    "target": target,
                    "dx": dx,
                    "dy": dy,
                    "smooth": smooth
                });
                let resp = Self::send_post(port, "/scroll/container", &payload);
                println!("{}", resp);
                true
            }

            "select-text" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent select-text <target> [--start <n>] [--length <n>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let start = args.iter()
                    .position(|a| a == "--start" || a == "-s")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());
                let length = args.iter()
                    .position(|a| a == "--length" || a == "-l" || a == "--len")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());

                let mut payload = serde_json::json!({ "target": target });
                if let Some(s) = start {
                    payload["start"] = serde_json::json!(s);
                }
                if let Some(l) = length {
                    payload["length"] = serde_json::json!(l);
                }
                let resp = Self::send_post(port, "/element/select_text", &payload);
                println!("{}", resp);
                true
            }

            "links" | "extract-links" => {
                let max = args.iter()
                    .position(|a| a == "--max" || a == "-m")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| s.parse::<usize>().ok());
                let external = args.iter().any(|a| a == "--external" || a == "-e");
                let filter = args.iter()
                    .position(|a| a == "--filter" || a == "-f")
                    .and_then(|idx| args.get(idx + 1));

                let mut params = Vec::new();
                if let Some(m) = max {
                    params.push(format!("max={}", m));
                }
                if external {
                    params.push("external=1".to_string());
                }
                if let Some(f) = filter {
                    params.push(format!("filter={}", crate::url_utils::urlencoding_simple(f)));
                }

                let endpoint = if params.is_empty() {
                    "/links".to_string()
                } else {
                    format!("/links?{}", params.join("&"))
                };
                let resp = Self::send_get(port, &endpoint);
                println!("{}", resp);
                true
            }

            "element-state" | "el-state" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent element-state <target> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let endpoint = format!("/element/state?target={}", crate::url_utils::urlencoding_simple(target));
                let resp = Self::send_get(port, &endpoint);
                println!("{}", resp);
                true
            }

            "class-toggle" | "toggle-class" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent class-toggle <target> <class_name> [toggle|add|remove|contains] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let class_name = &args[3];
                let action = if args.len() >= 5 && !args[4].starts_with('-') {
                    args[4].as_str()
                } else {
                    "toggle"
                };

                let payload = serde_json::json!({
                    "target": target,
                    "class_name": class_name,
                    "action": action
                });
                let resp = Self::send_post(port, "/element/class", &payload);
                println!("{}", resp);
                true
            }

            "set-text" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent set-text <target> <text> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let text = &args[3];
                let payload = serde_json::json!({
                    "target": target,
                    "text": text
                });
                let resp = Self::send_post(port, "/element/text/set", &payload);
                println!("{}", resp);
                true
            }

            "dispatch-event" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent dispatch-event <target> <event_type> [--detail <json>] [--no-bubble] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let event_type = &args[3];
                let detail = args.iter()
                    .position(|a| a == "--detail" || a == "-d")
                    .and_then(|idx| args.get(idx + 1))
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
                let no_bubble = args.iter().any(|a| a == "--no-bubble");

                let mut payload = serde_json::json!({
                    "target": target,
                    "event_type": event_type,
                    "bubbles": !no_bubble,
                    "cancelable": true
                });
                if let Some(d) = detail {
                    payload["detail"] = d;
                }
                let resp = Self::send_post(port, "/element/dispatch_event", &payload);
                println!("{}", resp);
                true
            }

            "active-element" => {
                let resp = Self::send_get(port, "/active_element");
                println!("{}", resp);
                true
            }

            "clickable" | "check-clickable" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent clickable <target> [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let endpoint = format!("/element/clickable?target={}", crate::url_utils::urlencoding_simple(target));
                let resp = Self::send_get(port, &endpoint);
                println!("{}", resp);
                true
            }

            "wheel" => {
                if args.len() < 5 {
                    eprintln!("Usage: aerolith-agent wheel <target> <dx> <dy> [--ctrl] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let dx = args[3].parse::<f64>().unwrap_or(0.0);
                let dy = args[4].parse::<f64>().unwrap_or(0.0);
                let ctrl = args.iter().any(|a| a == "--ctrl" || a == "-c");

                let payload = serde_json::json!({
                    "target": target,
                    "delta_x": dx,
                    "delta_y": dy,
                    "ctrl_key": ctrl
                });
                let resp = Self::send_post(port, "/element/wheel", &payload);
                println!("{}", resp);
                true
            }

            "css-vars" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent css-vars <var1> [var2...] [--target <selector>] [--port <port>]");
                    std::process::exit(1);
                }
                let target = args.iter()
                    .position(|a| a == "--target" || a == "-t")
                    .and_then(|idx| args.get(idx + 1))
                    .cloned();

                let mut vars = Vec::new();
                let mut i = 2;
                while i < args.len() {
                    let a = &args[i];
                    if a == "--target" || a == "-t" || a == "--port" || a == "-p" {
                        i += 2;
                    } else if a.starts_with("--port=") {
                        i += 1;
                    } else {
                        vars.push(a.clone());
                        i += 1;
                    }
                }

                let mut payload = serde_json::json!({ "variables": vars });
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/css/variables", &payload);
                println!("{}", resp);
                true
            }

            "clear-highlights" => {
                let resp = Self::send_post(port, "/highlights/clear", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "pointer-click" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent pointer-click <target> [--count <n>] [--button <btn>]");
                    return true;
                }
                let target = &args[2];
                let mut count = 1usize;
                let mut button = 0i32;
                for i in 3..args.len() {
                    if (args[i] == "--count" || args[i] == "-n") && i + 1 < args.len() {
                        if let Ok(c) = args[i + 1].parse() {
                            count = c;
                        }
                    }
                    if args[i] == "--button" && i + 1 < args.len() {
                        if let Ok(b) = args[i + 1].parse() {
                            button = b;
                        }
                    }
                }
                let payload = serde_json::json!({
                    "target": target,
                    "click_count": count,
                    "button": button
                });
                let resp = Self::send_post(port, "/element/pointer_click", &payload);
                println!("{}", resp);
                true
            }

            "meta-tags" => {
                let no_json_ld = args.iter().any(|a| a == "--no-json-ld");
                let path = if no_json_ld { "/meta/tags?json_ld=false" } else { "/meta/tags" };
                let resp = Self::send_get(port, path);
                println!("{}", resp);
                true
            }

            "geometry" => {
                let resp = Self::send_get(port, "/document/geometry");
                println!("{}", resp);
                true
            }

            "orientation" => {
                if args.len() < 5 {
                    eprintln!("Usage: aerolith-agent orientation <alpha> <beta> <gamma> [--absolute]");
                    return true;
                }
                let alpha: f64 = args[2].parse().unwrap_or(0.0);
                let beta: f64 = args[3].parse().unwrap_or(0.0);
                let gamma: f64 = args[4].parse().unwrap_or(0.0);
                let absolute = args.iter().any(|a| a == "--absolute");
                let payload = serde_json::json!({
                    "alpha": alpha,
                    "beta": beta,
                    "gamma": gamma,
                    "absolute": absolute
                });
                let resp = Self::send_post(port, "/emulate/orientation", &payload);
                println!("{}", resp);
                true
            }

            "clear-cache" => {
                let clear_storage = args.iter().any(|a| a == "--storage");
                let clear_cookies = args.iter().any(|a| a == "--cookies");
                let payload = serde_json::json!({
                    "clear_storage": clear_storage,
                    "clear_cookies": clear_cookies
                });
                let resp = Self::send_post(port, "/cache/clear", &payload);
                println!("{}", resp);
                true
            }

            "element-spacing" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent element-spacing <target>");
                    return true;
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/element/spacing", &payload);
                println!("{}", resp);
                true
            }

            "wait-url" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent wait-url <pattern> [--timeout <ms>]");
                    return true;
                }
                let pattern = &args[2];
                let mut timeout = 5000u64;
                for i in 3..args.len() {
                    if (args[i] == "--timeout" || args[i] == "-t") && i + 1 < args.len() {
                        if let Ok(t) = args[i + 1].parse() {
                            timeout = t;
                        }
                    }
                }
                let payload = serde_json::json!({
                    "pattern": pattern,
                    "timeout_ms": timeout
                });
                let resp = Self::send_post(port, "/wait_for_url", &payload);
                println!("{}", resp);
                true
            }

            "touch" => {
                if args.len() < 5 {
                    eprintln!("Usage: aerolith-agent touch <target> <x> <y> [--type <touchstart|touchmove|touchend>]");
                    return true;
                }
                let target = &args[2];
                let x: f64 = args[3].parse().unwrap_or(0.0);
                let y: f64 = args[4].parse().unwrap_or(0.0);
                let mut event_type = "touchstart".to_string();
                for i in 5..args.len() {
                    if (args[i] == "--type" || args[i] == "-t") && i + 1 < args.len() {
                        event_type = args[i + 1].clone();
                    }
                }
                let payload = serde_json::json!({
                    "event_type": event_type,
                    "touches": [
                        {
                            "target": target,
                            "x": x,
                            "y": y
                        }
                    ]
                });
                let resp = Self::send_post(port, "/touch", &payload);
                println!("{}", resp);
                true
            }

            "a11y-tree" => {
                let mut target = None;
                let mut depth = 5usize;
                let mut i = 2;
                while i < args.len() {
                    if (args[i] == "--depth" || args[i] == "-d") && i + 1 < args.len() {
                        depth = args[i + 1].parse().unwrap_or(5);
                        i += 2;
                    } else if args[i].starts_with("--port") {
                        i += 2;
                    } else if !args[i].starts_with('-') && target.is_none() {
                        target = Some(args[i].clone());
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                let mut payload = serde_json::json!({ "max_depth": depth });
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/a11y", &payload);
                println!("{}", resp);
                true
            }

            "chord" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent chord <keys> [<target>]");
                    return true;
                }
                let chord = &args[2];
                let target = args.get(3).filter(|s| !s.starts_with('-')).cloned();
                let mut payload = serde_json::json!({ "chord": chord });
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/keyboard/chord", &payload);
                println!("{}", resp);
                true
            }

            "boxes" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent boxes '<json_boxes>'");
                    return true;
                }
                let raw_json = &args[2];
                let boxes_val: serde_json::Value = serde_json::from_str(raw_json).unwrap_or(serde_json::json!([]));
                let payload = serde_json::json!({ "boxes": boxes_val });
                let resp = Self::send_post(port, "/highlights/boxes", &payload);
                println!("{}", resp);
                true
            }

            "element-attrs" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent element-attrs <target>");
                    return true;
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/element/attributes", &payload);
                println!("{}", resp);
                true
            }

            "filter" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent filter <target>");
                    return true;
                }
                let target = &args[2];
                let payload = serde_json::json!({ "target": target });
                let resp = Self::send_post(port, "/element/filter", &payload);
                println!("{}", resp);
                true
            }

            "dismiss-modals" => {
                let resp = Self::send_post(port, "/modals/dismiss", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "palette" => {
                let mut limit = 15;
                let mut i = 2;
                while i < args.len() {
                    if args[i] == "--limit" && i + 1 < args.len() {
                        limit = args[i + 1].parse().unwrap_or(15);
                        i += 1;
                    }
                    i += 1;
                }
                let payload = serde_json::json!({ "limit": limit });
                let resp = Self::send_post(port, "/palette", &payload);
                println!("{}", resp);
                true
            }

            "drag-drop" => {
                if args.len() < 4 {
                    eprintln!("Usage: aerolith-agent drag-drop <source> <target> [--steps <n>] [--hold <ms>]");
                    return true;
                }
                let source = &args[2];
                let target = &args[3];
                let mut steps = 10;
                let mut hold_ms = 0;
                let mut i = 4;
                while i < args.len() {
                    if args[i] == "--steps" && i + 1 < args.len() {
                        steps = args[i + 1].parse().unwrap_or(10);
                        i += 1;
                    } else if args[i] == "--hold" && i + 1 < args.len() {
                        hold_ms = args[i + 1].parse().unwrap_or(0);
                        i += 1;
                    }
                    i += 1;
                }
                let payload = serde_json::json!({
                    "source": source,
                    "target": target,
                    "steps": steps,
                    "hold_ms": hold_ms
                });
                let resp = Self::send_post(port, "/drag_and_drop", &payload);
                println!("{}", resp);
                true
            }

            "audit-forms" => {
                let target = if args.len() >= 3 && !args[2].starts_with("--") {
                    Some(&args[2])
                } else {
                    None
                };
                let mut payload = serde_json::json!({});
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/forms/audit", &payload);
                println!("{}", resp);
                true
            }

            "paging" => {
                let resp = Self::send_post(port, "/viewport/paging", &serde_json::json!({}));
                println!("{}", resp);
                true
            }

            "pseudo" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent pseudo <target> [pseudo] [--props <p1,p2...>]");
                    return true;
                }
                let target = &args[2];
                let pseudo = if args.len() >= 4 && !args[3].starts_with("--") {
                    &args[3]
                } else {
                    "::before"
                };
                let mut props: Vec<String> = vec![];
                let mut i = 3;
                while i < args.len() {
                    if args[i] == "--props" && i + 1 < args.len() {
                        props = args[i + 1].split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        i += 1;
                    }
                    i += 1;
                }
                let payload = serde_json::json!({
                    "target": target,
                    "pseudo": pseudo,
                    "properties": props
                });
                let resp = Self::send_post(port, "/element/pseudo", &payload);
                println!("{}", resp);
                true
            }

            "paste" => {
                if args.len() < 3 {
                    eprintln!("Usage: aerolith-agent paste <text> [--target <selector>]");
                    return true;
                }
                let text = &args[2];
                let mut target: Option<String> = None;
                let mut i = 3;
                while i < args.len() {
                    if args[i] == "--target" && i + 1 < args.len() {
                        target = Some(args[i + 1].clone());
                        i += 1;
                    }
                    i += 1;
                }
                let mut payload = serde_json::json!({ "text": text });
                if let Some(t) = target {
                    payload["target"] = serde_json::json!(t);
                }
                let resp = Self::send_post(port, "/clipboard/paste", &payload);
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

    fn send_delete(port: u16, path: &str) -> String {
        Self::send_http(port, "DELETE", path, None)
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

        let args4 = vec!["aerolith-agent".into(), "hover".into(), "nav".into(), "-p".into(), "7777".into()];
        assert_eq!(AgentCli::extract_port(&args4), Some(7777));
    }

    #[test]
    fn test_base64_decode() {
        // "Hello World" in base64 is "SGVsbG8gV29ybGQ="
        let decoded = AgentCli::base64_decode("SGVsbG8gV29ybGQ=").unwrap();
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");

        let empty = AgentCli::base64_decode("").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_handle_client_subcommand_non_client() {
        let args = vec!["aerolith-agent".into()];
        assert!(!AgentCli::handle_client_subcommand(&args));

        let args_unknown = vec!["aerolith-agent".into(), "unknown_action".into()];
        assert!(!AgentCli::handle_client_subcommand(&args_unknown));
    }
}

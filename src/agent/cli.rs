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
                    eprintln!("Usage: aerolith-agent screenshot-element <ref_or_selector> [out.png] [--port <port>]");
                    std::process::exit(1);
                }
                let target = &args[2];
                let path = args.iter().skip(3).find(|a| a.ends_with(".png")).cloned();
                let endpoint = format!("/screenshot/element?target={}", crate::url_utils::urlencoding_simple(target));
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

            "emulate" | "viewport" => {
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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod adblock;
mod agent;
mod geometry;
mod governor;
mod paths;
mod storage;
mod tabs;
mod types;
mod ui;
mod url_utils;
mod webview_factory;

use adblock::AdBlocker;
use agent::cli::AgentCli;
use agent::mcp::McpServer;
use agent::protocol::{AgentCommand, AgentResultEnvelope};
use agent::server::AgentServer;
use agent::PendingAgentRequests;
use geometry::{chrome_expanded_height, content_rect, physical_chrome_height, reassert_chrome_bounds};
use governor::MemoryGovernor;
use paths::{dirs_storage_path, dirs_webview_data_path, launch_touch_keyboard, show_fatal_error};
use storage::BrowserStorage;
use tabs::{create_tab, TabManager};
use types::{ChromeMsg, UserEvent, DEFAULT_HEIGHT, DEFAULT_WIDTH, HOME_URL};
use url_utils::{clean_js_json, sanitize_title};
use webview_factory::create_chrome_webview;

#[cfg(all(target_os = "windows", target_env = "gnu"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn RoOriginateErrorW(_error: i32, _cchmax: u32, _message: *const u16) -> i32 {
    0
}

use std::path::PathBuf;
use tao::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window, WindowBuilder},
};
use wry::{Rect, WebView};

fn handle_chrome_msg(
    msg: ChromeMsg,
    tab_mgr: &mut TabManager,
    chrome_wv: &WebView,
    window: &Window,
    proxy: &EventLoopProxy<UserEvent>,
    adblocker: &mut AdBlocker,
    storage: &BrowserStorage,
    webview_data_dir: &PathBuf,
    chrome_expansion_mode: &mut String,
    current_scale_factor: f64,
    current_size: PhysicalSize<u32>,
) {
    let bounds = content_rect(current_size, current_scale_factor);

    match msg.action.as_str() {
        "chrome_ready" => {
            tab_mgr.sync_tabs_to_chrome(chrome_wv);
            tab_mgr.sync_address_to_chrome(chrome_wv, storage);
            let _ = chrome_wv.evaluate_script(&format!("window.updateShortFormState({});", adblocker.block_short_form));
            let engine = storage.get_setting_string("search_engine", "duckduckgo");
            let clear = storage.get_setting_bool("clear_on_exit", false);
            let free_mb = MemoryGovernor::get_available_mb();
            let load_pct = MemoryGovernor::get_memory_load_pct();
            let script = format!(
                "window.updateSettingsData('{}', {}, {}, {});",
                engine, clear, free_mb, load_pct
            );
            let _ = chrome_wv.evaluate_script(&script);
        }

        "navigate" => {
            if let Some(payload) = msg.payload {
                if let Some(raw_url) = payload["url"].as_str() {
                    tab_mgr.navigate(raw_url, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "switch_tab" => {
            if let Some(payload) = msg.payload {
                if let Some(target_idx) = payload["index"].as_u64() {
                    tab_mgr.switch_tab(target_idx as usize, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "new_tab" => {
            let raw_url = msg.payload
                .as_ref()
                .and_then(|p| p["url"].as_str())
                .unwrap_or(HOME_URL);
            tab_mgr.new_tab(raw_url, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
            reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
        }

        "close_tab" => {
            if let Some(payload) = msg.payload {
                if let Some(close_idx) = payload["index"].as_u64() {
                    tab_mgr.close_tab(close_idx as usize, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "reopen_closed_tab" => {
            tab_mgr.reopen_closed_tab(window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
            reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
        }

        "duplicate_tab" => {
            if let Some(payload) = &msg.payload {
                if let Some(idx) = payload["index"].as_u64() {
                    tab_mgr.duplicate_tab(idx as usize, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "reload_tab" => {
            if let Some(payload) = &msg.payload {
                if let Some(idx) = payload["index"].as_u64() {
                    if let Some(tab) = tab_mgr.get(idx as usize) {
                        if let Some(wv) = &tab.webview {
                            let _ = wv.evaluate_script("window.location.reload()");
                        }
                    }
                }
            }
        }

        "close_other_tabs" => {
            if let Some(payload) = &msg.payload {
                if let Some(idx) = payload["index"].as_u64() {
                    tab_mgr.close_other_tabs(
                        idx as usize,
                        window,
                        proxy,
                        bounds,
                        adblocker,
                        webview_data_dir,
                        chrome_wv,
                        storage,
                    );
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "close_tabs_to_right" => {
            if let Some(payload) = &msg.payload {
                if let Some(idx) = payload["index"].as_u64() {
                    tab_mgr.close_tabs_to_right(
                        idx as usize,
                        window,
                        proxy,
                        bounds,
                        adblocker,
                        webview_data_dir,
                        chrome_wv,
                        storage,
                    );
                    reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
                }
            }
        }

        "toggle_mute" => {
            if let Some(payload) = &msg.payload {
                if let Some(target_idx) = payload["index"].as_u64() {
                    tab_mgr.toggle_mute(target_idx as usize, chrome_wv);
                }
            }
        }

        "back" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Some(wv) = &tab.webview {
                    let _ = wv.go_back();
                }
            }
        }

        "forward" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Some(wv) = &tab.webview {
                    let _ = wv.go_forward();
                }
            }
        }

        "reload" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script("window.location.reload()");
                }
            }
        }

        "stop" => {
            if let Some(tab) = tab_mgr.active_tab_mut() {
                tab.is_loading = false;
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script("window.stop();");
                }
                let _ = chrome_wv.evaluate_script("window.setLoading(false);");
            }
        }

        "home" => {
            tab_mgr.navigate(HOME_URL, window, proxy, bounds, adblocker, webview_data_dir, chrome_wv, storage);
            reassert_chrome_bounds(chrome_wv, chrome_expansion_mode, current_scale_factor, current_size);
        }

        "toggle_fullscreen" => {
            let is_fs = window.fullscreen().is_some();
            if is_fs {
                window.set_fullscreen(None);
            } else {
                window.set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)));
            }
        }

        "launch_keyboard" => {
            launch_touch_keyboard();
        }

        "toggle_bookmark" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Ok(is_bm) = storage.toggle_bookmark(&tab.url, &tab.title) {
                    let (can_back, can_fwd) = TabManager::tab_nav_state(tab);
                    let url_json = clean_js_json(&serde_json::to_string(&tab.url).unwrap_or_else(|_| "\"\"".to_string()));
                    let title_json = clean_js_json(&serde_json::to_string(&tab.title).unwrap_or_else(|_| "\"\"".to_string()));
                    let script = format!(
                        "window.updateAddress({}, {}, {}, {}, {}, {});",
                        url_json, title_json, can_back, can_fwd, is_bm, tab.blocked_count
                    );
                    let _ = chrome_wv.evaluate_script(&script);
                }
            }
        }

        "get_bookmarks" => {
            let bookmarks = storage.get_bookmarks();
            if let Ok(json) = serde_json::to_string(&bookmarks) {
                let script = format!("window.showDrawerData({}, 'bookmarks');", clean_js_json(&json));
                let _ = chrome_wv.evaluate_script(&script);
            }
        }

        "delete_bookmark" => {
            if let Some(payload) = &msg.payload {
                if let Some(url) = payload["url"].as_str() {
                    let _ = storage.delete_bookmark(url);
                    let bookmarks = storage.get_bookmarks();
                    if let Ok(json) = serde_json::to_string(&bookmarks) {
                        let script = format!("window.showDrawerData({}, 'bookmarks');", clean_js_json(&json));
                        let _ = chrome_wv.evaluate_script(&script);
                    }
                    if let Some(tab) = tab_mgr.active_tab() {
                        if tab.url == url {
                            tab_mgr.sync_address_to_chrome(chrome_wv, storage);
                        }
                    }
                }
            }
        }

        "get_history" => {
            let history = storage.get_recent_history(50);
            if let Ok(json) = serde_json::to_string(&history) {
                let script = format!("window.showDrawerData({}, 'history');", clean_js_json(&json));
                let _ = chrome_wv.evaluate_script(&script);
            }
        }

        "delete_history_entry" => {
            if let Some(payload) = &msg.payload {
                if let Some(url) = payload["url"].as_str() {
                    let _ = storage.delete_history_item(url);
                    let history = storage.get_recent_history(50);
                    if let Ok(json) = serde_json::to_string(&history) {
                        let script = format!("window.showDrawerData({}, 'history');", clean_js_json(&json));
                        let _ = chrome_wv.evaluate_script(&script);
                    }
                }
            }
        }

        "clear_history" => {
            let _ = storage.clear_history();
            let script = "window.showDrawerData([], 'history');";
            let _ = chrome_wv.evaluate_script(script);
        }

        "search_query" => {
            if let Some(payload) = &msg.payload {
                if let Some(query) = payload["query"].as_str() {
                    let results = storage.search(query, 6);
                    if let Ok(json) = serde_json::to_string(&results) {
                        let q_json = clean_js_json(&serde_json::to_string(query).unwrap_or_else(|_| "\"\"".to_string()));
                        let script = format!("window.showSearchResults({}, {});", clean_js_json(&json), q_json);
                        let _ = chrome_wv.evaluate_script(&script);
                    }
                }
            }
        }

        "toggle_shield" => {
            adblocker.enabled = !adblocker.enabled;
            let shield_script = AdBlocker::get_shield_toggle_script(adblocker.enabled);
            for tab in &tab_mgr.tabs {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script(shield_script);
                }
            }
            let script = format!("window.updateShieldState({});", adblocker.enabled);
            let _ = chrome_wv.evaluate_script(&script);
        }

        "toggle_short_form" => {
            adblocker.block_short_form = !adblocker.block_short_form;
            let _ = storage.set_setting_bool("block_short_form", adblocker.block_short_form);
            let script = AdBlocker::get_short_form_toggle_script(adblocker.block_short_form);
            for tab in &tab_mgr.tabs {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script(script);
                }
            }
            let chrome_script = format!("window.updateShortFormState({});", adblocker.block_short_form);
            let _ = chrome_wv.evaluate_script(&chrome_script);
        }

        "toggle_reader" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script(AdBlocker::get_reader_mode_script());
                }
            }
        }

        "toggle_dark_mode" => {
            let enabled = msg.payload
                .as_ref()
                .and_then(|p| p["enabled"].as_bool())
                .unwrap_or(false);
            adblocker.dark_mode = enabled;
            let script = AdBlocker::get_dark_mode_script(enabled);
            for tab in &tab_mgr.tabs {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script(script);
                }
            }
        }

        "find_in_page" => {
            if let Some(payload) = &msg.payload {
                let query = payload["query"].as_str().unwrap_or("");
                let backward = payload["backward"].as_bool().unwrap_or(false);
                tab_mgr.find_in_page(query, backward);
            }
        }

        "zoom_in" => {
            tab_mgr.zoom_in();
        }

        "zoom_out" => {
            tab_mgr.zoom_out();
        }

        "zoom_reset" => {
            tab_mgr.zoom_reset();
        }

        "set_chrome_expanded" => {
            if let Some(payload) = &msg.payload {
                let mode = payload["mode"].as_str().unwrap_or("none");
                *chrome_expansion_mode = mode.to_string();
                let h = chrome_expanded_height(mode, current_scale_factor, current_size.height);
                let _ = chrome_wv.set_bounds(Rect {
                    position: PhysicalPosition::new(0, 0).into(),
                    size: PhysicalSize::new(current_size.width, h).into(),
                });
                if mode == "full" {
                    if let Some(tab) = tab_mgr.active_tab() {
                        if let Some(wv) = &tab.webview {
                            let _ = wv.set_visible(false);
                        }
                    }
                } else {
                    if let Some(tab) = tab_mgr.active_tab() {
                        if let Some(wv) = &tab.webview {
                            let _ = wv.set_visible(true);
                        }
                    }
                }
            }
        }

        "hibernate_all" => {
            tab_mgr.hibernate_all_background(chrome_wv);
            let free_mb = MemoryGovernor::get_available_mb();
            let load_pct = MemoryGovernor::get_memory_load_pct();
            let engine = storage.get_setting_string("search_engine", "duckduckgo");
            let clear = storage.get_setting_bool("clear_on_exit", false);
            let script = format!(
                "window.updateSettingsData('{}', {}, {}, {});",
                engine, clear, free_mb, load_pct
            );
            let _ = chrome_wv.evaluate_script(&script);
        }

        "print" => {
            if let Some(tab) = tab_mgr.active_tab() {
                if let Some(wv) = &tab.webview {
                    let _ = wv.evaluate_script("window.print();");
                }
            }
        }

        "set_search_engine" => {
            if let Some(payload) = &msg.payload {
                if let Some(engine) = payload["engine"].as_str() {
                    let _ = storage.set_setting_string("search_engine", engine);
                }
            }
        }

        "set_clear_on_exit" => {
            if let Some(payload) = &msg.payload {
                if let Some(enabled) = payload["enabled"].as_bool() {
                    let _ = storage.set_setting_bool("clear_on_exit", enabled);
                }
            }
        }

        "get_settings" => {
            let engine = storage.get_setting_string("search_engine", "duckduckgo");
            let clear = storage.get_setting_bool("clear_on_exit", false);
            let free_mb = MemoryGovernor::get_available_mb();
            let load_pct = MemoryGovernor::get_memory_load_pct();
            let script = format!(
                "window.updateSettingsData('{}', {}, {}, {});",
                engine, clear, free_mb, load_pct
            );
            let _ = chrome_wv.evaluate_script(&script);
        }

        _ => ()
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy: EventLoopProxy<UserEvent> = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("Aerolith Agent Browser")
        .with_inner_size(PhysicalSize::new(DEFAULT_WIDTH, DEFAULT_HEIGHT))
        .build(&event_loop)?;

    let pending_agent_requests = std::sync::Arc::new(PendingAgentRequests::new());

    // Configure Chrome DevTools Protocol (CDP) remote debugging port for WebView2
    let no_cdp = std::env::args().any(|a| a == "--no-cdp");
    let cdp_port = std::env::args()
        .position(|a| a == "--cdp-port")
        .and_then(|idx| std::env::args().nth(idx + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(9222);

    if !no_cdp {
        let current_args = std::env::var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").unwrap_or_default();
        if !current_args.contains("--remote-debugging-port") {
            let cdp_flag = format!("--remote-debugging-port={}", cdp_port);
            let combined = if current_args.is_empty() {
                cdp_flag
            } else {
                format!("{} {}", current_args, cdp_flag)
            };
            unsafe {
                std::env::set_var("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", combined);
            }
        }
        println!("[AerolithAgent] CDP DevTools enabled on http://127.0.0.1:{}", cdp_port);
    }

    // Start background agent HTTP server
    let agent_port = std::env::args()
        .position(|a| a == "--port" || a == "-p")
        .and_then(|idx| std::env::args().nth(idx + 1))
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8765);

    if let Ok(bound_port) = AgentServer::start(agent_port, cdp_port, proxy.clone()) {
        println!("[AerolithAgent] Server listening on http://127.0.0.1:{}", bound_port);
    }

    // Start MCP stdio server if --mcp or mcp argument is present
    let is_mcp = std::env::args().any(|a| a == "--mcp" || a == "mcp");
    if is_mcp {
        let mcp_proxy = proxy.clone();
        std::thread::Builder::new()
            .name("aerolith-mcp-server".into())
            .spawn(move || {
                McpServer::run_stdio(mcp_proxy);
            })?;
    }

    // Database, settings and adblocker setup
    let db_path = dirs_storage_path();
    let storage = BrowserStorage::new(&db_path)?;
    let mut adblocker = AdBlocker::new();
    adblocker.block_short_form = storage.get_setting_bool("block_short_form", true);

    let mut current_scale_factor = window.scale_factor();
    let mut current_size = window.inner_size();
    let initial_chrome_h = physical_chrome_height(current_scale_factor);
    let initial_chrome_rect = Rect {
        position: PhysicalPosition::new(0, 0).into(),
        size: PhysicalSize::new(current_size.width, initial_chrome_h).into(),
    };

    // 1. Build Chrome UI WebView (Pinned to top: 0..physical_chrome_height)
    let (chrome_wv, webview_data_dir) = create_chrome_webview(
        &window,
        &proxy,
        initial_chrome_rect,
        dirs_webview_data_path(),
    )?;

    // 2. Initialize TabManager and restore session
    let mut tab_mgr = TabManager::new();
    let governor = MemoryGovernor::new();
    let mut chrome_expansion_mode = "none".to_string();

    let session = storage.load_session();
    let (initial_urls, initial_active_idx) = match session {
        Some(s) if !s.tabs.is_empty() => (s.tabs, s.active_index),
        _ => (vec![HOME_URL.to_string()], 0),
    };

    let c_rect = content_rect(current_size, current_scale_factor);
    for u in &initial_urls {
        let t_id = tab_mgr.next_id;
        tab_mgr.next_id += 1;
        let effective_url = if adblocker.block_short_form && AdBlocker::is_short_form_url(u) {
            AdBlocker::sanitize_short_form_url(u).unwrap_or_else(|| u.clone())
        } else {
            u.clone()
        };
        if let Ok(t) = create_tab(t_id, &effective_url, &window, &proxy, c_rect, &adblocker, &webview_data_dir) {
            tab_mgr.tabs.push(t);
        }
    }
    if tab_mgr.is_empty() {
        let initial_tab = create_tab(
            tab_mgr.next_id,
            HOME_URL,
            &window,
            &proxy,
            c_rect,
            &adblocker,
            &webview_data_dir,
        )?;
        tab_mgr.tabs.push(initial_tab);
        tab_mgr.next_id += 1;
    }

    tab_mgr.active_index = if initial_active_idx < tab_mgr.len() {
        initial_active_idx
    } else {
        tab_mgr.len() - 1
    };

    for (idx, tab) in tab_mgr.tabs.iter_mut().enumerate() {
        if let Some(wv) = &tab.webview {
            let _ = wv.set_bounds(content_rect(current_size, current_scale_factor));
            let _ = wv.set_visible(idx == tab_mgr.active_index);
        }
    }
    reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);

    // Run Event Loop with 5-second Memory Governor wakeups
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_secs(5));

        match event {
            Event::NewEvents(StartCause::Init) => {
                tab_mgr.sync_tabs_to_chrome(&chrome_wv);
                tab_mgr.sync_address_to_chrome(&chrome_wv, &storage);
            }

            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                tab_mgr.apply_governor_tick(governor.is_memory_pressure_high(), &chrome_wv);
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let clear_on_exit = storage.get_setting_bool("clear_on_exit", false);
                if clear_on_exit {
                    let _ = storage.clear_history();
                    let _ = storage.clear_session();
                } else {
                    tab_mgr.save_session(&storage);
                }
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { scale_factor, new_inner_size },
                ..
            } => {
                current_scale_factor = scale_factor;
                current_size = *new_inner_size;
                reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
                if let Some(tab) = tab_mgr.active_tab_mut() {
                    if let Some(wv) = &tab.webview {
                        let _ = wv.set_bounds(content_rect(current_size, current_scale_factor));
                    }
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                if new_size.width > 0 && new_size.height > 0 {
                    current_size = new_size;
                    reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
                    if let Some(tab) = tab_mgr.active_tab_mut() {
                        if let Some(wv) = &tab.webview {
                            let _ = wv.set_bounds(content_rect(new_size, current_scale_factor));
                        }
                    }
                }
            }

            Event::UserEvent(UserEvent::MediaStateChanged { tab_id, playing }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.is_audio_playing = playing;
                }
                tab_mgr.sync_tabs_to_chrome(&chrome_wv);
            }

            Event::UserEvent(UserEvent::FormDirtyChanged { tab_id, dirty }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.has_form_input = dirty;
                }
            }

            Event::UserEvent(UserEvent::ScrollChanged { tab_id, scroll_x, scroll_y }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.scroll_x = scroll_x;
                    tab.scroll_y = scroll_y;
                }
            }

            Event::UserEvent(UserEvent::LoadingState { tab_id, loading }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.is_loading = loading;
                }
                if let Some(active_tab) = tab_mgr.active_tab() {
                    if active_tab.id == tab_id {
                        let script = format!("window.setLoading({});", loading);
                        let _ = chrome_wv.evaluate_script(&script);
                    }
                }
            }

            Event::UserEvent(UserEvent::ReaderStateChanged { tab_id, active }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.is_reader_active = active;
                }
                if let Some(active_tab) = tab_mgr.active_tab() {
                    if active_tab.id == tab_id {
                        let script = format!("window.setReaderState({});", active);
                        let _ = chrome_wv.evaluate_script(&script);
                    }
                }
            }

            Event::UserEvent(UserEvent::TitleChanged { tab_id, title }) => {
                let sanitized_title = sanitize_title(&title);
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    if !sanitized_title.is_empty() {
                        tab.title = sanitized_title.clone();
                    }
                    tab.last_active = std::time::Instant::now();
                }
                if let Some(active_tab) = tab_mgr.active_tab() {
                    if active_tab.id == tab_id {
                        tab_mgr.sync_address_to_chrome(&chrome_wv, &storage);
                        let _ = storage.record_visit(&active_tab.url, &sanitized_title);
                    }
                }
                tab_mgr.sync_tabs_to_chrome(&chrome_wv);
            }

            Event::UserEvent(UserEvent::UrlChanged { tab_id, url }) => {
                let stripped_url = crate::url_utils::strip_tracking_parameters(&url);
                let was_param_stripped = stripped_url != url;
                let was_short_form = adblocker.block_short_form && AdBlocker::is_short_form_url(&stripped_url);
                let was_sanitized = was_short_form || was_param_stripped;
                let effective_url = if was_short_form {
                    AdBlocker::sanitize_short_form_url(&stripped_url).unwrap_or(stripped_url)
                } else {
                    stripped_url
                };

                let is_about_home = if let Some(tab) = tab_mgr.tabs.iter().find(|t| t.id == tab_id) {
                    tab.url == HOME_URL && (effective_url == "about:blank" || effective_url == HOME_URL)
                } else {
                    false
                };

                let needs_redirect = !is_about_home && was_sanitized && effective_url != url && !effective_url.is_empty();

                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    if !effective_url.is_empty() && !is_about_home {
                        tab.url = effective_url.clone();
                    }
                    tab.last_active = std::time::Instant::now();
                    tab.is_audio_playing = false;
                    tab.has_form_input = false;
                    tab.is_reader_active = false;
                    if let Some(wv) = &tab.webview {
                        if needs_redirect {
                            let _ = wv.load_url(&effective_url);
                        }
                        if adblocker.dark_mode {
                            let _ = wv.evaluate_script(AdBlocker::get_dark_mode_script(true));
                        }
                        if adblocker.enabled {
                            let _ = wv.evaluate_script(AdBlocker::get_shield_toggle_script(true));
                        }
                        if adblocker.block_short_form {
                            let _ = wv.evaluate_script(AdBlocker::get_short_form_toggle_script(true));
                        }
                        if tab.scroll_x > 0 || tab.scroll_y > 0 {
                            let _ = wv.evaluate_script(&AdBlocker::get_tab_restore_script(tab.scroll_x, tab.scroll_y));
                        }
                    }
                }
                if let Some(active_tab) = tab_mgr.active_tab() {
                    if active_tab.id == tab_id {
                        tab_mgr.sync_address_to_chrome(&chrome_wv, &storage);
                        let _ = chrome_wv.evaluate_script("window.setReaderState(false);");
                        if !is_about_home {
                            let _ = storage.record_visit(&effective_url, &active_tab.title);
                        }
                        tab_mgr.save_session(&storage);
                    }
                }
                tab_mgr.sync_tabs_to_chrome(&chrome_wv);
            }

            Event::UserEvent(UserEvent::NewWindow(target_url)) => {
                let clean_url = crate::url_utils::strip_tracking_parameters(&target_url);
                let bounds = content_rect(current_size, current_scale_factor);
                tab_mgr.new_tab(
                    &clean_url,
                    &window,
                    &proxy,
                    bounds,
                    &adblocker,
                    &webview_data_dir,
                    &chrome_wv,
                    &storage,
                );
                reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
            }

            Event::UserEvent(UserEvent::Chrome(msg)) => {
                handle_chrome_msg(
                    msg,
                    &mut tab_mgr,
                    &chrome_wv,
                    &window,
                    &proxy,
                    &mut adblocker,
                    &storage,
                    &webview_data_dir,
                    &mut chrome_expansion_mode,
                    current_scale_factor,
                    current_size,
                );
            }

            Event::UserEvent(UserEvent::AdBlocked { tab_id }) => {
                if let Some(tab) = tab_mgr.find_by_id_mut(tab_id) {
                    tab.blocked_count += 1;
                }
                if let Some(active_tab) = tab_mgr.active_tab() {
                    if active_tab.id == tab_id {
                        tab_mgr.sync_address_to_chrome(&chrome_wv, &storage);
                    }
                }
                tab_mgr.sync_tabs_to_chrome(&chrome_wv);
            }

            Event::UserEvent(UserEvent::AgentAction { command, response_tx }) => {
                match command {
                    AgentCommand::Navigate { url } => {
                        let bounds = content_rect(current_size, current_scale_factor);
                        tab_mgr.navigate(&url, &window, &proxy, bounds, &adblocker, &webview_data_dir, &chrome_wv, &storage);
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                            "status": "navigated",
                            "url": url
                        })));
                    }

                    AgentCommand::Back => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let _ = wv.go_back();
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "navigated_back" })));
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for back"));
                    }

                    AgentCommand::Forward => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let _ = wv.go_forward();
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "navigated_forward" })));
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for forward"));
                    }

                    AgentCommand::Reload => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let _ = wv.evaluate_script("window.location.reload()");
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "reloaded" })));
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for reload"));
                    }

                    AgentCommand::GetText { target, attribute } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_text_script(req_id, &target, attribute.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for get_text"));
                    }

                    AgentCommand::Snapshot { viewport_only, filter_role } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_semantic_dom_script(req_id, viewport_only, filter_role.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for snapshot"));
                    }

                    AgentCommand::Highlight { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_highlight_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for highlight"));
                    }

                    AgentCommand::ScreenshotElement { target, padding } => {
                        let eval_params = serde_json::json!({
                            "expression": format!(
                                r#"(function() {{
                                    const target = {};
                                    const isNum = /^\d+$/.test(target);
                                    const el = isNum ? (window.__aerolith_elements && window.__aerolith_elements[target] || document.querySelector('[data-aerolith-ref="' + target + '"]')) : document.querySelector(target);
                                    if (!el) return null;
                                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                                    const r = el.getBoundingClientRect();
                                    return {{ x: Math.max(0, Math.round(r.x)), y: Math.max(0, Math.round(r.y)), width: Math.max(1, Math.round(r.width)), height: Math.max(1, Math.round(r.height)) }};
                                }})()"#,
                                serde_json::to_string(&target).unwrap_or_default()
                            ),
                            "returnByValue": true
                        });

                        match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Runtime.evaluate", Some(eval_params)) {
                            Ok(eval_res) => {
                                let bounds = eval_res.get("result").and_then(|r| r.get("value")).cloned();
                                if let Some(b) = bounds {
                                    if b.is_null() {
                                        let _ = response_tx.send(AgentResultEnvelope::err(format!("Element not found for target: {}", target)));
                                        return;
                                    }
                                    let pad = padding as i64;
                                    let x = (b["x"].as_i64().unwrap_or(0) - pad).max(0);
                                    let y = (b["y"].as_i64().unwrap_or(0) - pad).max(0);
                                    let width = b["width"].as_i64().unwrap_or(100) + (pad * 2);
                                    let height = b["height"].as_i64().unwrap_or(30) + (pad * 2);

                                    let shot_params = serde_json::json!({
                                        "clip": {
                                            "x": x,
                                            "y": y,
                                            "width": width,
                                            "height": height,
                                            "scale": 1
                                        },
                                        "format": "png"
                                    });

                                    match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Page.captureScreenshot", Some(shot_params)) {
                                        Ok(shot_res) => {
                                            let b64 = shot_res.get("data").and_then(|d| d.as_str()).unwrap_or("");
                                            let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                                "status": "element_screenshot_ready",
                                                "target": target,
                                                "padding": padding,
                                                "bounds": b,
                                                "screenshot_base64": b64
                                            })));
                                        }
                                        Err(e) => {
                                            let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP screenshot failed: {}", e)));
                                        }
                                    }
                                } else {
                                    let _ = response_tx.send(AgentResultEnvelope::err("Failed to evaluate element bounds"));
                                }
                            }
                            Err(e) => {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP element evaluate failed: {}", e)));
                            }
                        }
                    }

                    AgentCommand::FillForm { fields, submit } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_fill_form_script(req_id, &fields, submit);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for fill_form"));
                    }

                    AgentCommand::Clipboard { action, text } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_clipboard_script(req_id, &action, text.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for clipboard"));
                    }

                    AgentCommand::Emulate { width, height, dpr, mobile } => {
                        let scale = dpr.unwrap_or(1.0);
                        let cdp_params = serde_json::json!({
                            "width": width,
                            "height": height,
                            "deviceScaleFactor": scale,
                            "mobile": mobile
                        });
                        let _ = crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Emulation.setDeviceMetricsOverride", Some(cdp_params));
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let resize_js = "window.dispatchEvent(new Event('resize'));";
                                let _ = wv.evaluate_script(resize_js);
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                            "status": "emulated",
                            "width": width,
                            "height": height,
                            "dpr": scale,
                            "mobile": mobile
                        })));
                    }

                    AgentCommand::ConsoleLogs { clear } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_console_logs_script(req_id, clear);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for console_logs"));
                    }

                    AgentCommand::PrintPdf { landscape } => {
                        let pdf_params = serde_json::json!({
                            "landscape": landscape,
                            "printBackground": true,
                            "preferCSSPageSize": true
                        });
                        match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Page.printToPDF", Some(pdf_params)) {
                            Ok(res) => {
                                let b64 = res.get("data").and_then(|d| d.as_str()).unwrap_or("");
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                    "status": "pdf_ready",
                                    "landscape": landscape,
                                    "pdf_base64": b64
                                })));
                            }
                            Err(e) => {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Page.printToPDF failed: {}", e)));
                            }
                        }
                    }

                    AgentCommand::Markdown => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_markdown_script(req_id);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for markdown"));
                    }

                    AgentCommand::Click { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_click_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for click"));
                    }

                    AgentCommand::Type { target, text, clear, submit } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_type_script(req_id, &target, &text, clear, submit);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for typing"));
                    }

                    AgentCommand::Scroll { direction, amount } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_scroll_script(req_id, &direction, amount);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for scroll"));
                    }

                    AgentCommand::Eval { expression } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_eval_script(req_id, &expression);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for eval"));
                    }

                    AgentCommand::WaitFor { target, timeout_ms } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_wait_for_script(req_id, &target, timeout_ms);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for wait_for"));
                    }

                    AgentCommand::StorageState { action, state } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = if action == "set" {
                                    let empty = serde_json::json!({});
                                    let st = state.as_ref().unwrap_or(&empty);
                                    crate::agent::AgentScripts::get_storage_import_script(req_id, st)
                                } else {
                                    crate::agent::AgentScripts::get_storage_export_script(req_id)
                                };
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for storage state"));
                    }

                    AgentCommand::Find { query } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_find_script(req_id, &query);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for find"));
                    }

                    AgentCommand::DialogPolicy { alert, confirm, prompt_value } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_dialog_policy_script(
                                    req_id,
                                    alert.as_deref(),
                                    confirm.as_deref(),
                                    prompt_value.as_deref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for dialog policy"));
                    }

                    AgentCommand::Hover { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_hover_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for hover"));
                    }

                    AgentCommand::PressKey { key, target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_press_key_script(req_id, &key, target.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for press_key"));
                    }

                    AgentCommand::SelectOption { target, value } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_select_option_script(req_id, &target, &value);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for select_option"));
                    }

                    AgentCommand::NetworkActivity { clear } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_network_activity_script(req_id, clear);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for network_activity"));
                    }

                    AgentCommand::Drag { source, target, offset_x, offset_y, steps } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_drag_script(
                                    req_id, &source, target.as_deref(), offset_x, offset_y, steps
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for drag"));
                    }

                    AgentCommand::Cookies { action, cookie, name, domain } => {
                        match action.as_str() {
                            "set" => {
                                let cdp_params = if let Some(c) = cookie {
                                    c
                                } else {
                                    serde_json::json!({
                                        "name": name.unwrap_or_default(),
                                        "value": "",
                                        "domain": domain.unwrap_or_default(),
                                    })
                                };
                                match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.setCookie", Some(cdp_params)) {
                                    Ok(res) => {
                                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                            "status": "cookie_set",
                                            "success": res.get("success").and_then(|s| s.as_bool()).unwrap_or(true),
                                            "result": res
                                        })));
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.setCookie failed: {}", e)));
                                    }
                                }
                            }
                            "delete" => {
                                let cdp_params = serde_json::json!({
                                    "name": name.unwrap_or_default(),
                                    "domain": domain
                                });
                                match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.deleteCookies", Some(cdp_params)) {
                                    Ok(_) => {
                                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                            "status": "cookie_deleted"
                                        })));
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.deleteCookies failed: {}", e)));
                                    }
                                }
                            }
                            "clear" => {
                                match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.clearBrowserCookies", None) {
                                    Ok(_) => {
                                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                            "status": "cookies_cleared"
                                        })));
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.clearBrowserCookies failed: {}", e)));
                                    }
                                }
                            }
                            _ => {
                                match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.getCookies", None) {
                                    Ok(res) => {
                                        let cookies = res.get("cookies").cloned().unwrap_or(serde_json::json!([]));
                                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                            "status": "cookies_retrieved",
                                            "cookies": cookies
                                        })));
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.getCookies failed: {}", e)));
                                    }
                                }
                            }
                        }
                    }

                    AgentCommand::ExtraHeaders { headers } => {
                        let cdp_params = serde_json::json!({ "headers": headers });
                        match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.setExtraHTTPHeaders", Some(cdp_params)) {
                            Ok(_) => {
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                    "status": "headers_configured",
                                    "headers": headers
                                })));
                            }
                            Err(e) => {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.setExtraHTTPHeaders failed: {}", e)));
                            }
                        }
                    }

                    AgentCommand::UserAgentOverride { user_agent, accept_language, platform } => {
                        let mut cdp_params = serde_json::json!({ "userAgent": user_agent });
                        if let Some(lang) = accept_language {
                            cdp_params["acceptLanguage"] = serde_json::json!(lang);
                        }
                        if let Some(plat) = platform {
                            cdp_params["platform"] = serde_json::json!(plat);
                        }
                        match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.setUserAgentOverride", Some(cdp_params)) {
                            Ok(_) => {
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                    "status": "user_agent_configured",
                                    "user_agent": user_agent
                                })));
                            }
                            Err(e) => {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.setUserAgentOverride failed: {}", e)));
                            }
                        }
                    }

                    AgentCommand::UploadFile { target, files } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_upload_file_script(req_id, &target, &files);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for upload_file"));
                    }

                    AgentCommand::Location { latitude, longitude, accuracy, timezone } => {
                        let acc = accuracy.unwrap_or(1.0);
                        let geo_params = serde_json::json!({
                            "latitude": latitude,
                            "longitude": longitude,
                            "accuracy": acc
                        });
                        let _ = crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Emulation.setGeolocationOverride", Some(geo_params));
                        if let Some(ref tz) = timezone {
                            let tz_params = serde_json::json!({ "timezoneId": tz });
                            let _ = crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Emulation.setTimezoneOverride", Some(tz_params));
                        }
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_location_script(req_id, latitude, longitude, acc);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for location"));
                    }

                    AgentCommand::MediaControl { action } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_media_control_script(req_id, &action);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for media_control"));
                    }

                    AgentCommand::Inspect { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_inspect_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for inspect"));
                    }

                    AgentCommand::RouteRule { action, pattern, rule_type, status, body, headers } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_route_rule_script(
                                    req_id,
                                    &action,
                                    pattern.as_deref(),
                                    rule_type.as_deref(),
                                    status,
                                    body.as_deref(),
                                    headers.as_ref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for route_rule"));
                    }

                    AgentCommand::DoubleClick { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_double_click_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for double_click"));
                    }

                    AgentCommand::RightClick { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_right_click_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for right_click"));
                    }

                    AgentCommand::ExtractTable { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_extract_table_script(req_id, target.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for extract_table"));
                    }

                    AgentCommand::Downloads { clear } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_downloads_script(req_id, clear);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for downloads"));
                    }

                    AgentCommand::NetworkConditions { profile, latency, download_throughput, upload_throughput, offline } => {
                        let is_offline = offline.unwrap_or(profile == "offline");
                        let (lat, dl, ul, conn_type) = if is_offline {
                            (0.0, 0.0, 0.0, "none")
                        } else if profile == "slow-3g" {
                            (latency.unwrap_or(500.0), download_throughput.unwrap_or(50000.0), upload_throughput.unwrap_or(50000.0), "cellular3g")
                        } else if profile == "fast-3g" {
                            (latency.unwrap_or(150.0), download_throughput.unwrap_or(200000.0), upload_throughput.unwrap_or(93750.0), "cellular3g")
                        } else {
                            (latency.unwrap_or(0.0), download_throughput.unwrap_or(-1.0), upload_throughput.unwrap_or(-1.0), "wifi")
                        };

                        let cdp_params = serde_json::json!({
                            "offline": is_offline,
                            "latency": lat,
                            "downloadThroughput": dl,
                            "uploadThroughput": ul,
                            "connectionType": conn_type
                        });

                        match crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Network.emulateNetworkConditions", Some(cdp_params)) {
                            Ok(_) => {
                                let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                    "status": "network_conditions_configured",
                                    "profile": profile,
                                    "offline": is_offline,
                                    "latency_ms": lat,
                                    "download_kbps": if dl > 0.0 { dl / 1024.0 } else { 0.0 },
                                    "upload_kbps": if ul > 0.0 { ul / 1024.0 } else { 0.0 },
                                    "connection_type": conn_type
                                })));
                            }
                            Err(e) => {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("CDP Network.emulateNetworkConditions failed: {}", e)));
                            }
                        }
                    }

                    AgentCommand::InjectCss { css, remove } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_inject_css_script(req_id, &css, remove);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for inject_css"));
                    }

                    AgentCommand::QueryElements { selector, attribute, max_items } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_query_elements_script(req_id, &selector, attribute.as_deref(), max_items);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for query_elements"));
                    }

                    AgentCommand::Focus { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_focus_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for focus"));
                    }

                    AgentCommand::Blur { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_blur_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for blur"));
                    }

                    AgentCommand::PageInfo => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_page_info_script(req_id);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for page_info"));
                    }

                    AgentCommand::GetHtml { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_html_script(req_id, target.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for get_html"));
                    }

                    AgentCommand::ScrollIntoView { target, align } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_scroll_into_view_script(req_id, &target, &align);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for scroll_into_view"));
                    }

                    AgentCommand::SetZoom { zoom_factor } => {
                        let factor = zoom_factor.clamp(0.25, 5.0);
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let script = format!(
                                    r#"(function() {{ document.body.style.zoom = '{:.2}'; }})();"#,
                                    factor
                                );
                                let _ = wv.evaluate_script(&script);
                            }
                        }
                        let params = serde_json::json!({ "pageScaleFactor": factor });
                        let _ = crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Emulation.setPageScaleFactor", Some(params));
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                            "status": "ok",
                            "zoom_factor": factor
                        })));
                    }

                    AgentCommand::StorageItem { storage_type, action, key, value } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_storage_item_script(
                                    req_id,
                                    &storage_type,
                                    &action,
                                    key.as_deref(),
                                    value.as_deref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for storage_item"));
                    }

                    AgentCommand::AssertElement { target, assertions } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_assert_element_script(
                                    req_id,
                                    &target,
                                    &assertions,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for assert_element"));
                    }

                    AgentCommand::WaitForSelector { target, state, timeout_ms } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_wait_for_selector_script(
                                    req_id,
                                    &target,
                                    &state,
                                    timeout_ms,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for wait_for_selector"));
                    }

                    AgentCommand::GetPerformanceMetrics => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_performance_metrics_script(req_id);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for performance_metrics"));
                    }

                    AgentCommand::ScrollToCoords { x, y, smooth } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_scroll_to_coords_script(req_id, x, y, smooth);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for scroll_to_coords"));
                    }

                    AgentCommand::GetPageOutline { max_links, max_buttons } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_page_outline_script(req_id, max_links, max_buttons);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for page_outline"));
                    }

                    AgentCommand::WaitForNetworkIdle { idle_time_ms, timeout_ms, max_inflight } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_wait_for_network_idle_script(
                                    req_id,
                                    idle_time_ms,
                                    timeout_ms,
                                    max_inflight,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for wait_for_network_idle"));
                    }

                    AgentCommand::WatchMutations { action, target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_watch_mutations_script(
                                    req_id,
                                    &action,
                                    target.as_deref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for watch_mutations"));
                    }

                    AgentCommand::TabEval { tab_index, script } => {
                        if let Some(tab) = tab_mgr.tabs.get(tab_index) {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let eval_script = crate::agent::AgentScripts::get_eval_script(req_id, &script);
                                let _ = wv.evaluate_script(&eval_script);
                                return;
                            } else {
                                let _ = response_tx.send(AgentResultEnvelope::err(format!("Tab {} is hibernated", tab_index)));
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err(format!("Tab index {} out of range", tab_index)));
                    }

                    AgentCommand::GetResponseBody { pattern, clear } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_response_body_script(req_id, &pattern, clear);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for response_body"));
                    }

                    AgentCommand::DropFile { target, file_name, mime_type, content } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_drop_file_script(
                                    req_id,
                                    &target,
                                    &file_name,
                                    &mime_type,
                                    &content,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for drop_file"));
                    }

                    AgentCommand::GetDomTree { target, max_depth } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_dom_tree_script(
                                    req_id,
                                    target.as_deref(),
                                    max_depth,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for dom_tree"));
                    }

                    AgentCommand::SetOffline { offline } => {
                        let params = serde_json::json!({
                            "offline": offline,
                            "latency": 0,
                            "downloadThroughput": -1,
                            "uploadThroughput": -1
                        });
                        let res = crate::agent::server::AgentServer::execute_cdp_method(
                            cdp_port,
                            "Network.emulateNetworkConditions",
                            Some(params),
                        );
                        if let Err(e) = res {
                            let _ = response_tx.send(AgentResultEnvelope::err(format!("Failed to emulate offline state: {}", e)));
                        } else {
                            let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                "status": "offline_mode_updated",
                                "offline": offline
                            })));
                        }
                    }

                    AgentCommand::WaitForText { target, text, timeout_ms } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_wait_for_text_script(
                                    req_id,
                                    target.as_deref(),
                                    &text,
                                    timeout_ms,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for wait_for_text"));
                    }

                    AgentCommand::GetComputedStyles { target, properties } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_computed_styles_script(
                                    req_id,
                                    &target,
                                    &properties,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for computed_styles"));
                    }

                    AgentCommand::WaitForDownload { file_pattern, timeout_ms } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_wait_for_download_script(
                                    req_id,
                                    file_pattern.as_deref(),
                                    timeout_ms,
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for wait_for_download"));
                    }

                    AgentCommand::SetContent { html } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_set_content_script(req_id, &html);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for set_content"));
                    }

                    AgentCommand::GetForms { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_forms_script(req_id, target.as_deref());
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for get_forms"));
                    }

                    AgentCommand::SetVisibility { state } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_set_visibility_script(req_id, &state);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for set_visibility"));
                    }

                    AgentCommand::EmulateMedia { media_type, color_scheme } => {
                        let mut params = serde_json::Map::new();
                        if let Some(media) = &media_type {
                            params.insert("media".to_string(), serde_json::Value::String(media.clone()));
                        }
                        if let Some(scheme) = &color_scheme {
                            params.insert(
                                "features".to_string(),
                                serde_json::json!([{
                                    "name": "prefers-color-scheme",
                                    "value": scheme
                                }])
                            );
                        }
                        let res = crate::agent::server::AgentServer::execute_cdp_method(
                            cdp_port,
                            "Emulation.setEmulatedMedia",
                            Some(serde_json::Value::Object(params)),
                        );
                        if let Err(e) = res {
                            let _ = response_tx.send(AgentResultEnvelope::err(format!("Failed to emulate media: {}", e)));
                        } else {
                            let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                                "status": "media_emulation_updated",
                                "media_type": media_type,
                                "color_scheme": color_scheme
                            })));
                        }
                    }

                    AgentCommand::GetElementBounds { target } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_element_bounds_script(req_id, &target);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for get_element_bounds"));
                    }

                    AgentCommand::MockRoute { action, pattern, status, content_type, body } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_mock_route_script(
                                    req_id,
                                    &action,
                                    pattern.as_deref(),
                                    status,
                                    &content_type,
                                    body.as_deref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for mock_route"));
                    }

                    AgentCommand::GetViewport => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_viewport_script(req_id);
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for get_viewport"));
                    }

                    AgentCommand::SetElementAttribute { target, attribute, value } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                let req_id = crate::agent::next_request_id();
                                pending_agent_requests.register(req_id, response_tx);
                                let script = crate::agent::AgentScripts::get_set_element_attribute_script(
                                    req_id,
                                    &target,
                                    &attribute,
                                    value.as_deref(),
                                );
                                let _ = wv.evaluate_script(&script);
                                return;
                            }
                        }
                        let _ = response_tx.send(AgentResultEnvelope::err("No active webview available for set_element_attribute"));
                    }

                    AgentCommand::Screenshot { som, full_page } => {
                        if let Some(tab) = tab_mgr.active_tab() {
                            if let Some(wv) = &tab.webview {
                                if som {
                                    let _ = wv.evaluate_script(crate::agent::AgentScripts::get_som_inject_script());
                                }
                            }
                        }
                        let mut response_data = serde_json::json!({
                            "status": "screenshot_ready",
                            "som_active": som,
                            "full_page": full_page,
                            "url": tab_mgr.active_tab().map(|t| t.url.as_str()).unwrap_or("about:home")
                        });

                        let params = if full_page {
                            Some(serde_json::json!({ "captureBeyondViewport": true, "format": "png" }))
                        } else {
                            Some(serde_json::json!({ "format": "png" }))
                        };
                        if let Ok(cdp_res) = crate::agent::server::AgentServer::execute_cdp_method(cdp_port, "Page.captureScreenshot", params) {
                            if let Some(b64) = cdp_res.get("data").and_then(|d| d.as_str()) {
                                response_data["screenshot_base64"] = serde_json::Value::String(b64.to_string());
                            }
                        }

                        let _ = response_tx.send(AgentResultEnvelope::ok(response_data));
                    }

                    AgentCommand::TabsList => {
                        let tabs: Vec<serde_json::Value> = tab_mgr.tabs.iter().enumerate().map(|(idx, t)| {
                            serde_json::json!({
                                "index": idx,
                                "id": t.id,
                                "title": t.title,
                                "url": t.url,
                                "is_active": idx == tab_mgr.active_index,
                                "is_sleeping": t.webview.is_none(),
                            })
                        }).collect();
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({
                            "tabs": tabs,
                            "active_index": tab_mgr.active_index
                        })));
                    }

                    AgentCommand::TabNew { url } => {
                        let bounds = content_rect(current_size, current_scale_factor);
                        tab_mgr.new_tab(&url, &window, &proxy, bounds, &adblocker, &webview_data_dir, &chrome_wv, &storage);
                        reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "tab_created", "url": url })));
                    }

                    AgentCommand::TabSwitch { index } => {
                        let bounds = content_rect(current_size, current_scale_factor);
                        tab_mgr.switch_tab(index, &window, &proxy, bounds, &adblocker, &webview_data_dir, &chrome_wv, &storage);
                        reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "tab_switched", "index": index })));
                    }

                    AgentCommand::TabClose { index } => {
                        let bounds = content_rect(current_size, current_scale_factor);
                        tab_mgr.close_tab(index, &window, &proxy, bounds, &adblocker, &webview_data_dir, &chrome_wv, &storage);
                        reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
                        let _ = response_tx.send(AgentResultEnvelope::ok(serde_json::json!({ "status": "tab_closed", "index": index })));
                    }
                }
            }

            Event::UserEvent(UserEvent::AgentResponse { tab_id: _, request_id, payload, error }) => {
                let result = if let Some(err) = error {
                    AgentResultEnvelope::err(err)
                } else if let Some(data) = payload {
                    AgentResultEnvelope::ok(data)
                } else {
                    AgentResultEnvelope::ok(serde_json::json!({ "status": "success" }))
                };
                pending_agent_requests.complete(request_id, result);
            }

            _ => ()
        }
    });
}

fn main() {
    #[cfg(target_os = "windows")]
    unsafe {
        unsafe extern "system" {
            fn AttachConsole(dwProcessId: u32) -> i32;
        }
        AttachConsole(0xFFFFFFFF);
    }

    let args: Vec<String> = std::env::args().collect();
    if AgentCli::handle_client_subcommand(&args) {
        return;
    }

    if let Err(e) = run() {
        show_fatal_error(&format!("Aerolith Agent Browser failed to start:\n\n{}", e));
    }
}

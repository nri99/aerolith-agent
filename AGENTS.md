# Aerolith Agent — Agent Developer Guide & Architecture Directives

> **CRITICAL DIRECTIVE FOR ALL AGENTS**: Do NOT read unnecessary files or dump entire directories into context. Follow the targeted module routing below to complete tasks in minimal tokens and turns.

---

## 1. Quick Module Routing (Targeted Reads Only)

**Never read `main.rs` unless modifying global window lifecycle or top-level event routing.** Jump directly to the specific module responsible for your task:

| Task / Feature Target | Exact File to Read & Edit | Description |
|---|---|---|
| **AI Agent MCP Server** | `src/agent/mcp.rs` | JSON-RPC 2.0 stdio Model Context Protocol server for Claude / Antigravity / Cursor |
| **Semantic DOM & SOM Scripts** | `src/agent/scripts.rs` | In-engine JS: accessibility tree snapshotting, Set-Of-Marks tags, clean Markdown |
| **Agent HTTP REST Daemon** | `src/agent/server.rs` | Embedded HTTP REST server on 127.0.0.1:8765 (`/snapshot`, `/click`, `/type`, etc.) |
| **Agent CLI Commands** | `src/agent/cli.rs` | Subcommands for terminal agents (`nav`, `snapshot`, `click`, `type`, `markdown`) |
| **Agent Command Schemas** | `src/agent/protocol.rs` | Wire types, MCP tool definitions, and response envelopes |
| **Toolbar, CSS, HTML, UI JS** | `src/ui.rs` | Custom HTML5/CSS3 tab strip, omnibox, settings drawer, speed dial |
| **Tab State, Lifecycle, Hibernation** | `src/tabs.rs` | Tab management, smart sleep exemptions, scroll restoration |
| **URL Normalization & Tracking Cleaner** | `src/url_utils.rs` | URL parsing, search engines, query tracking parameter stripper |
| **IPC Events & Messages** | `src/types.rs` | `UserEvent`, `ChromeMsg`, `TabInfo`, `TabState` |
| **Geometry & Scaling** | `src/geometry.rs` | `physical_chrome_height`, `content_rect`, high-DPI scaling |
| **Adblock & Anti-Fingerprinting** | `src/adblock.rs` | 85+ tracker hosts, canvas farbling, audio noise, WebRTC protection |
| **ACID Database & Settings** | `src/storage.rs` | `redb` embedded key-value store, sessions, bookmarks, history |
| **Memory Governor** | `src/governor.rs` | Win32 FFI, Linux /proc/meminfo, macOS sysctl memory tracking |
| **Paths & Platform Utilities** | `src/paths.rs` | Cross-platform data dirs, virtual keyboard autodetection |
| **WebView Creation** | `src/webview_factory.rs` | `wry` child WebView builders, script injections |
| **Main Window Loop** | `src/main.rs` | Event loop routing, agent server startup, CLI dispatch |

---

## 2. Inviolable Architecture & Z-Order Rules (DO NOT BREAK)

1. **Child WebViews Must Use `build_as_child(window)`**:
   - In `wry-0.57` on Windows, `WebViewBuilder::build(window)` sets `is_child = false`, which ignores custom bounds and calls `resize_to_parent()`, positioning the WebView at `(0, 0, full_width, full_height)` and covering the Chrome Header!
   - **ALWAYS** use `build_as_child(window)?` and immediately call `wv.set_bounds(bounds)`.
2. **Re-assert Chrome Header Bounds on Every Tab Action**:
   - When creating or activating any tab, call:
     ```rust
     reassert_chrome_bounds(&chrome_wv, &chrome_expansion_mode, current_scale_factor, current_size);
     ```
   - This guarantees `chrome_wv` remains pinned at `y = 0..chrome_h` and is never occluded in the Win32 child HWND Z-order.
3. **Safe Horizontal Tab Strip Panning (No Vertical Scroll)**:
   - In `src/ui.rs`, never use element `scrollIntoView()` on tabs, as it can vertically scroll the viewport. Use `tabStrip.scrollTo({ left: ..., behavior: 'smooth' })`.

---

## 3. Fast Testing & Build Protocol (Bypassing File Locks)

The repository resides inside Google Drive on Windows. Google Drive file streaming can momentarily lock compiled binaries (`LNK1104` / `LNK1201`).

To test instantly with **zero file-lock collisions** and **sub-second runtimes (0.15s)**:
```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\aerolith_agent_target"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" test --manifest-path aerolith-agent\Cargo.toml
```

To compile release binary:
```powershell
$env:CARGO_TARGET_DIR = "$env:TEMP\aerolith_agent_target"
& "$env:USERPROFILE\.cargo\bin\cargo.exe" build --release --manifest-path aerolith-agent\Cargo.toml
Copy-Item -Force "$env:TEMP\aerolith_agent_target\release\aerolith-agent.exe" "aerolith-agent\target\release\aerolith-agent.exe"
```

# Aerolith Agent — Playwright Python SDK

First-class Python Playwright integration for **Aerolith Agent**. Combines the full power of Playwright with Aerolith's anti-bot stealth, Set-Of-Marks grounding, token-efficient semantic snapshots, and atomic form filling.

## Installation

```bash
pip install playwright
```

*(No extra dependencies required — `aerolith_playwright.py` uses Python standard library `urllib` and `base64`).*

## Quickstart

Start the Aerolith Agent browser:
```bash
aerolith-agent serve --port 8765 --cdp-port 9222
```

In your Python automation script:
```python
from playwright.sync_api import sync_playwright
from aerolith_playwright import connect_aerolith

with sync_playwright() as p:
    browser, page = connect_aerolith(p, cdp_url="http://127.0.0.1:9222", agent_port=8765)
    page.goto("https://github.com/login")

    # 1. Capture token-efficient accessibility snapshot
    snapshot = page.semantic_snapshot(viewport_only=True)
    print(snapshot)

    # 2. Visually highlight element to verify grounding
    page.highlight("input[name=login]")

    # 3. Atomically fill multiple fields in one DOM turn
    page.fill_form([
        {"target": "input[name=login]", "value": "my-user"},
        {"target": "input[name=password]", "value": "secret"}
    ], submit=True)

    # 4. Capture Set-Of-Marks numbered bounding boxes for vision models
    page.screenshot_som("github_som.png")
```

## Extended API Reference

| Method | Description |
|---|---|
| `page.semantic_snapshot(viewport_only=False, role=None)` | Returns compact text tree with numbered `@ref` IDs and coordinates. |
| `page.highlight(target)` | Injects an animated `#00e676` focus ring around the target element. |
| `page.screenshot_element(target, path=None)` | Captures exact bounding-box cropped PNG image bytes. |
| `page.get_text(target, attribute=None)` | Extracts element text content, input value, or specific HTML attribute. |
| `page.hover(target)` | Hovers mouse over element to trigger flyouts, tooltips, and `:hover` states. |
| `page.press_key(key, target=None)` | Dispatches realistic keyboard events (`Escape`, `Enter`, `Tab`, etc.). |
| `page.select_option(target, value)` | Selects an option in a `<select>` dropdown by value, label, or index. |
| `page.drag(source, target=None, offset_x=None, offset_y=None, steps=10)` | Drags an element or adjusts a range slider with smooth interpolation. |
| `page.get_cookies()` | Retrieves all cookies (including `HttpOnly` and secure flags) via CDP. |
| `page.set_cookie(cookie)` | Sets a cookie dictionary via CDP. |
| `page.delete_cookie(name, domain=None)` | Deletes a cookie by name and domain via CDP. |
| `page.clear_cookies()` | Clears all browser cookies via CDP. |
| `page.set_extra_headers(headers)` | Sets persistent custom HTTP request headers via CDP. |
| `page.set_user_agent(user_agent, accept_language=None, platform=None)` | Overrides browser User-Agent and language via CDP. |
| `page.upload_file(target, files)` | Sets files on `<input type="file">` elements via synthetic DataTransfer. |
| `page.set_location(lat, lng, accuracy=None, timezone=None)` | Overrides geographical coordinates and timezone via CDP and geolocation APIs. |
| `page.media_control(action="status")` | Controls or queries audio and video playback on the page. |
| `page.pause_media()` | Pauses all video and audio playback across the page. |
| `page.get_network_activity(clear=False)` | Retrieves recorded AJAX/fetch requests and in-flight count. |
| `page.fill_form(fields, submit=False)` | Atomically populates multiple inputs and optionally submits. |
| `page.set_viewport(width, height, dpr=1.0, mobile=False)` | Emulates screen dimensions, device pixel ratio, and touch. |
| `page.get_console_logs(clear=False)` | Retrieves buffered browser console logs (log, warn, error). |
| `page.print_pdf(path=None, landscape=False)` | Generates high-fidelity vector PDF via CDP Page.printToPDF. |
| `page.get_markdown()` | Extracts clean, reader-friendly article markdown. |
| `page.screenshot_som(path=None)` | Renders high-contrast Set-Of-Marks tags before saving screenshot. |
| `page.export_storage_state(path=None)` | Exports cookies, `localStorage`, and `sessionStorage`. |
| `page.import_storage_state(state)` | Restores cookies, `localStorage`, and `sessionStorage`. |
| `page.raw_page` | Returns the raw, unadorned Playwright `Page` instance. |

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
| `page.screenshot_element(target, path=None, padding=0)` | Captures exact bounding-box cropped PNG image bytes with optional padding. |
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
| `page.inspect(target)` | Returns complete attributes, computed CSS styles, viewport visibility, and bounding box. |
| `page.route_abort(pattern)` | Dynamically intercepts and aborts network requests matching a URL pattern. |
| `page.route_mock(pattern, body="", status=200, headers=None)` | Dynamically intercepts and mocks responses matching a URL pattern. |
| `page.route_clear()` | Clears all dynamic route interception rules. |
| `page.wait_for_network_idle(timeout_ms=10000)` | Waits until all network requests have settled for at least 250ms. |
| `page.wait_for_url(pattern, timeout_ms=10000)` | Waits until page URL matches or contains the target substring. |
| `page.wait_for_function(js_code, timeout_ms=10000)` | Waits until a JavaScript condition evaluates to truthy. |
| `page.get_tabs()` | Lists all open browser tabs and identifies the active tab index. |
| `page.new_tab(url="about:home")` | Opens a new browser tab. |
| `page.switch_tab(index)` | Switches active focus to a tab by its index. |
| `page.close_tab(index)` | Closes a tab by its index. |
| `page.get_network_activity(clear=False)` | Retrieves recorded AJAX/fetch requests and in-flight count. |
| `page.fill_form(fields, submit=False)` | Atomically populates multiple inputs and optionally submits. |
| `page.set_viewport(width, height, dpr=1.0, mobile=False)` | Emulates screen dimensions, device pixel ratio, and touch. |
| `page.get_console_logs(clear=False)` | Retrieves buffered browser console logs (log, warn, error). |
| `page.print_pdf(path=None, landscape=False)` | Generates high-fidelity vector PDF via CDP Page.printToPDF. |
| `page.get_markdown()` | Extracts clean, reader-friendly article markdown. |
| `page.screenshot_som(path=None)` | Renders high-contrast Set-Of-Marks tags before saving screenshot. |
| `page.export_storage_state(path=None)` | Exports cookies, `localStorage`, and `sessionStorage`. |
| `page.import_storage_state(state)` | Restores cookies, `localStorage`, and `sessionStorage`. |
| `page.double_click(target)` | Simulates a human-like double click with sticky-header occlusion compensation. |
| `page.right_click(target)` | Simulates a right-click / contextmenu event on target element. |
| `page.extract_table(target=None)` | Extracts structured rows and headers from an HTML table into clean JSON records. |
| `page.get_downloads(clear=False)` | Retrieves recorded browser file downloads or clears history. |
| `page.emulate_network(profile="online", ...)` | Throttles network conditions (online, offline, slow-3g, fast-3g) via CDP. |
| `page.inject_css(css)` | Injects custom CSS rules to eliminate floating headers, banners, or occluding elements. |
| `page.clear_css()` | Removes previously injected custom CSS stylesheet. |
| `page.query_elements(selector, attribute=None, max_items=50)` | Batch queries all matching elements and returns indexed metadata records. |
| `page.focus(target)` | Explicitly focuses an element to trigger framework validation. |
| `page.blur(target)` | Explicitly blurs an element to trigger framework change events. |
| `page.get_page_info()` | Retrieves page URL, title, viewport dimensions, document height, and scroll metrics. |
| `page.get_html(target=None)` | Extracts raw HTML source for the entire document or a specific DOM element. |
| `page.scroll_into_view(target, align="center")` | Scrolls element into view with block alignment and sticky-header compensation. |
| `page.set_zoom(zoom_factor)` | Sets browser rendering zoom factor with dual-layer CSS and CDP scaling. |
| `page.storage_item(action, storage_type="local", ...)` | Inspects or mutates granular localStorage or sessionStorage keys. |
| `page.assert_element(target, **assertions)` | Verifies element state against declarative assertions (visible, hidden, disabled, checked, text, value, classes, attributes). |
| `page.wait_for_selector(target, state="visible", timeout_ms=5000)` | Waits for element to reach state: 'visible', 'hidden', 'attached', 'detached'. |
| `page.get_performance_metrics()` | Retrieves page performance metrics (TTFB, DOMContentLoaded, load time, resource breakdown, transfer bytes, JS heap). |
| `page.scroll_to(x, y, smooth=False)` | Scrolls viewport directly to specific absolute (x, y) coordinates. |
| `page.get_page_outline(max_links=25, max_buttons=25)` | Retrieves compact semantic outline of the page: headings, forms, landmarks, buttons, and links. |
| `page.wait_for_network_idle(idle_time_ms=500, ...)` | Waits until active network requests remain at or below max_inflight for consecutive idle_time_ms. |
| `page.watch_mutations(action="get", target=None)` | Controls and queries DOM mutation records (start, stop, clear, get). |
| `page.tab_eval(tab_index, script)` | Evaluates a JavaScript expression directly in a specific tab by index without switching active GUI focus. |
| `page.get_response_body(pattern="", clear=False)` | Searches recorded network activity for URL pattern and returns captured response body. |
| `page.drop_file(target, file_name, content, ...)` | Simulates synthetic file drag-and-drop or file input upload with custom metadata. |
| `page.get_dom_tree(target=None, max_depth=4)` | Serializes nested hierarchical JSON tree of visible DOM nodes. |
| `page.set_offline(offline=True)` | Instantly simulates network offline or online state via CDP Network emulation. |
| `page.wait_for_text(text, target=None, timeout_ms=5000)` | Waits until a specific text substring appears in an element or document body. |
| `page.get_computed_styles(target, properties=None)` | Reads computed CSS style properties (display, visibility, opacity, color, z-index, etc.). |
| `page.wait_for_download(file_pattern=None, timeout_ms=15000)` | Waits for a new file download initiated by the page to appear. |
| `page.set_content(html)` | Sets the HTML content of the active page directly via document.write. |
| `page.get_forms(target=None)` | Inspects and structures all HTML forms and their controls (input, select, textarea, button). |
| `page.set_visibility(state="visible")` | Force-emulates document.visibilityState ('visible' or 'hidden') and window focus. |
| `page.emulate_media(media_type=None, color_scheme=None)` | Emulates media color scheme ('dark', 'light') and/or media type ('screen', 'print'). |
| `page.get_element_bounds(target)` | Extracts exact pixel bounding box coordinates (x, y, width, height, center_x, center_y) and visibility metrics. |
| `page.mock_route(pattern, body=None, status=200, content_type="application/json")` | Mocks network responses for fetch/XHR requests matching a URL pattern. |
| `page.clear_mock_routes()` | Clears all active network mock routes. |
| `page.get_viewport()` | Retrieves active viewport dimensions, scroll offsets, screen metrics, and device pixel ratio. |
| `page.set_element_attribute(target, attribute, value=None)` | Dynamically sets or removes an HTML attribute on a DOM element for self-healing. |
| `page.search_page_text(query, max_matches=20, case_sensitive=False)` | Searches visible page text for substrings and returns matches, element refs, and bounding boxes. |
| `page.dump_storage(storage_type="all")` | Dumps all localStorage and/or sessionStorage key-value pairs in a single structured JSON payload. |
| `page.human_jitter(duration_ms=800, move_mouse=True, micro_scroll=True)` | Simulates subtle mouse micro-movements and scroll adjustments to pass anti-bot behavioral heuristics. |
| `page.type_keystrokes(target, text, delay_ms=25, clear=False, submit=False)` | Types text character-by-character with realistic key delays and complete KeyboardEvents. |
| `page.get_canvas_data(target, mime_type="image/png", quality=0.92)` | Extracts base64 image data URL from <canvas>, <img>, or arbitrary element via rasterization. |
| `page.get_resource_timing(initiator_type=None, max_entries=50, min_duration_ms=0.0)` | Extracts structured resource timing and transfer bandwidth metrics from window.performance. |
| `page.get_dom_stats()` | Inspects structural DOM complexity metrics (element count, tree depth, forms, buttons, links, inputs). |
| `page.scroll_by(dx=0, dy=0, smooth=False)` | Performs relative window scroll by horizontal (dx) and vertical (dy) pixel deltas. |
| `page.scroll_container(target, dx=0, dy=0, smooth=False)` | Scrolls a specific inner scrollable container (div, modal, textarea, section). |
| `page.select_element_text(target, start=None, length=None)` | Selects text range inside an input, textarea, or contentEditable element. |
| `page.extract_page_links(max_links=100, external_only=False, filter_path=None)` | Extracts structured, deduplicated hyperlinks graph from active document. |
| `page.get_element_state(target)` | Atomically inspects complete interactive, disabled, focused, checked, and visibility state. |
| `page.toggle_element_class(target, class_name, action="toggle")` | Adds, removes, toggles, or checks a CSS class name on the target DOM element. |
| `page.set_element_text(target, text)` | Directly overwrites the textContent of target element and dispatches input/change events. |
| `page.dispatch_custom_event(target, event_type, bubbles=True, cancelable=True, detail=None)` | Constructs and dispatches a synthetic CustomEvent on target element. |
| `page.get_active_element()` | Inspects the currently focused DOM element (document.activeElement). |
| `page.check_element_clickable(target)` | Verifies whether an element is physically clickable at its center point or occluded by overlays/modals. |
| `page.dispatch_wheel_event(target, delta_x=0.0, delta_y=0.0, ctrl_key=False)` | Constructs and dispatches a synthetic WheelEvent on target element (for map zooms, pan widgets, and charts). |
| `page.get_css_variables(target=None, variable_names=None)` | Resolves computed values of CSS custom properties (--var) on an element or root document. |
| `page.clear_highlights()` | Removes all agent visual highlight overlays and badge borders from the page DOM. |
| `page.simulate_pointer_click(target, click_count=1, button=0)` | Dispatches full sequence of PointerEvents (pointerdown, mousedown, pointerup, mouseup, click) at target center. |
| `page.extract_meta_tags(include_json_ld=True)` | Extracts structured page metadata, OpenGraph tags, Twitter cards, links, and JSON-LD schemas. |
| `page.get_document_geometry()` | Extracts complete document dimensions, viewport metrics, scroll offsets, and scrollable container count. |
| `page.set_device_orientation(alpha=0.0, beta=0.0, gamma=0.0, absolute=False)` | Emulates mobile device orientation tilt angles (alpha, beta, gamma) and dispatches deviceorientation event. |
| `page.clear_browser_cache(clear_storage=False, clear_cookies=False)` | Purges browser in-memory and disk caches via CDP, with optional storage/cookie reset. |
| `page.get_element_spacing(target)` | Inspects box-model computed spacing (margin, padding, border, dimensions, z-index, box-sizing) for an element. |
| `page.wait_for_url(pattern, timeout_ms=5000)` | Waits until window.location.href matches a target substring or regex pattern. |
| `page.dispatch_touch_event(event_type="touchstart", touches=None)` | Dispatches synthetic touch events (touchstart, touchmove, touchend, touchcancel) with multiple touch points. |
| `page.get_a11y_tree(target=None, max_depth=5)` | Extracts hierarchical computed WAI-ARIA accessibility (a11y) tree for navigation and screen reader analysis. |
| `page.simulate_keyboard_chord(chord, target=None)` | Dispatches keyboard hotkey chord (e.g. 'Control+Shift+P', 'Meta+K', 'Alt+Tab') with keydown/keyup sequence. |
| `page.highlight_bounding_boxes(boxes)` | Renders visual bounding boxes with label badges over page regions for grounding and multimodal verification. |
| `page.get_element_attributes(target)` | Extracts all HTML attributes of a DOM element as a complete key-value dictionary. |
| `page.get_element_filter(target)` | Inspects CSS filters, backdrop filters, opacity, pointer-events, and ancestor occlusion. |
| `page.dismiss_all_modals()` | Dismisses modal overlays, cookie consent popups, backdrop dialogs, and restores body scroll. |
| `page.get_color_palette(limit=15)` | Extracts dominant computed colors and background colors used across visible elements on the page. |
| `page.drag_and_drop_element(source, target, steps=10, hold_ms=0)` | Drags a source element and drops it onto a target element with multi-step pointer and drag event simulation. |
| `page.audit_forms(target=None)` | Audits form elements, input validation constraints (required, pattern, min/max), and HTML5 validity states. |
| `page.get_viewport_paging()` | Calculates viewport dimensions, multi-page scroll coordinates, total pages, and current scroll progress. |
| `page.get_computed_pseudo_element(target, pseudo="::before", properties=None)` | Inspects computed CSS styles of pseudo-elements like ::before or ::after (icons, badges, content). |
| `page.simulate_clipboard_paste(text, target=None)` | Simulates a clipboard paste event into a target or active element with DataTransfer text payload. |
| `page.smart_fill(mapping, submit=False)` | Smartly auto-fills form controls by fuzzy matching human-readable labels, names, or placeholders. |
| `page.observe_text(target, timeout_ms=10000, settle_ms=1000)` | Observes streaming text changes on an element until updates settle or timeout expires. |
| `page.set_visibility_state(state="visible")` | Overrides document.visibilityState to prevent bot pausing or background idle detection. |
| `page.get_computed_font(target)` | Extracts computed font typography, font family, weight, line-height, and canvas measurement metrics. |
| `page.get_action_map(viewport_only=True, limit=50)` | Extracts dense interactive element click-map with centroids and numeric ref bindings. |
| `page.get_motion_state(target=None)` | Inspects active CSS animations and transitions on the page or target element. |
| `page.get_element_listeners(target)` | Inspects event listeners, inline handlers, cursor styles, and interactive affordances of an element. |
| `page.get_web_vitals()` | Extracts Core Web Vitals, performance navigation timings (TTFB, FCP, DOMContentLoaded), and memory metrics. |
| `page.raw_page` | Returns the raw, unadorned Playwright `Page` instance. |

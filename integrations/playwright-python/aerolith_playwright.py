"""
Aerolith Agent - First-Class Playwright Python Integration.

Enables seamless connection between Python Playwright and Aerolith Agent over
Chrome DevTools Protocol (CDP), enhancing standard Playwright Page objects with
Aerolith's autonomous agent superpowers:
  - page.semantic_snapshot(viewport_only=False, role=None)
  - page.highlight(target)
  - page.screenshot_element(target, path=None)
  - page.fill_form(fields, submit=False)
  - page.get_markdown()
  - page.screenshot_som(path=None)
  - page.export_storage_state(path=None)
  - page.import_storage_state(state)
"""

import base64
import json
import urllib.parse
import urllib.request
from typing import Any, Dict, List, Optional, Union


class AerolithPageMixin:
    """Extension methods injected onto Playwright Page instances."""

    def __init__(self, page, agent_port: int = 8765):
        self._page = page
        self._agent_port = agent_port

    @property
    def raw_page(self):
        """Returns the underlying standard Playwright Page instance."""
        return self._page

    def _http_get(self, path: str) -> Dict[str, Any]:
        url = f"http://127.0.0.1:{self._agent_port}{path}"
        req = urllib.request.Request(url, headers={"User-Agent": "AerolithPlaywright/0.2.0"})
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def _http_post(self, path: str, payload: Dict[str, Any]) -> Dict[str, Any]:
        url = f"http://127.0.0.1:{self._agent_port}{path}"
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(
            url,
            data=data,
            headers={"Content-Type": "application/json", "User-Agent": "AerolithPlaywright/0.2.0"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def semantic_snapshot(self, viewport_only: bool = False, role: Optional[str] = None) -> str:
        """Capture a token-efficient semantic accessibility tree with numbered target IDs (@1, @2, ...)."""
        params = []
        if viewport_only:
            params.append("viewport_only=1")
        if role:
            params.append(f"role={urllib.parse.quote(role)}")
        query_str = f"?{'&'.join(params)}" if params else ""
        res = self._http_get(f"/snapshot{query_str}")
        return res.get("data", {}).get("tree_text", "")

    def highlight(self, target: str) -> Dict[str, Any]:
        """Draw an animated focus ring around an element to verify visual grounding."""
        res = self._http_post("/highlight", {"target": str(target)})
        return res.get("data", {})

    def screenshot_element(self, target: str, path: Optional[str] = None) -> bytes:
        """Capture an exact cropped visual screenshot of an element bounding box."""
        res = self._http_post("/screenshot/element", {"target": str(target)})
        b64 = res.get("data", {}).get("screenshot_base64", "")
        img_bytes = base64.b64decode(b64) if b64 else b""
        if path and img_bytes:
            with open(path, "wb") as f:
                f.write(img_bytes)
        return img_bytes

    def fill_form(self, fields: List[Dict[str, str]], submit: bool = False) -> Dict[str, Any]:
        """Atomically populate multiple input fields in a single DOM turn."""
        res = self._http_post("/fill", {"fields": fields, "submit": submit})
        return res.get("data", {})

    def hover(self, target: str) -> Dict[str, Any]:
        """Hover over an element to reveal flyouts, dropdowns, or trigger CSS :hover states."""
        res = self._http_post("/hover", {"target": str(target)})
        return res.get("data", {})

    def press_key(self, key: str, target: Optional[str] = None) -> Dict[str, Any]:
        """Press a keyboard key (e.g. 'Escape', 'Enter', 'Tab', 'ArrowDown', 'Backspace')."""
        payload: Dict[str, Any] = {"key": key}
        if target:
            payload["target"] = str(target)
        res = self._http_post("/key", payload)
        return res.get("data", {})

    def select_option(self, target: str, value: str) -> Dict[str, Any]:
        """Select an option in a HTML <select> dropdown by value, label, or index."""
        res = self._http_post("/select", {"target": str(target), "value": str(value)})
        return res.get("data", {})

    def drag(
        self,
        source: str,
        target: Optional[str] = None,
        offset_x: Optional[int] = None,
        offset_y: Optional[int] = None,
        steps: int = 10,
    ) -> Dict[str, Any]:
        """Perform a drag-and-drop gesture or slide an <input type="range"> slider."""
        payload: Dict[str, Any] = {"source": str(source), "steps": steps}
        if target:
            payload["target"] = str(target)
        if offset_x is not None:
            payload["offset_x"] = offset_x
        if offset_y is not None:
            payload["offset_y"] = offset_y
        res = self._http_post("/drag", payload)
        return res.get("data", {})

    def get_cookies(self) -> List[Dict[str, Any]]:
        """Retrieve all browser cookies (including HttpOnly and secure cookies) via CDP."""
        res = self._http_get("/cookies")
        return res.get("data", {}).get("cookies", [])

    def set_cookie(self, cookie: Dict[str, Any]) -> Dict[str, Any]:
        """Set a browser cookie via CDP."""
        res = self._http_post("/cookies", cookie)
        return res.get("data", {})

    def delete_cookie(self, name: str, domain: Optional[str] = None) -> Dict[str, Any]:
        """Delete a browser cookie by name and domain via CDP."""
        payload: Dict[str, Any] = {"name": name}
        if domain:
            payload["domain"] = domain
        res = self._http_post("/cookies/delete", payload)
        return res.get("data", {})

    def clear_cookies(self) -> Dict[str, Any]:
        """Clear all browser cookies via CDP."""
        res = self._http_post("/cookies/clear", {})
        return res.get("data", {})

    def set_extra_headers(self, headers: Dict[str, str]) -> Dict[str, Any]:
        """Set custom HTTP headers applied to all subsequent requests via CDP."""
        res = self._http_post("/headers", headers)
        return res.get("data", {})

    def set_user_agent(
        self,
        user_agent: str,
        accept_language: Optional[str] = None,
        platform: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Override browser User-Agent, Accept-Language, and Platform identifiers via CDP."""
        payload: Dict[str, Any] = {"user_agent": user_agent}
        if accept_language:
            payload["accept_language"] = accept_language
        if platform:
            payload["platform"] = platform
        res = self._http_post("/user_agent", payload)
        return res.get("data", {})

    def upload_file(self, target: str, files: Union[str, List[str]]) -> Dict[str, Any]:
        """Upload one or more files to an <input type="file"> element."""
        file_list = [files] if isinstance(files, str) else list(files)
        res = self._http_post("/upload", {"target": str(target), "files": file_list})
        return res.get("data", {})

    def set_location(
        self,
        latitude: float,
        longitude: float,
        accuracy: Optional[float] = None,
        timezone: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Emulate geographical coordinates (latitude, longitude, accuracy) and timezone."""
        payload: Dict[str, Any] = {"latitude": latitude, "longitude": longitude}
        if accuracy is not None:
            payload["accuracy"] = accuracy
        if timezone is not None:
            payload["timezone"] = timezone
        res = self._http_post("/location", payload)
        return res.get("data", {})

    def media_control(self, action: str = "status") -> Dict[str, Any]:
        """Inspect and control page media playback (status, pause, play, mute, unmute)."""
        res = self._http_get("/media") if action == "status" else self._http_post("/media", {"action": action})
        return res.get("data", {})

    def pause_media(self) -> Dict[str, Any]:
        """Pause all playing video and audio elements on the page."""
        return self.media_control("pause")

    def mute_media(self) -> Dict[str, Any]:
        """Mute all video and audio elements on the page."""
        return self.media_control("mute")

    def get_network_activity(self, clear: bool = False) -> Dict[str, Any]:
        """Retrieve recent network requests (fetch/XHR) and active in-flight request count."""
        path = "/network/clear" if clear else "/network"
        res = self._http_post(path, {}) if clear else self._http_get(path)
        return res.get("data", {})

    def set_viewport(self, width: int, height: int, dpr: Optional[float] = None, mobile: bool = False) -> Dict[str, Any]:
        """Emulate responsive device viewport dimensions, pixel ratio, and mobile touch."""
        payload = {"width": width, "height": height, "mobile": mobile}
        if dpr is not None:
            payload["dpr"] = dpr
        res = self._http_post("/emulate", payload)
        return res.get("data", {})

    def get_console_logs(self, clear: bool = False) -> List[Dict[str, Any]]:
        """Retrieve buffered browser console logs (log, warn, error, unhandled exceptions)."""
        path = "/console/clear" if clear else "/console"
        res = self._http_get(path)
        return res.get("data", {}).get("logs", [])

    def print_pdf(self, path: Optional[str] = None, landscape: bool = False) -> bytes:
        """Generate high-fidelity vector PDF from active page via CDP Page.printToPDF."""
        path_query = "/pdf?landscape=1" if landscape else "/pdf"
        res = self._http_get(path_query)
        b64 = res.get("data", {}).get("pdf_base64", "")
        pdf_bytes = base64.b64decode(b64) if b64 else b""
        if path and pdf_bytes:
            with open(path, "wb") as f:
                f.write(pdf_bytes)
        return pdf_bytes

    def inspect(self, target: str) -> Dict[str, Any]:
        """Deeply inspect element tag, text, value, computed styles, bounds, and state flags."""
        res = self._http_post("/inspect", {"target": str(target)})
        return res.get("data", {})

    def route_abort(self, pattern: str) -> Dict[str, Any]:
        """Intercept and immediately abort network requests matching URL pattern (e.g. '*.png', '*/ads/*')."""
        res = self._http_post("/route", {"action": "abort", "pattern": pattern, "rule_type": "abort"})
        return res.get("data", {})

    def route_mock(
        self,
        pattern: str,
        body: str = "",
        status: int = 200,
        headers: Optional[Dict[str, str]] = None,
    ) -> Dict[str, Any]:
        """Intercept and return a synthetic mock response for requests matching URL pattern."""
        payload: Dict[str, Any] = {
            "action": "mock",
            "pattern": pattern,
            "rule_type": "mock",
            "status": status,
            "body": body,
        }
        if headers:
            payload["headers"] = headers
        res = self._http_post("/route", payload)
        return res.get("data", {})

    def route_clear(self) -> Dict[str, Any]:
        """Clear all active dynamic route interception rules."""
        res = self._http_post("/route", {"action": "clear"})
        return res.get("data", {})

    def get_routes(self) -> List[Dict[str, Any]]:
        """Retrieve all currently active dynamic route rules."""
        res = self._http_get("/route")
        return res.get("data", {}).get("routes", [])

    def wait_for_network_idle(self, timeout_ms: int = 10000) -> Dict[str, Any]:
        """Wait until all in-flight network requests (fetch/XHR) have resolved for at least 250ms."""
        res = self._http_post("/wait_for", {"target": "networkidle", "timeout_ms": timeout_ms})
        return res.get("data", {})

    def wait_for_url(self, pattern: str, timeout_ms: int = 10000) -> Dict[str, Any]:
        """Wait until the browser URL matches or contains the specified string."""
        res = self._http_post("/wait_for", {"target": f"url:{pattern}", "timeout_ms": timeout_ms})
        return res.get("data", {})

    def wait_for_function(self, js_code: str, timeout_ms: int = 10000) -> Dict[str, Any]:
        """Wait until the provided JavaScript expression evaluates to truthy."""
        res = self._http_post("/wait_for", {"target": f"js:{js_code}", "timeout_ms": timeout_ms})
        return res.get("data", {})

    def get_tabs(self) -> List[Dict[str, Any]]:
        """List all open browser tabs and identify the active tab index."""
        res = self._http_get("/tabs")
        return res.get("data", {}).get("tabs", [])

    def new_tab(self, url: str = "about:home") -> Dict[str, Any]:
        """Open a new browser tab with the specified URL."""
        res = self._http_post("/tabs/new", {"url": url})
        return res.get("data", {})

    def switch_tab(self, index: int) -> Dict[str, Any]:
        """Switch active focus to a tab by its zero-based index."""
        res = self._http_post("/tabs/switch", {"index": index})
        return res.get("data", {})

    def close_tab(self, index: int) -> Dict[str, Any]:
        """Close a tab by its zero-based index."""
        res = self._http_post("/tabs/close", {"index": index})
        return res.get("data", {})

    def get_text(self, target: str, attribute: Optional[str] = None) -> str:
        """Extract text content, input value, or specific HTML attribute of an element."""
        params = [f"target={urllib.parse.quote(str(target))}"]
        if attribute:
            params.append(f"attr={urllib.parse.quote(attribute)}")
        res = self._http_get(f"/text?{'&'.join(params)}")
        return res.get("data", {}).get("text", "")

    def get_markdown(self) -> str:
        """Extract clean, reader-friendly article markdown, stripping navbars and ads."""
        res = self._http_get("/markdown")
        return res.get("data", {}).get("markdown", "")

    def screenshot_som(self, path: Optional[str] = None) -> bytes:
        """Capture a screenshot with high-contrast Set-Of-Marks numbered badges over elements."""
        res = self._http_get("/screenshot?som=1")
        b64 = res.get("data", {}).get("screenshot_base64", "")
        img_bytes = base64.b64decode(b64) if b64 else b""
        if path and img_bytes:
            with open(path, "wb") as f:
                f.write(img_bytes)
        return img_bytes

    def export_storage_state(self, path: Optional[str] = None) -> Dict[str, Any]:
        """Export browser cookies, localStorage, and sessionStorage."""
        res = self._http_get("/storage/state")
        data = res.get("data", {})
        if path:
            with open(path, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2)
        return data

    def import_storage_state(self, state: Union[Dict[str, Any], str]) -> Dict[str, Any]:
        """Restore browser cookies, localStorage, and sessionStorage from dict or JSON file."""
        if isinstance(state, str):
            with open(state, "r", encoding="utf-8") as f:
                state_dict = json.load(f)
        else:
            state_dict = state
        res = self._http_post("/storage/state", state_dict)
        return res.get("data", {})

    def double_click(self, target: str) -> Dict[str, Any]:
        """Perform a human-like double click with sticky-header occlusion compensation."""
        res = self._http_post("/dblclick", {"target": str(target)})
        return res.get("data", {})

    def dblclick(self, target: str) -> Dict[str, Any]:
        """Alias for double_click."""
        return self.double_click(target)

    def right_click(self, target: str) -> Dict[str, Any]:
        """Perform a right-click / contextmenu event on the target element."""
        res = self._http_post("/right_click", {"target": str(target)})
        return res.get("data", {})

    def contextmenu(self, target: str) -> Dict[str, Any]:
        """Alias for right_click."""
        return self.right_click(target)

    def extract_table(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Extract structured rows and headers from an HTML table into clean JSON records."""
        path = f"/table?target={urllib.parse.quote(str(target))}" if target else "/table"
        res = self._http_get(path)
        return res.get("data", {})

    def get_downloads(self, clear: bool = False) -> List[Dict[str, Any]]:
        """Retrieve recorded browser file downloads, optionally clearing download history."""
        res = self._http_post("/downloads/clear", {}) if clear else self._http_get("/downloads")
        return res.get("data", {}).get("downloads", [])

    def emulate_network(
        self,
        profile: str = "online",
        latency: Optional[float] = None,
        download: Optional[float] = None,
        upload: Optional[float] = None,
        offline: bool = False,
    ) -> Dict[str, Any]:
        """Throttle or emulate network conditions (profiles: 'online', 'offline', 'slow-3g', 'fast-3g')."""
        payload: Dict[str, Any] = {"profile": profile, "offline": offline}
        if latency is not None:
            payload["latency"] = latency
        if download is not None:
            payload["download"] = download
        if upload is not None:
            payload["upload"] = upload
        res = self._http_post("/network/emulate", payload)
        return res.get("data", {})

    def inject_css(self, css: str) -> Dict[str, Any]:
        """Inject custom CSS rules to hide headers, popups, or adjust layout."""
        res = self._http_post("/css", {"css": css, "remove": False})
        return res.get("data", {})

    def clear_css(self) -> Dict[str, Any]:
        """Remove previously injected custom CSS stylesheet."""
        res = self._http_post("/css/clear", {})
        return res.get("data", {})

    def query_elements(
        self,
        selector: str,
        attribute: Optional[str] = None,
        max_items: int = 50,
    ) -> List[Dict[str, Any]]:
        """Batch query all elements matching a CSS selector and return indexed metadata records."""
        payload: Dict[str, Any] = {"selector": selector, "max_items": max_items}
        if attribute:
            payload["attribute"] = attribute
        res = self._http_post("/query", payload)
        return res.get("data", {}).get("items", [])

    def focus(self, target: str) -> Dict[str, Any]:
        """Explicitly focus an element to trigger framework validation and input readiness."""
        res = self._http_post("/focus", {"target": str(target)})
        return res.get("data", {})

    def blur(self, target: str) -> Dict[str, Any]:
        """Explicitly blur an element to trigger framework change events and input validation."""
        res = self._http_post("/blur", {"target": str(target)})
        return res.get("data", {})

    def get_page_info(self) -> Dict[str, Any]:
        """Retrieve comprehensive page layout metrics, scroll progress, document dimensions, and window metadata."""
        res = self._http_get("/page_info")
        return res.get("data", {})

    def get_html(self, target: Optional[str] = None) -> str:
        """Extract raw HTML source for the entire document or a specific DOM element by selector or ref."""
        endpoint = f"/html?target={urllib.parse.quote(target)}" if target else "/html"
        res = self._http_get(endpoint)
        return res.get("data", {}).get("html", "")

    def scroll_into_view(self, target: str, align: str = "center") -> Dict[str, Any]:
        """Scroll a specific element into view with block alignment and automatic sticky-header occlusion compensation."""
        res = self._http_post("/scroll_into_view", {"target": str(target), "align": align})
        return res.get("data", {})

    def set_zoom(self, zoom_factor: float) -> Dict[str, Any]:
        """Set the browser rendering zoom factor (e.g. 1.2 for 120%) with dual-layer CSS and CDP scaling."""
        res = self._http_post("/zoom", {"zoom_factor": float(zoom_factor)})
        return res.get("data", {})

    def storage_item(
        self,
        action: str,
        storage_type: str = "local",
        key: Optional[str] = None,
        value: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Inspect or mutate granular localStorage or sessionStorage keys (get, set, delete, clear, keys)."""
        payload: Dict[str, Any] = {"action": action, "storage_type": storage_type}
        if key is not None:
            payload["key"] = key
        if value is not None:
            payload["value"] = value
        res = self._http_post("/storage/item", payload)
        return res.get("data", {})

    def assert_element(
        self,
        target: str,
        visible: Optional[bool] = None,
        hidden: Optional[bool] = None,
        disabled: Optional[bool] = None,
        checked: Optional[bool] = None,
        text_contains: Optional[str] = None,
        value: Optional[str] = None,
        has_class: Optional[str] = None,
        attr_name: Optional[str] = None,
        attr_value: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Verify element state against declarative assertions (visible, hidden, disabled, checked, text, value, classes, attributes)."""
        assertions: Dict[str, Any] = {}
        if visible is not None: assertions["visible"] = visible
        if hidden is not None: assertions["hidden"] = hidden
        if disabled is not None: assertions["disabled"] = disabled
        if checked is not None: assertions["checked"] = checked
        if text_contains is not None: assertions["text_contains"] = text_contains
        if value is not None: assertions["value"] = value
        if has_class is not None: assertions["has_class"] = has_class
        if attr_name is not None: assertions["attr_name"] = attr_name
        if attr_value is not None: assertions["attr_value"] = attr_value

        payload = {"target": str(target), "assertions": assertions}
        res = self._http_post("/assert_element", payload)
        return res.get("data", {})

    def wait_for_selector(
        self,
        target: str,
        state: str = "visible",
        timeout_ms: int = 5000,
    ) -> Dict[str, Any]:
        """Wait for a DOM element to reach a target state: 'visible', 'hidden', 'attached', or 'detached'."""
        payload = {"target": str(target), "state": state, "timeout_ms": timeout_ms}
        res = self._http_post("/wait_for_selector", payload)
        return res.get("data", {})

    def get_performance_metrics(self) -> Dict[str, Any]:
        """Retrieve page performance metrics (TTFB, DOMContentLoaded, load time, resource breakdown, transfer bytes, JS heap)."""
        res = self._http_get("/performance")
        return res.get("data", {}).get("metrics", {})

    def scroll_to(self, x: int, y: int, smooth: bool = False) -> Dict[str, Any]:
        """Scroll the browser viewport directly to specific absolute (x, y) coordinates."""
        payload = {"x": int(x), "y": int(y), "smooth": smooth}
        res = self._http_post("/scroll_to", payload)
        return res.get("data", {})

    def __getattr__(self, name):
        """Pass through standard Playwright Page attributes and methods."""
        return getattr(self._page, name)


def connect_aerolith(playwright_instance, cdp_url: str = "http://127.0.0.1:9222", agent_port: int = 8765):
    """
    Connect a Playwright instance to a running Aerolith Agent browser.

    Args:
        playwright_instance: The Playwright instance (from sync_playwright() or async_playwright())
        cdp_url: The CDP debugging URL (default: http://127.0.0.1:9222)
        agent_port: The Aerolith Agent HTTP server port (default: 8765)

    Returns:
        tuple: (browser, page_with_aerolith_extensions)
    """
    browser = playwright_instance.chromium.connect_over_cdp(cdp_url)
    context = browser.contexts[0]
    raw_page = context.pages[0] if context.pages else context.new_page()
    page = AerolithPageMixin(raw_page, agent_port=agent_port)
    return browser, page

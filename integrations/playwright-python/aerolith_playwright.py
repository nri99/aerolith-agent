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

    def screenshot_element(self, target: str, path: Optional[str] = None, padding: int = 0) -> bytes:
        """Capture an exact cropped visual screenshot of an element bounding box with optional padding."""
        payload = {"target": str(target), "padding": int(padding)}
        res = self._http_post("/screenshot/element", payload)
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

    def get_page_outline(self, max_links: int = 25, max_buttons: int = 25) -> Dict[str, Any]:
        """Retrieve compact semantic outline of the page: headings hierarchy, forms, landmarks, buttons, and links."""
        payload = {"max_links": int(max_links), "max_buttons": int(max_buttons)}
        res = self._http_post("/outline", payload)
        return res.get("data", {})

    def wait_for_network_idle(
        self,
        idle_time_ms: int = 500,
        timeout_ms: int = 10000,
        max_inflight: int = 0,
    ) -> Dict[str, Any]:
        """Wait until active network requests remain at or below max_inflight for consecutive idle_time_ms."""
        payload = {
            "idle_time_ms": int(idle_time_ms),
            "timeout_ms": int(timeout_ms),
            "max_inflight": int(max_inflight),
        }
        res = self._http_post("/wait_for_idle", payload)
        return res.get("data", {})

    def watch_mutations(self, action: str = "get", target: Optional[str] = None) -> Dict[str, Any]:
        """Control and query DOM mutation records (start, stop, clear, get)."""
        payload: Dict[str, Any] = {"action": str(action)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/mutations", payload)
        return res.get("data", {})

    def tab_eval(self, tab_index: int, script: str) -> Any:
        """Evaluate a JavaScript expression directly in a specific tab by index without switching active GUI focus."""
        payload = {"tab_index": int(tab_index), "script": str(script)}
        res = self._http_post("/tabs/eval", payload)
        return res.get("data", {}).get("result")

    def get_response_body(self, pattern: str = "", clear: bool = False) -> Dict[str, Any]:
        """Search recorded network activity for requests matching a URL pattern and return the captured response body."""
        payload = {"pattern": str(pattern), "clear": bool(clear)}
        res = self._http_post("/network/body", payload)
        return res.get("data", {})

    def drop_file(
        self,
        target: str,
        file_name: str,
        content: str,
        mime_type: str = "application/octet-stream",
    ) -> Dict[str, Any]:
        """Dispatch a synthetic file drag-and-drop or file input upload with custom metadata and content."""
        payload = {
            "target": str(target),
            "file_name": str(file_name),
            "mime_type": str(mime_type),
            "content": str(content),
        }
        res = self._http_post("/drop_file", payload)
        return res.get("data", {})

    def get_dom_tree(self, target: Optional[str] = None, max_depth: int = 4) -> Dict[str, Any]:
        """Serialize a nested hierarchical JSON tree of visible DOM nodes starting from root or target selector."""
        payload: Dict[str, Any] = {"max_depth": int(max_depth)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/dom/tree", payload)
        return res.get("data", {}).get("tree", {})

    def set_offline(self, offline: bool = True) -> Dict[str, Any]:
        """Instantly simulate network offline or online state via CDP Network emulation."""
        res = self._http_post("/network/offline", {"offline": bool(offline)})
        return res.get("data", {})

    def wait_for_text(self, text: str, target: Optional[str] = None, timeout_ms: int = 5000) -> Dict[str, Any]:
        """Wait until a specific text substring appears in an element or document body."""
        payload: Dict[str, Any] = {"text": str(text), "timeout_ms": int(timeout_ms)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/wait_for_text", payload)
        return res.get("data", {})

    def get_computed_styles(self, target: str, properties: Optional[List[str]] = None) -> Dict[str, Any]:
        """Read computed CSS style properties (e.g. display, visibility, opacity, color, z-index) for an element."""
        payload: Dict[str, Any] = {"target": str(target)}
        if properties is not None:
            payload["properties"] = list(properties)
        res = self._http_post("/computed_styles", payload)
        return res.get("data", {}).get("styles", {})

    def wait_for_download(self, file_pattern: Optional[str] = None, timeout_ms: int = 15000) -> Dict[str, Any]:
        """Wait for a new file download initiated by the page to appear, optionally matching a filename pattern."""
        payload: Dict[str, Any] = {"timeout_ms": int(timeout_ms)}
        if file_pattern is not None:
            payload["file_pattern"] = str(file_pattern)
        res = self._http_post("/wait_for_download", payload)
        return res.get("data", {})

    def set_content(self, html: str) -> Dict[str, Any]:
        """Set the HTML content of the active page directly via document.write, replacing the current document."""
        res = self._http_post("/set_content", {"html": str(html)})
        return res.get("data", {})

    def get_forms(self, target: Optional[str] = None) -> List[Dict[str, Any]]:
        """Inspect and structure all HTML forms and their controls (input, select, textarea, button) on the active page."""
        payload: Dict[str, Any] = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/forms", payload)
        return res.get("data", {}).get("forms", [])

    def set_visibility(self, state: str = "visible") -> Dict[str, Any]:
        """Force-emulate document.visibilityState ('visible' or 'hidden') and trigger visibilitychange event."""
        res = self._http_post("/visibility", {"state": str(state)})
        return res.get("data", {})

    def emulate_media(self, media_type: Optional[str] = None, color_scheme: Optional[str] = None) -> Dict[str, Any]:
        """Emulate media color scheme ('dark', 'light', 'no-preference') and/or media type ('screen', 'print') via CDP."""
        payload: Dict[str, Any] = {}
        if media_type is not None:
            payload["media_type"] = str(media_type)
        if color_scheme is not None:
            payload["color_scheme"] = str(color_scheme)
        res = self._http_post("/emulate/media", payload)
        return res.get("data", {})

    def get_element_bounds(self, target: str) -> Dict[str, Any]:
        """Extract exact pixel bounding box coordinates (x, y, width, height, center_x, center_y) and visibility metrics."""
        res = self._http_post("/element/bounds", {"target": str(target)})
        return res.get("data", {}).get("bounds", {})

    def mock_route(
        self,
        pattern: str,
        body: Optional[str] = None,
        status: int = 200,
        content_type: str = "application/json",
        action: str = "set",
    ) -> Dict[str, Any]:
        """Mock network responses for fetch/XHR requests matching a URL pattern."""
        payload: Dict[str, Any] = {
            "action": str(action),
            "pattern": str(pattern),
            "status": int(status),
            "content_type": str(content_type),
        }
        if body is not None:
            payload["body"] = str(body)
        res = self._http_post("/network/mock", payload)
        return res.get("data", {})

    def clear_mock_routes(self) -> Dict[str, Any]:
        """Clear all active network mock routes."""
        res = self._http_post("/network/mock", {"action": "clear"})
        return res.get("data", {})

    def get_viewport(self) -> Dict[str, Any]:
        """Retrieve active viewport dimensions, scroll offsets, screen metrics, and device pixel ratio."""
        res = self._http_get("/viewport")
        return res.get("data", {}).get("viewport", {})

    def set_element_attribute(
        self, target: str, attribute: str, value: Optional[str] = None
    ) -> Dict[str, Any]:
        """Dynamically set or remove an HTML attribute on a DOM element for self-healing automation."""
        payload: Dict[str, Any] = {
            "target": str(target),
            "attribute": str(attribute),
        }
        if value is not None:
            payload["value"] = str(value)
        res = self._http_post("/element/attribute", payload)
        return res.get("data", {})

    def search_page_text(
        self, query: str, max_matches: int = 20, case_sensitive: bool = False
    ) -> List[Dict[str, Any]]:
        """Search all visible text content on the page and return matched snippets, ref IDs, and bounds."""
        payload = {
            "query": query,
            "max_matches": max_matches,
            "case_sensitive": case_sensitive,
        }
        res = self._http_post("/search/text", payload)
        return res.get("data", {}).get("matches", [])

    def dump_storage(self, storage_type: str = "all") -> Dict[str, Any]:
        """Dump all localStorage and/or sessionStorage key-value pairs in a single structured payload."""
        res = self._http_get(f"/storage/dump?type={storage_type}")
        return res.get("data", {})

    def human_jitter(
        self, duration_ms: int = 800, move_mouse: bool = True, micro_scroll: bool = True
    ) -> Dict[str, Any]:
        """Simulate natural mouse micro-movements and subtle scrolling for anti-bot behavioral compliance."""
        payload = {
            "duration_ms": duration_ms,
            "move_mouse": move_mouse,
            "micro_scroll": micro_scroll,
        }
        res = self._http_post("/human_jitter", payload)
        return res.get("data", {})

    def type_keystrokes(
        self, target: str, text: str, delay_ms: int = 25, clear: bool = False, submit: bool = False
    ) -> Dict[str, Any]:
        """Type text character-by-character into target element with realistic delays and KeyboardEvents."""
        payload = {
            "target": str(target),
            "text": text,
            "delay_ms": delay_ms,
            "clear": clear,
            "submit": submit,
        }
        res = self._http_post("/type/keystrokes", payload)
        return res.get("data", {})

    def get_canvas_data(
        self, target: str, mime_type: str = "image/png", quality: float = 0.92
    ) -> Dict[str, Any]:
        """Extract base64 image data URL from <canvas>, <img>, or arbitrary element via rasterization."""
        payload = {
            "target": str(target),
            "mime_type": mime_type,
            "quality": quality,
        }
        res = self._http_post("/canvas/data", payload)
        return res.get("data", {})

    def get_resource_timing(
        self, initiator_type: Optional[str] = None, max_entries: int = 50, min_duration_ms: float = 0.0
    ) -> List[Dict[str, Any]]:
        """Extract structured resource timing and transfer bandwidth metrics from window.performance."""
        payload: Dict[str, Any] = {
            "max_entries": max_entries,
            "min_duration_ms": min_duration_ms,
        }
        if initiator_type is not None:
            payload["initiator_type"] = initiator_type
        res = self._http_post("/performance/resources", payload)
        return res.get("data", {}).get("resources", [])

    def get_dom_stats(self) -> Dict[str, Any]:
        """Inspect structural DOM complexity metrics (element count, tree depth, forms, buttons, links, inputs)."""
        res = self._http_get("/dom/stats")
        return res.get("data", {})

    def scroll_by(self, dx: int = 0, dy: int = 0, smooth: bool = False) -> Dict[str, Any]:
        """Perform relative window scroll by horizontal (dx) and vertical (dy) pixel deltas."""
        payload = {
            "dx": int(dx),
            "dy": int(dy),
            "smooth": bool(smooth),
        }
        res = self._http_post("/scroll/by", payload)
        return res.get("data", {})

    def scroll_container(
        self, target: str, dx: int = 0, dy: int = 0, smooth: bool = False
    ) -> Dict[str, Any]:
        """Scroll a specific inner scrollable container (e.g. div, modal, textarea, section)."""
        payload = {
            "target": str(target),
            "dx": int(dx),
            "dy": int(dy),
            "smooth": bool(smooth),
        }
        res = self._http_post("/scroll/container", payload)
        return res.get("data", {})

    def select_element_text(
        self, target: str, start: Optional[int] = None, length: Optional[int] = None
    ) -> Dict[str, Any]:
        """Select text range inside an input, textarea, or contentEditable element."""
        payload: Dict[str, Any] = {"target": str(target)}
        if start is not None:
            payload["start"] = start
        if length is not None:
            payload["length"] = length
        res = self._http_post("/element/select_text", payload)
        return res.get("data", {})

    def extract_page_links(
        self, max_links: int = 100, external_only: bool = False, filter_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Extract structured, deduplicated hyperlinks graph from active document with external origin detection."""
        payload: Dict[str, Any] = {
            "max_links": max_links,
            "external_only": external_only,
        }
        if filter_path is not None:
            payload["filter_path"] = filter_path
        res = self._http_post("/links", payload)
        return res.get("data", {})

    def get_element_state(self, target: str) -> Dict[str, Any]:
        """Atomically inspect the complete interactive, disabled, focused, checked, and visibility state of an element."""
        res = self._http_get(f"/element/state?target={target}")
        return res.get("data", {})

    def toggle_element_class(
        self, target: str, class_name: str, action: str = "toggle"
    ) -> Dict[str, Any]:
        """Add, remove, toggle, or check a CSS class name on the target DOM element."""
        payload = {
            "target": str(target),
            "class_name": str(class_name),
            "action": str(action),
        }
        res = self._http_post("/element/class", payload)
        return res.get("data", {})

    def set_element_text(self, target: str, text: str) -> Dict[str, Any]:
        """Directly overwrite the textContent of target element and dispatch input/change events."""
        payload = {
            "target": str(target),
            "text": str(text),
        }
        res = self._http_post("/element/text/set", payload)
        return res.get("data", {})

    def dispatch_custom_event(
        self,
        target: str,
        event_type: str,
        bubbles: bool = True,
        cancelable: bool = True,
        detail: Optional[Dict[str, Any]] = None,
    ) -> Dict[str, Any]:
        """Construct and dispatch a synthetic CustomEvent on target element with optional detail payload."""
        payload: Dict[str, Any] = {
            "target": str(target),
            "event_type": str(event_type),
            "bubbles": bool(bubbles),
            "cancelable": bool(cancelable),
        }
        if detail is not None:
            payload["detail"] = detail
        res = self._http_post("/element/dispatch_event", payload)
        return res.get("data", {})

    def get_active_element(self) -> Dict[str, Any]:
        """Inspect the currently focused DOM element (document.activeElement)."""
        res = self._http_get("/active_element")
        return res.get("data", {})

    def check_element_clickable(self, target: str) -> Dict[str, Any]:
        """Verify whether an element is physically clickable at its center point or occluded by overlays/modals."""
        res = self._http_post("/element/clickable", {"target": str(target)})
        return res.get("data", {})

    def dispatch_wheel_event(
        self,
        target: str,
        delta_x: float = 0.0,
        delta_y: float = 0.0,
        ctrl_key: bool = False,
    ) -> Dict[str, Any]:
        """Construct and dispatch a synthetic WheelEvent on target element (for map zooms, pan widgets, and charts)."""
        payload = {
            "target": str(target),
            "delta_x": float(delta_x),
            "delta_y": float(delta_y),
            "ctrl_key": bool(ctrl_key),
        }
        res = self._http_post("/element/wheel", payload)
        return res.get("data", {})

    def get_css_variables(
        self,
        target: Optional[str] = None,
        variable_names: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Resolve computed values of CSS custom properties (--var) on an element or root document."""
        payload: Dict[str, Any] = {}
        if target is not None:
            payload["target"] = str(target)
        if variable_names is not None:
            payload["variable_names"] = [str(v) for v in variable_names]
        res = self._http_post("/css/variables", payload)
        return res.get("data", {})

    def clear_highlights(self) -> Dict[str, Any]:
        """Remove all agent visual highlight overlays and badge borders from the page DOM."""
        res = self._http_post("/highlights/clear", {})
        return res.get("data", {})

    def simulate_pointer_click(
        self,
        target: str,
        click_count: int = 1,
        button: int = 0,
    ) -> Dict[str, Any]:
        """Dispatch a full sequence of PointerEvents (pointerdown, mousedown, pointerup, mouseup, click) at target center."""
        payload = {
            "target": str(target),
            "click_count": int(click_count),
            "button": int(button),
        }
        res = self._http_post("/element/pointer_click", payload)
        return res.get("data", {})

    def extract_meta_tags(self, include_json_ld: bool = True) -> Dict[str, Any]:
        """Extract structured page metadata, OpenGraph tags, Twitter cards, links, and JSON-LD schemas."""
        res = self._http_post("/meta/tags", {"include_json_ld": bool(include_json_ld)})
        return res.get("data", {})

    def get_document_geometry(self) -> Dict[str, Any]:
        """Extract complete document dimensions, viewport metrics, scroll offsets, and scrollable container count."""
        res = self._http_get("/document/geometry")
        return res.get("data", {})

    def set_device_orientation(
        self,
        alpha: float = 0.0,
        beta: float = 0.0,
        gamma: float = 0.0,
        absolute: bool = False,
    ) -> Dict[str, Any]:
        """Emulate mobile device orientation tilt angles (alpha, beta, gamma) and dispatch deviceorientation event."""
        payload = {
            "alpha": float(alpha),
            "beta": float(beta),
            "gamma": float(gamma),
            "absolute": bool(absolute),
        }
        res = self._http_post("/emulate/orientation", payload)
        return res.get("data", {})

    def clear_browser_cache(
        self,
        clear_storage: bool = False,
        clear_cookies: bool = False,
    ) -> Dict[str, Any]:
        """Purge browser in-memory/disk caches via CDP, and optionally clear localStorage, sessionStorage, and cookies."""
        payload = {
            "clear_storage": bool(clear_storage),
            "clear_cookies": bool(clear_cookies),
        }
        res = self._http_post("/cache/clear", payload)
        return res.get("data", {})

    def get_element_spacing(self, target: str) -> Dict[str, Any]:
        """Inspect box-model computed spacing (margin, padding, border, dimensions, z-index, box-sizing) for an element."""
        res = self._http_post("/element/spacing", {"target": str(target)})
        return res.get("data", {})

    def wait_for_url(self, pattern: str, timeout_ms: int = 5000) -> Dict[str, Any]:
        """Wait until window.location.href matches a substring or regex pattern."""
        payload = {
            "pattern": str(pattern),
            "timeout_ms": int(timeout_ms),
        }
        res = self._http_post("/wait_for_url", payload)
        return res.get("data", {})

    def dispatch_touch_event(
        self,
        event_type: str = "touchstart",
        touches: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        """Dispatch synthetic touch events (touchstart, touchmove, touchend, touchcancel) with multiple touch points."""
        payload = {
            "event_type": str(event_type),
            "touches": touches or [],
        }
        res = self._http_post("/touch", payload)
        return res.get("data", {})

    def get_a11y_tree(
        self,
        target: Optional[str] = None,
        max_depth: int = 5,
    ) -> Dict[str, Any]:
        """Extract a hierarchical computed WAI-ARIA accessibility (a11y) tree for navigation and screen reader analysis."""
        payload: Dict[str, Any] = {"max_depth": int(max_depth)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/a11y", payload)
        return res.get("data", {})

    def simulate_keyboard_chord(
        self,
        chord: str,
        target: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Dispatch a keyboard hotkey chord (e.g. 'Control+Shift+P', 'Meta+K', 'Alt+Tab') with keydown/keyup sequence."""
        payload: Dict[str, Any] = {"chord": str(chord)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/keyboard/chord", payload)
        return res.get("data", {})

    def highlight_bounding_boxes(
        self,
        boxes: List[Dict[str, Any]],
    ) -> Dict[str, Any]:
        """Render visual bounding boxes with label badges over page regions for grounding and multimodal verification."""
        res = self._http_post("/highlights/boxes", {"boxes": boxes})
        return res.get("data", {})

    def get_element_attributes(self, target: str) -> Dict[str, Any]:
        """Extract all HTML attributes of a DOM element as a complete key-value dictionary."""
        res = self._http_post("/element/attributes", {"target": str(target)})
        return res.get("data", {}).get("attributes", {})

    def get_element_filter(self, target: str) -> Dict[str, Any]:
        """Inspect CSS filters, backdrop filters, opacity, pointer-events, and ancestor occlusion."""
        res = self._http_post("/element/filter", {"target": str(target)})
        return res.get("data", {})

    def dismiss_all_modals(self) -> Dict[str, Any]:
        """Dismiss modal overlays, cookie consent popups, backdrop dialogs, and restore body scroll."""
        res = self._http_post("/modals/dismiss", {})
        return res.get("data", {})

    def get_color_palette(self, limit: int = 15) -> Dict[str, Any]:
        """Extract dominant computed colors and background colors used across visible elements on the page."""
        res = self._http_post("/palette", {"limit": int(limit)})
        return res.get("data", {})

    def drag_and_drop_element(
        self,
        source: str,
        target: str,
        steps: int = 10,
        hold_ms: int = 0,
    ) -> Dict[str, Any]:
        """Drag a source element and drop it onto a target element with multi-step pointer and drag event simulation."""
        payload = {
            "source": str(source),
            "target": str(target),
            "steps": int(steps),
            "hold_ms": int(hold_ms),
        }
        res = self._http_post("/drag_and_drop", payload)
        return res.get("data", {})

    def audit_forms(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Audit form elements, input validation constraints (required, pattern, min/max), and HTML5 validity states."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/forms/audit", payload)
        return res.get("data", {})

    def get_viewport_paging(self) -> Dict[str, Any]:
        """Calculate viewport dimensions, multi-page scroll coordinates, total pages, and current scroll progress."""
        res = self._http_post("/viewport/paging", {})
        return res.get("data", {})

    def get_computed_pseudo_element(
        self,
        target: str,
        pseudo: str = "::before",
        properties: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Inspect computed CSS styles of pseudo-elements like ::before or ::after (e.g. icons, badges, content strings)."""
        payload: Dict[str, Any] = {
            "target": str(target),
            "pseudo": str(pseudo),
        }
        if properties is not None:
            payload["properties"] = [str(p) for p in properties]
        res = self._http_post("/element/pseudo", payload)
        return res.get("data", {})

    def simulate_clipboard_paste(
        self,
        text: str,
        target: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Simulate a clipboard paste event into a target or active element with DataTransfer text payload."""
        payload: Dict[str, Any] = {"text": str(text)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/clipboard/paste", payload)
        return res.get("data", {})

    def smart_fill(self, mapping: Dict[str, Any], submit: bool = False) -> Dict[str, Any]:
        """Smartly auto-fill form controls by fuzzy matching human-readable labels, names, or placeholders to values."""
        payload = {
            "mapping": dict(mapping),
            "submit": bool(submit),
        }
        res = self._http_post("/forms/smart_fill", payload)
        return res.get("data", {})

    def observe_text(
        self,
        target: str,
        timeout_ms: int = 10000,
        settle_ms: int = 1000,
    ) -> Dict[str, Any]:
        """Observe streaming text changes on a target element until updates settle or timeout expires."""
        payload = {
            "target": str(target),
            "timeout_ms": int(timeout_ms),
            "settle_ms": int(settle_ms),
        }
        res = self._http_post("/observe/text", payload)
        return res.get("data", {})

    def set_visibility_state(self, state: str = "visible") -> Dict[str, Any]:
        """Override document.visibilityState ('visible'/'hidden') to prevent bot pausing or idle detection."""
        res = self._http_post("/emulate/visibility", {"state": str(state)})
        return res.get("data", {})

    def get_computed_font(self, target: str) -> Dict[str, Any]:
        """Extract computed font typography, font family, weight, line-height, and canvas measurement metrics."""
        res = self._http_post("/element/font", {"target": str(target)})
        return res.get("data", {})

    def get_action_map(self, viewport_only: bool = True, limit: int = 50) -> Dict[str, Any]:
        """Extract dense interactive element click-map with bounding box centroid coordinates and numeric ref bindings."""
        payload = {
            "viewport_only": bool(viewport_only),
            "limit": int(limit),
        }
        res = self._http_post("/action/map", payload)
        return res.get("data", {})

    def get_motion_state(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect active CSS animations and transitions on the page or target element to detect motion stability."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/motion/state", payload)
        return res.get("data", {})

    def get_element_listeners(self, target: str) -> Dict[str, Any]:
        """Inspect event listeners, inline handlers, cursor styles, and interactive affordances of an element."""
        res = self._http_post("/element/listeners", {"target": str(target)})
        return res.get("data", {})

    def get_web_vitals(self) -> Dict[str, Any]:
        """Extract Core Web Vitals, performance navigation timings (TTFB, FCP, DOMContentLoaded), and memory heap metrics."""
        res = self._http_post("/performance/vitals", {})
        return res.get("data", {})

    def get_select_options(self, target: str) -> Dict[str, Any]:
        """Inspect selectable options, option groups, values, text labels, and selected indices of a <select> element."""
        res = self._http_post("/select/options", {"target": str(target)})
        return res.get("data", {})

    def fuzzy_find_elements(
        self,
        query: str,
        limit: int = 10,
        container: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Fuzzy search interactive elements by text content, ARIA labels, or attributes with similarity scoring."""
        payload: Dict[str, Any] = {
            "query": str(query),
            "limit": int(limit),
        }
        if container is not None:
            payload["container"] = str(container)
        res = self._http_post("/elements/fuzzy", payload)
        return res.get("data", {})

    def get_heading_hierarchy(self, container: Optional[str] = None) -> Dict[str, Any]:
        """Extract hierarchical heading structure (h1-h6) with levels, text content, IDs, and anchor links."""
        payload = {}
        if container is not None:
            payload["container"] = str(container)
        res = self._http_post("/headings", payload)
        return res.get("data", {})

    def get_element_stacking(self, target: str) -> Dict[str, Any]:
        """Inspect CSS stacking context, z-index, position, opacity, transform, and ancestor clipping bounds."""
        res = self._http_post("/element/stacking", {"target": str(target)})
        return res.get("data", {})

    def query_shadow_elements(
        self,
        selector: str,
        limit: int = 50,
        container: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Recursively query elements across both light DOM and all open shadow roots."""
        payload: Dict[str, Any] = {
            "selector": str(selector),
            "limit": int(limit),
        }
        if container is not None:
            payload["container"] = str(container)
        res = self._http_post("/elements/shadow", payload)
        return res.get("data", {})

    def get_element_visibility_matrix(self, target: str) -> Dict[str, Any]:
        """Perform 5-point occlusion testing and compute visibility ratio against viewport and ancestor clipping."""
        res = self._http_post("/element/visibility_matrix", {"target": str(target)})
        return res.get("data", {})

    def get_error_digest(self, max_errors: int = 50, clear: bool = False) -> Dict[str, Any]:
        """Retrieve aggregated diagnostics of runtime JavaScript errors, unhandled rejections, and console errors."""
        payload = {
            "max_errors": int(max_errors),
            "clear": bool(clear),
        }
        res = self._http_post("/errors/digest", payload)
        return res.get("data", {})

    def extract_grid_data(self, target: Optional[str] = None, max_rows: int = 100) -> Dict[str, Any]:
        """Extract structured datasets from HTML tables and ARIA grids/treegrids into column-mapped row records."""
        payload: Dict[str, Any] = {"max_rows": int(max_rows)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/grid/data", payload)
        return res.get("data", {})

    def diff_form_inputs(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect all form inputs and compare pristine default values with current values to identify dirty fields."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/forms/diff", payload)
        return res.get("data", {})

    def unstick_scroll_locks(self) -> Dict[str, Any]:
        """Neutralize overflow:hidden locks, fixed position body wrappers, and invisible click backdrops."""
        res = self._http_post("/viewport/unstick", {})
        return res.get("data", {})

    def diff_dom_text(
        self,
        baseline: Optional[str] = None,
        target: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Compute text mutations and diff between current visible content and a baseline text snapshot."""
        payload: Dict[str, Any] = {}
        if baseline is not None:
            payload["baseline"] = str(baseline)
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/dom/diff", payload)
        return res.get("data", {})

    def get_frame_hierarchy(self) -> Dict[str, Any]:
        """Inspect embedded frames and iframes with cross-origin detection and viewport coordinates."""
        res = self._http_get("/frames/hierarchy")
        return res.get("data", {})

    def get_role_elements(
        self,
        roles: Optional[List[str]] = None,
        viewport_only: bool = True,
        limit: int = 50,
    ) -> Dict[str, Any]:
        """Filter and extract interactive elements matching specific ARIA roles or semantics."""
        payload: Dict[str, Any] = {
            "viewport_only": bool(viewport_only),
            "limit": int(limit),
        }
        if roles is not None:
            payload["roles"] = [str(r) for r in roles]
        res = self._http_post("/elements/roles", payload)
        return res.get("data", {})

    def harvest_resources(self, resource_types: Optional[List[str]] = None) -> Dict[str, Any]:
        """Harvest all page resources (images, scripts, styles, media, and iframes) with resolved absolute URLs."""
        payload = {}
        if resource_types is not None:
            payload["resource_types"] = [str(t) for t in resource_types]
        res = self._http_post("/page/resources", payload)
        return res.get("data", {})

    def detect_bot_challenges(self) -> Dict[str, Any]:
        """Scan page for active bot challenges, CAPTCHAs (Turnstile, reCAPTCHA, hCaptcha), and WAF blocks."""
        res = self._http_get("/security/challenges")
        return res.get("data", {})

    def get_scroll_state(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect scrollability metrics, directional scroll boundaries, and progress for an element or document."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/scroll/state", payload)
        return res.get("data", {})

    def get_proximity_elements(
        self,
        target: Optional[str] = None,
        x: Optional[float] = None,
        y: Optional[float] = None,
        limit: int = 10,
        radius: Optional[float] = None,
        interactive_only: bool = True,
    ) -> Dict[str, Any]:
        """Find and rank nearby interactive elements sorted by Euclidean distance from an anchor point or element."""
        payload: Dict[str, Any] = {
            "limit": int(limit),
            "interactive_only": bool(interactive_only),
        }
        if target is not None:
            payload["target"] = str(target)
        if x is not None:
            payload["x"] = float(x)
        if y is not None:
            payload["y"] = float(y)
        if radius is not None:
            payload["radius"] = float(radius)
        res = self._http_post("/elements/proximity", payload)
        return res.get("data", {})

    def emulate_screen_metrics(
        self,
        width: Optional[int] = None,
        height: Optional[int] = None,
        device_scale_factor: Optional[float] = None,
        mobile: bool = False,
        orientation: Optional[str] = None,
        color_scheme: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Emulate responsive viewport dimensions, scale factor, screen orientation, and color scheme."""
        payload: Dict[str, Any] = {
            "mobile": bool(mobile),
        }
        if width is not None:
            payload["width"] = int(width)
        if height is not None:
            payload["height"] = int(height)
        if device_scale_factor is not None:
            payload["device_scale_factor"] = float(device_scale_factor)
        if orientation is not None:
            payload["orientation"] = str(orientation)
        if color_scheme is not None:
            payload["color_scheme"] = str(color_scheme)
        res = self._http_post("/screen/metrics", payload)
        return res.get("data", {})

    def export_table_csv(
        self,
        target: Optional[str] = None,
        include_headers: bool = True,
        delimiter: str = ",",
    ) -> Dict[str, Any]:
        """Extract structured table or grid elements formatted directly as clean RFC 4180 CSV."""
        payload: Dict[str, Any] = {
            "include_headers": bool(include_headers),
            "delimiter": str(delimiter),
        }
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/table/csv", payload)
        return res.get("data", {})

    def reset_form(
        self,
        target: Optional[str] = None,
        trigger_events: bool = True,
        clear_custom_validations: bool = True,
    ) -> Dict[str, Any]:
        """Reset target form or container form controls back to pristine default values."""
        payload: Dict[str, Any] = {
            "trigger_events": bool(trigger_events),
            "clear_custom_validations": bool(clear_custom_validations),
        }
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/form/reset", payload)
        return res.get("data", {})

    def get_datalist_options(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect selectable options in an HTML5 datalist or linked autocomplete input combobox."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/elements/datalist", payload)
        return res.get("data", {})

    def get_canvas_context(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect canvas rendering context (2D, WebGL, WebGL2), dimensions, and renderer hardware metrics."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/canvas/context", payload)
        return res.get("data", {})

    def control_media_playback(
        self,
        action: str = "status",
        target: Optional[str] = None,
        value: Optional[float] = None,
    ) -> Dict[str, Any]:
        """Inspect and control HTML5 audio/video playback elements with playback state telemetry."""
        payload: Dict[str, Any] = {
            "action": str(action),
        }
        if target is not None:
            payload["target"] = str(target)
        if value is not None:
            payload["value"] = float(value)
        res = self._http_post("/media/playback", payload)
        return res.get("data", {})

    def get_element_ancestry(self, target: str, max_depth: int = 20) -> Dict[str, Any]:
        """Inspect the full DOM ancestry tree and selector hierarchy path from an element up to document root."""
        payload = {
            "target": str(target),
            "max_depth": int(max_depth),
        }
        res = self._http_post("/elements/ancestry", payload)
        return res.get("data", {})

    def get_layout_metrics(self, target: Optional[str] = None) -> Dict[str, Any]:
        """Inspect CSS Grid and Flexbox layout metrics, alignments, tracks, and child coordinates."""
        payload = {}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/layout/metrics", payload)
        return res.get("data", {})

    def audit_contrast_ratio(self, target: str) -> Dict[str, Any]:
        """Compute WCAG 2.1 relative luminance, contrast ratio, and accessibility compliance."""
        payload = {
            "target": str(target),
        }
        res = self._http_post("/accessibility/contrast", payload)
        return res.get("data", {})

    def dispatch_advanced_event(
        self,
        target: str,
        event_type: str = "click",
        modifiers: Optional[List[str]] = None,
        button: int = 0,
        pressure: float = 0.5,
        detail: int = 1,
    ) -> Dict[str, Any]:
        """Dispatch PointerEvent, MouseEvent, or KeyboardEvent with synthetic modifier keys."""
        payload: Dict[str, Any] = {
            "target": str(target),
            "event_type": str(event_type),
            "button": int(button),
            "pressure": float(pressure),
            "detail": int(detail),
        }
        if modifiers is not None:
            payload["modifiers"] = [str(m) for m in modifiers]
        res = self._http_post("/events/dispatch_advanced", payload)
        return res.get("data", {})

    def get_network_har(self, clear: bool = False) -> Dict[str, Any]:
        """Harvest intercepted network activity logs formatted as a standard HAR 1.2 archive."""
        payload = {
            "clear": bool(clear),
        }
        res = self._http_post("/network/har", payload)
        return res.get("data", {})

    def get_caret_position(
        self,
        target: str,
        set_start: Optional[int] = None,
        set_end: Optional[int] = None,
    ) -> Dict[str, Any]:
        """Inspect or set the caret position, selection offsets, and selected text in an element."""
        payload = {"target": str(target)}
        if set_start is not None:
            payload["set_start"] = int(set_start)
        if set_end is not None:
            payload["set_end"] = int(set_end)
        res = self._http_post("/caret/position", payload)
        return res.get("data", {})

    def dismiss_cookie_banners(self, click_accept: bool = True) -> Dict[str, Any]:
        """Auto-detect and click Accept/Consent or Reject buttons on popular CMP cookie consent dialogs."""
        payload = {"click_accept": bool(click_accept)}
        res = self._http_post("/consent/dismiss", payload)
        return res.get("data", {})

    def emulate_speech_api(
        self,
        action: Optional[str] = None,
        mock_transcript: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Emulate Web Speech API to feed synthetic voice recognition transcripts or control recognition state."""
        payload = {}
        if action is not None:
            payload["action"] = str(action)
        if mock_transcript is not None:
            payload["mock_transcript"] = str(mock_transcript)
        res = self._http_post("/speech/emulate", payload)
        return res.get("data", {})

    def get_dom_mutations(self, limit: int = 50, clear: bool = True) -> Dict[str, Any]:
        """Harvest real-time DOM mutation logs captured by the background MutationObserver."""
        payload = {
            "limit": int(limit),
            "clear": bool(clear),
        }
        res = self._http_post("/dom/mutations", payload)
        return res.get("data", {})

    def get_active_routes(self) -> Dict[str, Any]:
        """Inspect active route interception, mock, and abort rules registered in the active tab."""
        res = self._http_get("/routes/active")
        return res.get("data", {})

    def exec_editor_command(
        self,
        command: str,
        value: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Execute native rich text editor commands via document.execCommand."""
        payload = {"command": str(command)}
        if value is not None:
            payload["value"] = str(value)
        res = self._http_post("/editor/command", payload)
        return res.get("data", {})

    def preload_lazy_resources(
        self,
        force_all: bool = True,
        container: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Force hydration and eager preloading of lazy-loaded images, sources, and background media."""
        payload = {"force_all": bool(force_all)}
        if container is not None:
            payload["container"] = str(container)
        res = self._http_post("/resources/preload-lazy", payload)
        return res.get("data", {})

    def emulate_window_focus(
        self,
        focused: bool = True,
        emit_events: bool = True,
    ) -> Dict[str, Any]:
        """Emulate window and document focus state with optional focus/blur event dispatch."""
        payload = {
            "focused": bool(focused),
            "emit_events": bool(emit_events),
        }
        res = self._http_post("/window/focus", payload)
        return res.get("data", {})

    def send_key_sequence(
        self,
        keys: Any,
        target: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Dispatch an atomic sequence of keyboard shortcuts, key chords, or keystrokes."""
        if isinstance(keys, str):
            key_list = [k.strip() for k in keys.split(",") if k.strip()]
        else:
            key_list = [str(k) for k in keys]
        payload = {"keys": key_list}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/keyboard/sequence", payload)
        return res.get("data", {})

    def diff_element_styles(
        self,
        target: str,
        baseline: Optional[Dict[str, str]] = None,
        properties: Optional[List[str]] = None,
    ) -> Dict[str, Any]:
        """Inspect and diff computed CSS styles against an optional baseline to verify UI changes."""
        payload = {"target": str(target)}
        if baseline is not None:
            payload["baseline"] = {str(k): str(v) for k, v in baseline.items()}
        if properties is not None:
            payload["properties"] = [str(p) for p in properties]
        res = self._http_post("/styles/diff", payload)
        return res.get("data", {})

    def dispatch_touch_gesture(
        self,
        gesture: str = "pinch_zoom",
        start_x1: float = 100.0,
        start_y1: float = 100.0,
        start_x2: float = 200.0,
        start_y2: float = 200.0,
        end_x1: Optional[float] = None,
        end_y1: Optional[float] = None,
        end_x2: Optional[float] = None,
        end_y2: Optional[float] = None,
        steps: int = 10,
        target: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Simulate realistic multi-touch gestures like pinch-to-zoom and two-finger pans."""
        payload = {
            "gesture": str(gesture),
            "start_x1": float(start_x1),
            "start_y1": float(start_y1),
            "start_x2": float(start_x2),
            "start_y2": float(start_y2),
            "end_x1": float(end_x1 if end_x1 is not None else start_x1),
            "end_y1": float(end_y1 if end_y1 is not None else start_y1),
            "end_x2": float(end_x2 if end_x2 is not None else start_x2),
            "end_y2": float(end_y2 if end_y2 is not None else start_y2),
            "steps": int(steps),
        }
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/touch/gesture", payload)
        return res.get("data", {})

    def inspect_storage_quota(
        self,
        include_indexed_db: bool = True,
        include_cache_api: bool = True,
    ) -> Dict[str, Any]:
        """Inspect storage quota estimates, Web Storage usage, Cache API, and IndexedDB inventories."""
        payload = {
            "include_indexed_db": bool(include_indexed_db),
            "include_cache_api": bool(include_cache_api),
        }
        res = self._http_post("/storage/quota", payload)
        return res.get("data", {})

    def inspect_element_readiness(self, target: str) -> Dict[str, Any]:
        """Inspect actionable readiness of an element (in_dom, displayed, visible, occluded, animating)."""
        payload = {"target": str(target)}
        res = self._http_post("/element/readiness", payload)
        return res.get("data", {})

    def wait_for_fonts_and_layout_settled(self, timeout_ms: int = 5000) -> Dict[str, Any]:
        """Wait for web font assets to finish loading and layout dimensions to stabilize."""
        payload = {"timeout_ms": int(timeout_ms)}
        res = self._http_post("/page/settled", payload)
        return res.get("data", {})

    def emulate_page_scale_factor(self, scale_factor: float = 1.0) -> Dict[str, Any]:
        """Emulate page zoom scaling factor and override window.devicePixelRatio."""
        payload = {"scale_factor": float(scale_factor)}
        res = self._http_post("/page/scale", payload)
        return res.get("data", {})

    def copy_formatted_content(
        self,
        target: Optional[str] = None,
        format: str = "markdown",
    ) -> Dict[str, Any]:
        """Extract structured formatted content (markdown, clean html, or citation text)."""
        payload = {"format": str(format)}
        if target is not None:
            payload["target"] = str(target)
        res = self._http_post("/content/formatted", payload)
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

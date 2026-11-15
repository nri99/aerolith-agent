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

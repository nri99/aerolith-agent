# Aerolith Agent — Playwright Node.js SDK

First-class Node.js & TypeScript Playwright integration for **Aerolith Agent**. Connects via standard Chrome DevTools Protocol (`chromium.connectOverCDP`) and automatically enriches `page` with Aerolith's token-saving, anti-bot, and visual grounding extensions.

## Installation

```bash
npm install playwright
```

*(No external dependencies needed — uses native Node.js `http` and `fs`).*

## Quickstart

Start the Aerolith Agent browser:
```bash
aerolith-agent serve --port 8765 --cdp-port 9222
```

In your Node.js script:
```javascript
const playwright = require('playwright');
const { connectAerolith } = require('./index');

(async () => {
  const { browser, page } = await connectAerolith(playwright, {
    cdpUrl: 'http://127.0.0.1:9222',
    agentPort: 8765
  });

  await page.goto('https://github.com/login');

  // 1. Capture token-efficient accessibility snapshot
  const snapshot = await page.semanticSnapshot({ viewportOnly: true });
  console.log(snapshot);

  // 2. Visually highlight element to verify grounding
  await page.highlight('input[name=login]');

  // 3. Atomically fill multiple fields in one turn
  await page.fillForm([
    { target: 'input[name=login]', value: 'my-username' },
    { target: 'input[name=password]', value: 'my-password' }
  ], { submit: true });

  // 4. Capture Set-Of-Marks numbered bounding boxes for vision models
  await page.screenshotSom({ path: 'som.png' });

  await browser.close();
})();
```

## Extended API Reference

| Method | Description |
|---|---|
| `page.semanticSnapshot({ viewportOnly, role })` | Returns compact accessibility text tree with numbered target IDs. |
| `page.highlight(target)` | Injects an animated focus ring around element to confirm visual grounding. |
| `page.screenshotElement(target, { path, padding })` | Captures exact bounding-box cropped PNG image buffer with optional padding. |
| `page.getText(target, attribute)` | Extracts element text content, input value, or specific HTML attribute. |
| `page.hover(target)` | Hovers mouse over element to trigger flyouts, tooltips, and `:hover` states. |
| `page.pressKey(key, { target })` | Dispatches realistic keyboard events (`Escape`, `Enter`, `Tab`, etc.). |
| `page.selectOption(target, value)` | Selects an option in a `<select>` dropdown by value, label, or index. |
| `page.drag(source, { target, offsetX, offsetY, steps })` | Drags an element or adjusts a range slider with smooth interpolation. |
| `page.getCookies()` | Retrieves all cookies (including `HttpOnly` and secure flags) via CDP. |
| `page.setCookie(cookie)` | Sets a cookie object via CDP. |
| `page.deleteCookie(name, domain)` | Deletes a cookie by name and domain via CDP. |
| `page.clearCookies()` | Clears all browser cookies via CDP. |
| `page.setExtraHeaders(headers)` | Sets persistent custom HTTP request headers via CDP. |
| `page.setUserAgent(userAgent, { acceptLanguage, platform })` | Overrides browser User-Agent and language via CDP. |
| `page.uploadFile(target, files)` | Sets files on `<input type="file">` elements via synthetic DataTransfer. |
| `page.setLocation(lat, lng, { accuracy, timezone })` | Overrides geographical coordinates and timezone via CDP and geolocation APIs. |
| `page.mediaControl(action)` | Controls or queries audio and video playback on the page. |
| `page.pauseMedia()` | Pauses all video and audio playback across the page. |
| `page.inspect(target)` | Returns complete attributes, computed CSS styles, viewport visibility, and bounding box. |
| `page.routeAbort(pattern)` | Dynamically intercepts and aborts network requests matching a URL pattern. |
| `page.routeMock(pattern, options)` | Dynamically intercepts and mocks responses matching a URL pattern. |
| `page.routeClear()` | Clears all dynamic route interception rules. |
| `page.waitForNetworkIdle(timeoutMs)` | Waits until all network requests have settled for at least 250ms. |
| `page.waitForUrl(pattern, timeoutMs)` | Waits until page URL matches or contains the target substring. |
| `page.waitForFunction(jsCode, timeoutMs)` | Waits until a JavaScript condition evaluates to truthy. |
| `page.getTabs()` | Lists all open browser tabs and identifies the active tab index. |
| `page.newTab(url)` | Opens a new browser tab. |
| `page.switchTab(index)` | Switches active focus to a tab by its index. |
| `page.closeTab(index)` | Closes a tab by its index. |
| `page.getNetworkActivity({ clear })` | Retrieves recorded AJAX/fetch requests and in-flight count. |
| `page.fillForm(fields, { submit })` | Atomically populates multiple inputs and optionally submits. |
| `page.setViewport(width, height, { dpr, mobile })` | Emulates screen dimensions, device pixel ratio, and touch. |
| `page.getConsoleLogs({ clear })` | Retrieves buffered browser console logs (log, warn, error). |
| `page.printPdf({ path, landscape })` | Generates high-fidelity vector PDF via CDP Page.printToPDF. |
| `page.getMarkdown()` | Extracts clean, reader-friendly article markdown. |
| `page.screenshotSom({ path })` | Renders high-contrast Set-Of-Marks tags before saving screenshot. |
| `page.exportStorageState({ path })` | Exports cookies, `localStorage`, and `sessionStorage`. |
| `page.importStorageState(state)` | Restores cookies, `localStorage`, and `sessionStorage`. |
| `page.doubleClick(target)` | Simulates a human-like double click with sticky-header occlusion compensation. |
| `page.rightClick(target)` | Simulates a right-click / contextmenu event on target element. |
| `page.extractTable(target)` | Extracts structured rows and headers from an HTML table into clean JSON records. |
| `page.getDownloads({ clear })` | Retrieves recorded browser file downloads or clears history. |
| `page.emulateNetwork(options)` | Throttles network conditions (online, offline, slow-3g, fast-3g) via CDP. |
| `page.injectCss(css)` | Injects custom CSS rules to eliminate floating headers, banners, or occluding elements. |
| `page.clearCss()` | Removes previously injected custom CSS stylesheet. |
| `page.queryElements(selector, options)` | Batch queries all matching elements and returns indexed metadata records. |
| `page.focus(target)` | Explicitly focuses an element to trigger framework validation. |
| `page.blur(target)` | Explicitly blurs an element to trigger framework change events. |
| `page.getPageInfo()` | Retrieves page URL, title, viewport dimensions, document height, and scroll metrics. |
| `page.getHtml(target)` | Extracts raw HTML source for the entire document or a specific DOM element. |
| `page.scrollIntoView(target, align)` | Scrolls element into view with block alignment and sticky-header compensation. |
| `page.setZoom(zoomFactor)` | Sets browser rendering zoom factor with dual-layer CSS and CDP scaling. |
| `page.storageItem(action, options)` | Inspects or mutates granular localStorage or sessionStorage keys. |
| `page.assertElement(target, assertions)` | Verifies element state against declarative assertions (visible, hidden, disabled, checked, text, value, classes, attributes). |
| `page.waitForSelectorState(target, options)` | Waits for element to reach state: 'visible', 'hidden', 'attached', 'detached'. |
| `page.getPerformanceMetrics()` | Retrieves page performance metrics (TTFB, DOMContentLoaded, load time, resource breakdown, transfer bytes, JS heap). |
| `page.scrollToCoords(x, y, options)` | Scrolls viewport directly to specific absolute (x, y) coordinates. |
| `page.getPageOutline(options)` | Retrieves compact semantic outline of the page: headings, forms, landmarks, buttons, and links. |
| `page.waitForNetworkIdle(options)` | Waits until active network requests remain at or below max_inflight for consecutive idle_time_ms. |
| `page.watchMutations(action, options)` | Controls and queries DOM mutation records (start, stop, clear, get). |
| `page.tabEval(tabIndex, script)` | Evaluates a JavaScript expression directly in a specific tab by index without switching active GUI focus. |
| `page.getResponseBody(pattern, options)` | Searches recorded network activity for URL pattern and returns captured response body. |
| `page.dropFile(target, options)` | Simulates synthetic file drag-and-drop or file input upload with custom metadata. |
| `page.getDomTree(options)` | Serializes nested hierarchical JSON tree of visible DOM nodes. |
| `page.setOffline(offline)` | Instantly simulates network offline or online state via CDP Network emulation. |
| `page.waitForText(text, options)` | Waits until a specific text substring appears in an element or document body. |
| `page.getComputedStyles(target, properties)` | Reads computed CSS style properties (display, visibility, opacity, color, z-index, etc.). |
| `page.waitForDownload(options)` | Waits for a new file download initiated by the page to appear. |
| `page.setContent(html)` | Sets the HTML content of the active page directly via document.write. |
| `page.getForms(target)` | Inspects and structures all HTML forms and their controls (input, select, textarea, button). |
| `page.setVisibility(state)` | Force-emulates document.visibilityState ('visible' or 'hidden') and window focus. |
| `page.emulateMedia(options)` | Emulates media color scheme ('dark', 'light') and/or media type ('screen', 'print'). |
| `page.getElementBounds(target)` | Extracts exact pixel bounding box coordinates (x, y, width, height, center_x, center_y) and visibility metrics. |
| `page.mockRoute(pattern, options)` | Mocks network responses for fetch/XHR requests matching a URL pattern. |
| `page.clearMockRoutes()` | Clears all active network mock routes. |
| `page.getViewport()` | Retrieves active viewport dimensions, scroll offsets, screen metrics, and device pixel ratio. |
| `page.setElementAttribute(target, attribute, value)` | Dynamically sets or removes an HTML attribute on a DOM element for self-healing. |
| `page.searchPageText(query, options)` | Searches visible page text for substrings and returns matches, element refs, and bounding boxes. |
| `page.dumpStorage(storageType)` | Dumps all localStorage and/or sessionStorage key-value pairs in a single structured JSON payload. |
| `page.humanJitter(options)` | Simulates subtle mouse micro-movements and scroll adjustments to pass anti-bot behavioral heuristics. |
| `page.typeKeystrokes(target, text, options)` | Types text character-by-character with realistic key delays and complete KeyboardEvents. |
| `page.getCanvasData(target, options)` | Extracts base64 image data URL from <canvas>, <img>, or arbitrary element via rasterization. |
| `page.getResourceTiming(options)` | Extracts structured resource timing and transfer bandwidth metrics from window.performance. |
| `page.getDomStats()` | Inspects structural DOM complexity metrics (element count, tree depth, forms, buttons, links, inputs). |
| `page.scrollBy(dx, dy, options)` | Performs relative window scroll by horizontal (dx) and vertical (dy) pixel deltas. |
| `page.scrollContainer(target, dx, dy, options)` | Scrolls a specific inner scrollable container (div, modal, textarea, section). |
| `page.selectElementText(target, options)` | Selects text range inside an input, textarea, or contentEditable element. |
| `page.extractPageLinks(options)` | Extracts structured, deduplicated hyperlinks graph from active document. |
| `page.getElementState(target)` | Atomically inspects complete interactive, disabled, focused, checked, and visibility state. |
| `page.toggleElementClass(target, className, action)` | Adds, removes, toggles, or checks a CSS class name on the target DOM element. |
| `page.setElementText(target, text)` | Directly overwrites the textContent of target element and dispatches input/change events. |
| `page.dispatchCustomEvent(target, eventType, options)` | Constructs and dispatches a synthetic CustomEvent on target element. |
| `page.getActiveElement()` | Inspects the currently focused DOM element (document.activeElement). |
| `page.checkElementClickable(target)` | Verifies whether an element is physically clickable at its center point or occluded by overlays/modals. |
| `page.dispatchWheelEvent(target, options)` | Constructs and dispatches a synthetic WheelEvent on target element (for map zooms, pan widgets, and charts). |
| `page.getCssVariables(options)` | Resolves computed values of CSS custom properties (--var) on an element or root document. |
| `page.clearHighlights()` | Removes all agent visual highlight overlays and badge borders from the page DOM. |
| `page.simulatePointerClick(target, options)` | Dispatches full sequence of PointerEvents (pointerdown, mousedown, pointerup, mouseup, click) at target center. |
| `page.extractMetaTags(options)` | Extracts structured page metadata, OpenGraph tags, Twitter cards, links, and JSON-LD schemas. |
| `page.getDocumentGeometry()` | Extracts complete document dimensions, viewport metrics, scroll offsets, and scrollable container count. |
| `page.setDeviceOrientation(alpha, beta, gamma, options)` | Emulates mobile device orientation tilt angles (alpha, beta, gamma) and dispatches deviceorientation event. |
| `page.clearBrowserCache(options)` | Purges browser in-memory and disk caches via CDP, with optional storage/cookie reset. |
| `page.getElementSpacing(target)` | Inspects box-model computed spacing (margin, padding, border, dimensions, z-index, box-sizing) for an element. |
| `page.waitForUrl(pattern, options)` | Waits until window.location.href matches a target substring or regex pattern. |
| `page.dispatchTouchEvent(eventType, touches)` | Dispatches synthetic touch events (touchstart, touchmove, touchend, touchcancel) with multiple touch points. |

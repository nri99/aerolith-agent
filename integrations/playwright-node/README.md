# Aerolith Agent — Playwright Node.js SDK

Set `AEROLITH_AGENT_TOKEN` to the same secret (at least 32 printable ASCII characters) in both browser and client environments. Start the browser with `aerolith-agent serve --cdp` to enable the separate, unauthenticated CDP port for trusted local clients. HTTP and command failures raise errors instead of returning empty results.


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
  const snapshot = await page.aerolith.semanticSnapshot({ viewportOnly: true });
  console.log(snapshot);

  // 2. Visually highlight element to verify grounding
  await page.aerolith.highlight('input[name=login]');

  // 3. Atomically fill multiple fields in one turn
  await page.aerolith.fillForm([
    { target: 'input[name=login]', value: 'my-username' },
    { target: 'input[name=password]', value: 'my-password' }
  ], { submit: true });

  // 4. Capture Set-Of-Marks numbered bounding boxes for vision models
  await page.aerolith.screenshotSom({ path: 'som.png' });

  await browser.close();
})();
```

## Extended API Reference

| Method | Description |
|---|---|
| `page.aerolith.semanticSnapshot({ viewportOnly, role })` | Returns compact accessibility text tree with numbered target IDs. |
| `page.aerolith.highlight(target)` | Injects an animated focus ring around element to confirm visual grounding. |
| `page.aerolith.screenshotElement(target, { path, padding })` | Captures exact bounding-box cropped PNG image buffer with optional padding. |
| `page.aerolith.getText(target, attribute)` | Extracts element text content, input value, or specific HTML attribute. |
| `page.aerolith.hover(target)` | Hovers mouse over element to trigger flyouts, tooltips, and `:hover` states. |
| `page.aerolith.pressKey(key, { target })` | Dispatches realistic keyboard events (`Escape`, `Enter`, `Tab`, etc.). |
| `page.aerolith.selectOption(target, value)` | Selects an option in a `<select>` dropdown by value, label, or index. |
| `page.aerolith.drag(source, { target, offsetX, offsetY, steps })` | Drags an element or adjusts a range slider with smooth interpolation. |
| `page.aerolith.getCookies()` | Retrieves all cookies (including `HttpOnly` and secure flags) via CDP. |
| `page.aerolith.setCookie(cookie)` | Sets a cookie object via CDP. |
| `page.aerolith.deleteCookie(name, domain)` | Deletes a cookie by name and domain via CDP. |
| `page.aerolith.clearCookies()` | Clears all browser cookies via CDP. |
| `page.aerolith.setExtraHeaders(headers)` | Sets persistent custom HTTP request headers via CDP. |
| `page.aerolith.setUserAgent(userAgent, { acceptLanguage, platform })` | Overrides browser User-Agent and language via CDP. |
| `page.aerolith.uploadFile(target, files)` | Sets files on `<input type="file">` elements via synthetic DataTransfer. |
| `page.aerolith.setLocation(lat, lng, { accuracy, timezone })` | Overrides geographical coordinates and timezone via CDP and geolocation APIs. |
| `page.aerolith.mediaControl(action)` | Controls or queries audio and video playback on the page.aerolith. |
| `page.aerolith.pauseMedia()` | Pauses all video and audio playback across the page.aerolith. |
| `page.aerolith.inspect(target)` | Returns complete attributes, computed CSS styles, viewport visibility, and bounding box. |
| `page.aerolith.routeAbort(pattern)` | Dynamically intercepts and aborts network requests matching a URL pattern. |
| `page.aerolith.routeMock(pattern, options)` | Dynamically intercepts and mocks responses matching a URL pattern. |
| `page.aerolith.routeClear()` | Clears all dynamic route interception rules. |
| `page.aerolith.waitForNetworkIdle(timeoutMs)` | Waits until all network requests have settled for at least 250ms. |
| `page.aerolith.waitForUrl(pattern, timeoutMs)` | Waits until page URL matches or contains the target substring. |
| `page.aerolith.waitForFunction(jsCode, timeoutMs)` | Waits until a JavaScript condition evaluates to truthy. |
| `page.aerolith.getTabs()` | Lists all open browser tabs and identifies the active tab index. |
| `page.aerolith.newTab(url)` | Opens a new browser tab. |
| `page.aerolith.switchTab(index)` | Switches active focus to a tab by its index. |
| `page.aerolith.closeTab(index)` | Closes a tab by its index. |
| `page.aerolith.getNetworkActivity({ clear })` | Retrieves recorded AJAX/fetch requests and in-flight count. |
| `page.aerolith.fillForm(fields, { submit })` | Atomically populates multiple inputs and optionally submits. |
| `page.aerolith.setViewport(width, height, { dpr, mobile })` | Emulates screen dimensions, device pixel ratio, and touch. |
| `page.aerolith.getConsoleLogs({ clear })` | Retrieves buffered browser console logs (log, warn, error). |
| `page.aerolith.printPdf({ path, landscape })` | Generates high-fidelity vector PDF via CDP Page.printToPDF. |
| `page.aerolith.getMarkdown()` | Extracts clean, reader-friendly article markdown. |
| `page.aerolith.screenshotSom({ path })` | Renders high-contrast Set-Of-Marks tags before saving screenshot. |
| `page.aerolith.exportStorageState({ path })` | Exports cookies, `localStorage`, and `sessionStorage`. |
| `page.aerolith.importStorageState(state)` | Restores cookies, `localStorage`, and `sessionStorage`. |
| `page.aerolith.doubleClick(target)` | Simulates a human-like double click with sticky-header occlusion compensation. |
| `page.aerolith.rightClick(target)` | Simulates a right-click / contextmenu event on target element. |
| `page.aerolith.extractTable(target)` | Extracts structured rows and headers from an HTML table into clean JSON records. |
| `page.aerolith.getDownloads({ clear })` | Retrieves recorded browser file downloads or clears history. |
| `page.aerolith.emulateNetwork(options)` | Throttles network conditions (online, offline, slow-3g, fast-3g) via CDP. |
| `page.aerolith.injectCss(css)` | Injects custom CSS rules to eliminate floating headers, banners, or occluding elements. |
| `page.aerolith.clearCss()` | Removes previously injected custom CSS stylesheet. |
| `page.aerolith.queryElements(selector, options)` | Batch queries all matching elements and returns indexed metadata records. |
| `page.aerolith.focus(target)` | Explicitly focuses an element to trigger framework validation. |
| `page.aerolith.blur(target)` | Explicitly blurs an element to trigger framework change events. |
| `page.aerolith.getPageInfo()` | Retrieves page URL, title, viewport dimensions, document height, and scroll metrics. |
| `page.aerolith.getHtml(target)` | Extracts raw HTML source for the entire document or a specific DOM element. |
| `page.aerolith.scrollIntoView(target, align)` | Scrolls element into view with block alignment and sticky-header compensation. |
| `page.aerolith.setZoom(zoomFactor)` | Sets browser rendering zoom factor with dual-layer CSS and CDP scaling. |
| `page.aerolith.storageItem(action, options)` | Inspects or mutates granular localStorage or sessionStorage keys. |
| `page.aerolith.assertElement(target, assertions)` | Verifies element state against declarative assertions (visible, hidden, disabled, checked, text, value, classes, attributes). |
| `page.aerolith.waitForSelectorState(target, options)` | Waits for element to reach state: 'visible', 'hidden', 'attached', 'detached'. |
| `page.aerolith.getPerformanceMetrics()` | Retrieves page performance metrics (TTFB, DOMContentLoaded, load time, resource breakdown, transfer bytes, JS heap). |
| `page.aerolith.scrollToCoords(x, y, options)` | Scrolls viewport directly to specific absolute (x, y) coordinates. |
| `page.aerolith.getPageOutline(options)` | Retrieves compact semantic outline of the page: headings, forms, landmarks, buttons, and links. |
| `page.aerolith.waitForNetworkIdle(options)` | Waits until active network requests remain at or below max_inflight for consecutive idle_time_ms. |
| `page.aerolith.watchMutations(action, options)` | Controls and queries DOM mutation records (start, stop, clear, get). |
| `page.aerolith.tabEval(tabIndex, script)` | Evaluates a JavaScript expression directly in a specific tab by index without switching active GUI focus. |
| `page.aerolith.getResponseBody(pattern, options)` | Searches recorded network activity for URL pattern and returns captured response body. |
| `page.aerolith.dropFile(target, options)` | Simulates synthetic file drag-and-drop or file input upload with custom metadata. |
| `page.aerolith.getDomTree(options)` | Serializes nested hierarchical JSON tree of visible DOM nodes. |
| `page.aerolith.setOffline(offline)` | Instantly simulates network offline or online state via CDP Network emulation. |
| `page.aerolith.waitForText(text, options)` | Waits until a specific text substring appears in an element or document body. |
| `page.aerolith.getComputedStyles(target, properties)` | Reads computed CSS style properties (display, visibility, opacity, color, z-index, etc.). |
| `page.aerolith.waitForDownload(options)` | Waits for a new file download initiated by the page to appear. |
| `page.aerolith.setContent(html)` | Sets the HTML content of the active page directly via document.write. |
| `page.aerolith.getForms(target)` | Inspects and structures all HTML forms and their controls (input, select, textarea, button). |
| `page.aerolith.setVisibility(state)` | Force-emulates document.visibilityState ('visible' or 'hidden') and window focus. |
| `page.aerolith.emulateMedia(options)` | Emulates media color scheme ('dark', 'light') and/or media type ('screen', 'print'). |
| `page.aerolith.getElementBounds(target)` | Extracts exact pixel bounding box coordinates (x, y, width, height, center_x, center_y) and visibility metrics. |
| `page.aerolith.mockRoute(pattern, options)` | Mocks network responses for fetch/XHR requests matching a URL pattern. |
| `page.aerolith.clearMockRoutes()` | Clears all active network mock routes. |
| `page.aerolith.getViewport()` | Retrieves active viewport dimensions, scroll offsets, screen metrics, and device pixel ratio. |
| `page.aerolith.setElementAttribute(target, attribute, value)` | Dynamically sets or removes an HTML attribute on a DOM element for self-healing. |
| `page.aerolith.searchPageText(query, options)` | Searches visible page text for substrings and returns matches, element refs, and bounding boxes. |
| `page.aerolith.dumpStorage(storageType)` | Dumps all localStorage and/or sessionStorage key-value pairs in a single structured JSON payload. |
| `page.aerolith.humanJitter(options)` | Simulates subtle mouse micro-movements and scroll adjustments to pass anti-bot behavioral heuristics. |
| `page.aerolith.typeKeystrokes(target, text, options)` | Types text character-by-character with realistic key delays and complete KeyboardEvents. |
| `page.aerolith.getCanvasData(target, options)` | Extracts base64 image data URL from <canvas>, <img>, or arbitrary element via rasterization. |
| `page.aerolith.getResourceTiming(options)` | Extracts structured resource timing and transfer bandwidth metrics from window.performance. |
| `page.aerolith.getDomStats()` | Inspects structural DOM complexity metrics (element count, tree depth, forms, buttons, links, inputs). |
| `page.aerolith.scrollBy(dx, dy, options)` | Performs relative window scroll by horizontal (dx) and vertical (dy) pixel deltas. |
| `page.aerolith.scrollContainer(target, dx, dy, options)` | Scrolls a specific inner scrollable container (div, modal, textarea, section). |
| `page.aerolith.selectElementText(target, options)` | Selects text range inside an input, textarea, or contentEditable element. |
| `page.aerolith.extractPageLinks(options)` | Extracts structured, deduplicated hyperlinks graph from active document. |
| `page.aerolith.getElementState(target)` | Atomically inspects complete interactive, disabled, focused, checked, and visibility state. |
| `page.aerolith.toggleElementClass(target, className, action)` | Adds, removes, toggles, or checks a CSS class name on the target DOM element. |
| `page.aerolith.setElementText(target, text)` | Directly overwrites the textContent of target element and dispatches input/change events. |
| `page.aerolith.dispatchCustomEvent(target, eventType, options)` | Constructs and dispatches a synthetic CustomEvent on target element. |
| `page.aerolith.getActiveElement()` | Inspects the currently focused DOM element (document.activeElement). |
| `page.aerolith.checkElementClickable(target)` | Verifies whether an element is physically clickable at its center point or occluded by overlays/modals. |
| `page.aerolith.dispatchWheelEvent(target, options)` | Constructs and dispatches a synthetic WheelEvent on target element (for map zooms, pan widgets, and charts). |
| `page.aerolith.getCssVariables(options)` | Resolves computed values of CSS custom properties (--var) on an element or root document. |
| `page.aerolith.clearHighlights()` | Removes all agent visual highlight overlays and badge borders from the page DOM. |
| `page.aerolith.simulatePointerClick(target, options)` | Dispatches full sequence of PointerEvents (pointerdown, mousedown, pointerup, mouseup, click) at target center. |
| `page.aerolith.extractMetaTags(options)` | Extracts structured page metadata, OpenGraph tags, Twitter cards, links, and JSON-LD schemas. |
| `page.aerolith.getDocumentGeometry()` | Extracts complete document dimensions, viewport metrics, scroll offsets, and scrollable container count. |
| `page.aerolith.setDeviceOrientation(alpha, beta, gamma, options)` | Emulates mobile device orientation tilt angles (alpha, beta, gamma) and dispatches deviceorientation event. |
| `page.aerolith.clearBrowserCache(options)` | Purges browser in-memory and disk caches via CDP, with optional storage/cookie reset. |
| `page.aerolith.getElementSpacing(target)` | Inspects box-model computed spacing (margin, padding, border, dimensions, z-index, box-sizing) for an element. |
| `page.aerolith.waitForUrl(pattern, options)` | Waits until window.location.href matches a target substring or regex pattern. |
| `page.aerolith.dispatchTouchEvent(eventType, touches)` | Dispatches synthetic touch events (touchstart, touchmove, touchend, touchcancel) with multiple touch points. |
| `page.aerolith.getA11yTree(options)` | Extracts hierarchical computed WAI-ARIA accessibility (a11y) tree for navigation and screen reader analysis. |
| `page.aerolith.simulateKeyboardChord(chord, options)` | Dispatches keyboard hotkey chord (e.g. 'Control+Shift+P', 'Meta+K', 'Alt+Tab') with keydown/keyup sequence. |
| `page.aerolith.highlightBoundingBoxes(boxes)` | Renders visual bounding boxes with label badges over page regions for grounding and multimodal verification. |
| `page.aerolith.getElementAttributes(target)` | Extracts all HTML attributes of a DOM element as a complete key-value dictionary. |
| `page.aerolith.getElementFilter(target)` | Inspects CSS filters, backdrop filters, opacity, pointer-events, and ancestor occlusion. |
| `page.aerolith.dismissAllModals()` | Dismisses modal overlays, cookie consent popups, backdrop dialogs, and restores body scroll. |
| `page.aerolith.getColorPalette(limit)` | Extracts dominant computed colors and background colors used across visible elements on the page.aerolith. |
| `page.aerolith.dragAndDropElement(source, target, options)` | Drags a source element and drops it onto a target element with multi-step pointer and drag event simulation. |
| `page.aerolith.auditForms(target)` | Audits form elements, input validation constraints (required, pattern, min/max), and HTML5 validity states. |
| `page.aerolith.getViewportPaging()` | Calculates viewport dimensions, multi-page scroll coordinates, total pages, and current scroll progress. |
| `page.aerolith.getComputedPseudoElement(target, options)` | Inspects computed CSS styles of pseudo-elements like ::before or ::after (icons, badges, content). |
| `page.aerolith.simulateClipboardPaste(text, options)` | Simulates a clipboard paste event into a target or active element with DataTransfer text payload. |
| `page.aerolith.smartFill(mapping, options)` | Smartly auto-fills form controls by fuzzy matching human-readable labels, names, or placeholders. |
| `page.aerolith.observeText(target, options)` | Observes streaming text changes on an element until updates settle or timeout expires. |
| `page.aerolith.setVisibilityState(state)` | Overrides document.visibilityState to prevent bot pausing or background idle detection. |
| `page.aerolith.getComputedFont(target)` | Extracts computed font typography, font family, weight, line-height, and canvas measurement metrics. |
| `page.aerolith.getActionMap(options)` | Extracts dense interactive element click-map with centroids and numeric ref bindings. |
| `page.aerolith.getMotionState(target)` | Inspects active CSS animations and transitions on the page or target element. |
| `page.aerolith.getElementListeners(target)` | Inspects event listeners, inline handlers, cursor styles, and interactive affordances of an element. |
| `page.aerolith.getWebVitals()` | Extracts Core Web Vitals, performance navigation timings (TTFB, FCP, DOMContentLoaded), and memory metrics. |
| `page.aerolith.getSelectOptions(target)` | Inspects selectable options, option groups, values, text labels, and selected indices of a <select> element. |
| `page.aerolith.fuzzyFindElements(query, options)` | Fuzzy searches interactive elements by text content, ARIA labels, or attributes with similarity scoring. |
| `page.aerolith.getHeadingHierarchy(options)` | Extracts hierarchical heading structure (h1-h6) with levels, text content, IDs, and anchor links. |
| `page.aerolith.getElementStacking(target)` | Inspects CSS stacking context, z-index, position, opacity, transform, and ancestor clipping bounds. |
| `page.aerolith.queryShadowElements(selector, options)` | Recursively queries elements across both light DOM and all open shadow roots. |
| `page.aerolith.getElementVisibilityMatrix(target)` | Performs 5-point occlusion testing and computes visibility ratio against viewport and ancestor clipping. |
| `page.aerolith.getErrorDigest(options)` | Retrieves aggregated diagnostics of runtime JavaScript errors, unhandled rejections, and console errors. |
| `page.aerolith.extractGridData(options)` | Extracts structured datasets from HTML tables and ARIA grids/treegrids into column-mapped row records. |
| `page.aerolith.diffFormInputs(target)` | Inspects all form inputs and compares pristine default values with current values to identify dirty fields. |
| `page.aerolith.unstickScrollLocks()` | Neutralizes overflow:hidden locks, fixed position body wrappers, and invisible click backdrops. |
| `page.aerolith.diffDomText(options)` | Computes text mutations and diff between current visible content and a baseline text snapshot. |
| `page.aerolith.getFrameHierarchy()` | Inspects embedded frames and iframes with cross-origin detection and viewport coordinates. |
| `page.aerolith.getRoleElements(options)` | Filters and extracts interactive elements matching specific ARIA roles or semantics. |
| `page.aerolith.harvestResources(options)` | Harvests all page resources (images, scripts, styles, media, iframes) with resolved absolute URLs. |
| `page.aerolith.detectBotChallenges()` | Scans page for active bot challenges, CAPTCHAs (Turnstile, reCAPTCHA, hCaptcha), and WAF blocks. |
| `page.aerolith.getScrollState(target)` | Inspects scrollability metrics, directional scroll boundaries, and progress for an element or document. |
| `page.aerolith.getProximityElements(options)` | Finds and ranks nearby interactive elements sorted by Euclidean distance and cardinal direction. |
| `page.aerolith.emulateScreenMetrics(options)` | Emulates responsive screen metrics, viewport breakpoints, device pixel ratio, orientation, and dark mode. |
| `page.aerolith.exportTableCsv(options)` | Extracts tabular data directly formatted as clean RFC 4180 CSV with colspan/rowspan handling. |
| `page.aerolith.resetForm(options)` | Resets forms and input controls back to pristine default values without page reloads. |
| `page.aerolith.getDatalistOptions(target)` | Inspects options in an HTML5 datalist or linked autocomplete input combobox. |
| `page.aerolith.getCanvasContext(target)` | Inspects canvas rendering context (2D, WebGL, WebGL2), dimensions, and renderer hardware metrics. |
| `page.aerolith.controlMediaPlayback(options)` | Inspects and controls HTML5 audio/video playback elements with playback state telemetry. |
| `page.aerolith.getElementAncestry(target, options)` | Inspects the full DOM ancestry tree and selector hierarchy path from an element up to document root. |
| `page.aerolith.getLayoutMetrics(target)` | Inspects CSS Grid and Flexbox layout metrics, alignments, tracks, and child coordinates. |
| `page.aerolith.auditContrastRatio(target)` | Computes WCAG 2.1 relative luminance, contrast ratio, and accessibility compliance. |
| `page.aerolith.dispatchAdvancedEvent(target, options)` | Dispatches PointerEvent, MouseEvent, or KeyboardEvent with synthetic modifier keys. |
| `page.aerolith.getNetworkHar(options)` | Harvests intercepted network activity logs formatted as a standard HAR 1.2 archive. |
| `page.aerolith.getCaretPosition(target, options)` | Inspects or sets the caret position, selection offsets, and selected text in an element. |
| `page.aerolith.dismissCookieBanners(options)` | Auto-detects and dismisses GDPR/CCPA cookie consent overlays. |
| `page.aerolith.emulateSpeechApi(options)` | Emulates Web Speech API to feed synthetic voice transcripts or control recognition state. |
| `page.aerolith.getDomMutations(options)` | Harvests real-time DOM mutation logs captured by the background MutationObserver. |
| `page.aerolith.getActiveRoutes()` | Inspects active route interception, mock, and abort rules registered in the active tab. |
| `page.aerolith.execEditorCommand(command, options)` | Executes native rich text editor commands via document.execCommand. |
| `page.aerolith.preloadLazyResources(options)` | Forces hydration and eager preloading of lazy-loaded images, sources, and background media. |
| `page.aerolith.emulateWindowFocus(options)` | Emulates window and document focus state with optional focus/blur event dispatch. |
| `page.aerolith.sendKeySequence(keys, options)` | Dispatches an atomic sequence of keyboard shortcuts, key chords, or keystrokes. |
| `page.aerolith.diffElementStyles(target, options)` | Inspects and diffs computed CSS styles against an optional baseline to verify UI changes. |
| `page.aerolith.dispatchTouchGesture(options)` | Simulates realistic multi-touch gestures like pinch-to-zoom and two-finger pans. |
| `page.aerolith.inspectStorageQuota(options)` | Inspects storage quota estimates, Web Storage usage, Cache API, and IndexedDB inventories. |
| `page.aerolith.inspectElementReadiness(target)` | Inspects actionable readiness of an element (in_dom, displayed, visible, occluded, animating). |
| `page.aerolith.waitForFontsAndLayoutSettled(options)` | Waits for web font assets to finish loading and layout dimensions to stabilize. |
| `page.aerolith.emulatePageScaleFactor(scaleFactor)` | Emulates page zoom scaling factor and overrides window.devicePixelRatio. |
| `page.aerolith.copyFormattedContent(options)` | Extracts structured formatted content (markdown, clean html, or citation text). |

All agent extensions are available under `page.aerolith`. Existing native Playwright methods, including `page.hover` and `page.selectOption`, are preserved. Non-conflicting extension names remain available directly on `page`.

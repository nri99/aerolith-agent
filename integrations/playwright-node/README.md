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
| `page.screenshotElement(target, { path })` | Captures exact bounding-box cropped PNG image buffer. |
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

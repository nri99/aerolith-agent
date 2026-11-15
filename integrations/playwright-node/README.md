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
| `page.fillForm(fields, { submit })` | Atomically populates multiple inputs and optionally submits. |
| `page.getMarkdown()` | Extracts clean, reader-friendly article markdown. |
| `page.screenshotSom({ path })` | Renders high-contrast Set-Of-Marks tags before saving screenshot. |
| `page.exportStorageState({ path })` | Exports cookies, `localStorage`, and `sessionStorage`. |
| `page.importStorageState(state)` | Restores cookies, `localStorage`, and `sessionStorage`. |

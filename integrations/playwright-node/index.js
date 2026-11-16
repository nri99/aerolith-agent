/**
 * Aerolith Agent - Node.js Playwright SDK Integration.
 *
 * Connects Playwright to Aerolith Agent over Chrome DevTools Protocol (CDP)
 * and extends the Page object with Aerolith's autonomous agent capabilities:
 *   - page.semanticSnapshot({ viewportOnly, role })
 *   - page.highlight(target)
 *   - page.screenshotElement(target, { path })
 *   - page.fillForm(fields, { submit })
 *   - page.getMarkdown()
 *   - page.screenshotSom({ path })
 *   - page.exportStorageState({ path })
 *   - page.importStorageState(state)
 */

const http = require('http');
const fs = require('fs');

function httpGet(port, path) {
  return new Promise((resolve, reject) => {
    http.get(`http://127.0.0.1:${port}${path}`, (res) => {
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(e);
        }
      });
    }).on('error', reject);
  });
}

function httpPost(port, path, payload) {
  return new Promise((resolve, reject) => {
    const postData = JSON.stringify(payload);
    const req = http.request({
      hostname: '127.0.0.1',
      port,
      path,
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(postData),
        'User-Agent': 'AerolithPlaywrightNode/0.2.0'
      }
    }, (res) => {
      let data = '';
      res.on('data', (chunk) => { data += chunk; });
      res.on('end', () => {
        try {
          resolve(JSON.parse(data));
        } catch (e) {
          reject(e);
        }
      });
    });
    req.on('error', reject);
    req.write(postData);
    req.end();
  });
}

function extendPage(page, agentPort = 8765) {
  // 1. Semantic Accessibility Tree Snapshot
  page.semanticSnapshot = async function (options = {}) {
    const params = [];
    if (options.viewportOnly) params.push('viewport_only=1');
    if (options.role) params.push(`role=${encodeURIComponent(options.role)}`);
    const query = params.length ? `?${params.join('&')}` : '';
    const res = await httpGet(agentPort, `/snapshot${query}`);
    return res?.data?.tree_text || '';
  };

  // 2. Visual Grounding Highlight Ring
  page.highlight = async function (target) {
    const res = await httpPost(agentPort, '/highlight', { target: String(target) });
    return res?.data || {};
  };

  // 3. Exact Element Bounding-Box Cropped Screenshot
  page.screenshotElement = async function (target, options = {}) {
    const res = await httpPost(agentPort, '/screenshot/element', { target: String(target) });
    const b64 = res?.data?.screenshot_base64 || '';
    const buffer = Buffer.from(b64, 'base64');
    if (options.path && buffer.length > 0) {
      fs.writeFileSync(options.path, buffer);
    }
    return buffer;
  };

  // 4. Atomic Multi-Field Form Fill
  page.fillForm = async function (fields, options = {}) {
    const submit = Boolean(options.submit);
    const res = await httpPost(agentPort, '/fill', { fields, submit });
    return res?.data || {};
  };

  // 4a. Element Hover
  page.hover = async function (target) {
    const res = await httpPost(agentPort, '/hover', { target: String(target) });
    return res?.data || {};
  };

  // 4b. Press Keyboard Key
  page.pressKey = async function (key, options = {}) {
    const payload = { key: String(key) };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/key', payload);
    return res?.data || {};
  };

  // 4c. Select Dropdown Option
  page.selectOption = async function (target, value) {
    const res = await httpPost(agentPort, '/select', { target: String(target), value: String(value) });
    return res?.data || {};
  };

  // 4d. Drag and Drop / Slider Gesture
  page.drag = async function (source, options = {}) {
    const payload = {
      source: String(source),
      steps: options.steps || 10
    };
    if (options.target) payload.target = String(options.target);
    if (options.offsetX !== undefined) payload.offset_x = options.offsetX;
    if (options.offsetY !== undefined) payload.offset_y = options.offsetY;
    const res = await httpPost(agentPort, '/drag', payload);
    return res?.data || {};
  };

  // 4e. Cookie Management (CDP)
  page.getCookies = async function () {
    const res = await httpGet(agentPort, '/cookies');
    return res?.data?.cookies || [];
  };

  page.setCookie = async function (cookie) {
    const res = await httpPost(agentPort, '/cookies', cookie);
    return res?.data || {};
  };

  page.deleteCookie = async function (name, domain) {
    const res = await httpPost(agentPort, '/cookies/delete', { name, domain });
    return res?.data || {};
  };

  page.clearCookies = async function () {
    const res = await httpPost(agentPort, '/cookies/clear', {});
    return res?.data || {};
  };

  // 4f. Custom Headers & User-Agent (CDP)
  page.setExtraHeaders = async function (headers) {
    const res = await httpPost(agentPort, '/headers', { headers });
    return res?.data || {};
  };

  page.setUserAgent = async function (userAgent, options = {}) {
    const payload = { user_agent: userAgent };
    if (options.acceptLanguage) payload.accept_language = options.acceptLanguage;
    if (options.platform) payload.platform = options.platform;
    const res = await httpPost(agentPort, '/user_agent', payload);
    return res?.data || {};
  };

  // 4g. File Upload
  page.uploadFile = async function (target, files) {
    const fileList = Array.isArray(files) ? files : [files];
    const res = await httpPost(agentPort, '/upload', { target: String(target), files: fileList });
    return res?.data || {};
  };

  // 4h. Geolocation & Timezone Emulation
  page.setLocation = async function (latitude, longitude, options = {}) {
    const payload = { latitude, longitude };
    if (options.accuracy !== undefined) payload.accuracy = options.accuracy;
    if (options.timezone) payload.timezone = options.timezone;
    const res = await httpPost(agentPort, '/location', payload);
    return res?.data || {};
  };

  // 4i. Media Playback Control
  page.mediaControl = async function (action = 'status') {
    const res = action === 'status'
      ? await httpGet(agentPort, '/media')
      : await httpPost(agentPort, '/media', { action });
    return res?.data || {};
  };

  page.pauseMedia = async function () {
    return page.mediaControl('pause');
  };

  page.muteMedia = async function () {
    return page.mediaControl('mute');
  };

  // 4j. Network Activity Inspection
  page.getNetworkActivity = async function (options = {}) {
    const clear = Boolean(options.clear);
    const res = clear
      ? await httpPost(agentPort, '/network/clear', {})
      : await httpGet(agentPort, '/network');
    return res?.data || {};
  };

  // 4e. Element Text & Attribute Extraction
  page.getText = async function (target, attribute) {
    const params = [`target=${encodeURIComponent(String(target))}`];
    if (attribute) params.push(`attr=${encodeURIComponent(attribute)}`);
    const res = await httpGet(agentPort, `/text?${params.join('&')}`);
    return res?.data?.text || '';
  };

  // 4c. Viewport Emulation
  page.setViewport = async function (width, height, options = {}) {
    const payload = {
      width,
      height,
      mobile: Boolean(options.mobile)
    };
    if (options.dpr !== undefined) payload.dpr = Number(options.dpr);
    const res = await httpPost(agentPort, '/emulate', payload);
    return res?.data || {};
  };

  // 4d. Console Logs Retrieval
  page.getConsoleLogs = async function (options = {}) {
    const clear = Boolean(options.clear);
    const res = await httpGet(agentPort, clear ? '/console/clear' : '/console');
    return res?.data?.logs || [];
  };

  // 4e. High-Fidelity PDF Generation
  page.printPdf = async function (options = {}) {
    const landscape = Boolean(options.landscape);
    const path = options.path;
    const res = await httpGet(agentPort, landscape ? '/pdf?landscape=1' : '/pdf');
    const b64 = res?.data?.pdf_base64 || '';
    const buffer = Buffer.from(b64, 'base64');
    if (path && buffer.length > 0) {
      fs.writeFileSync(path, buffer);
    }
    return buffer;
  };

  // 4f. Deep Element Inspection
  page.inspect = async function (target) {
    const res = await httpPost(agentPort, '/inspect', { target: String(target) });
    return res?.data || {};
  };

  // 4g. Dynamic Route Interception: Abort Requests Matching Pattern
  page.routeAbort = async function (pattern) {
    const res = await httpPost(agentPort, '/route', { action: 'abort', pattern: String(pattern), rule_type: 'abort' });
    return res?.data || {};
  };

  // 4h. Dynamic Route Interception: Mock Responses Matching Pattern
  page.routeMock = async function (pattern, options = {}) {
    const payload = {
      action: 'mock',
      pattern: String(pattern),
      rule_type: 'mock',
      status: options.status || 200,
      body: typeof options.body === 'object' ? JSON.stringify(options.body) : String(options.body || ''),
      headers: options.headers || { 'Content-Type': 'application/json' }
    };
    const res = await httpPost(agentPort, '/route', payload);
    return res?.data || {};
  };

  // 4i. Route Interception: Clear Rules
  page.routeClear = async function () {
    const res = await httpPost(agentPort, '/route', { action: 'clear' });
    return res?.data || {};
  };

  // 4j. Route Interception: Get Rules
  page.getRoutes = async function () {
    const res = await httpGet(agentPort, '/route');
    return res?.data?.routes || [];
  };

  // 4k. Smart Enhanced Wait: Network Idle
  page.waitForNetworkIdle = async function (timeoutMs = 10000) {
    const res = await httpPost(agentPort, '/wait_for', { target: 'networkidle', timeout_ms: timeoutMs });
    return res?.data || {};
  };

  // 4l. Smart Enhanced Wait: URL Match
  page.waitForUrl = async function (pattern, timeoutMs = 10000) {
    const res = await httpPost(agentPort, '/wait_for', { target: `url:${pattern}`, timeout_ms: timeoutMs });
    return res?.data || {};
  };

  // 4m. Smart Enhanced Wait: JS Function Condition
  page.waitForFunction = async function (jsCode, timeoutMs = 10000) {
    const res = await httpPost(agentPort, '/wait_for', { target: `js:${jsCode}`, timeout_ms: timeoutMs });
    return res?.data || {};
  };

  // 4n. Browser Tab Management: List Tabs
  page.getTabs = async function () {
    const res = await httpGet(agentPort, '/tabs');
    return res?.data?.tabs || [];
  };

  // 4o. Browser Tab Management: New Tab
  page.newTab = async function (url = 'about:home') {
    const res = await httpPost(agentPort, '/tabs/new', { url });
    return res?.data || {};
  };

  // 4p. Browser Tab Management: Switch Tab
  page.switchTab = async function (index) {
    const res = await httpPost(agentPort, '/tabs/switch', { index: Number(index) });
    return res?.data || {};
  };

  // 4q. Browser Tab Management: Close Tab
  page.closeTab = async function (index) {
    const res = await httpPost(agentPort, '/tabs/close', { index: Number(index) });
    return res?.data || {};
  };

  // 5. Clean Article Markdown Extraction
  page.getMarkdown = async function () {
    const res = await httpGet(agentPort, '/markdown');
    return res?.data?.markdown || '';
  };

  // 6. Set-Of-Marks Numbered Badge Screenshot
  page.screenshotSom = async function (options = {}) {
    const res = await httpGet(agentPort, '/screenshot?som=1');
    const b64 = res?.data?.screenshot_base64 || '';
    const buffer = Buffer.from(b64, 'base64');
    if (options.path && buffer.length > 0) {
      fs.writeFileSync(options.path, buffer);
    }
    return buffer;
  };

  // 7. Storage State Export
  page.exportStorageState = async function (options = {}) {
    const res = await httpGet(agentPort, '/storage/state');
    const data = res?.data || {};
    if (options.path) {
      fs.writeFileSync(options.path, JSON.stringify(data, null, 2), 'utf-8');
    }
    return data;
  };

  // 8. Storage State Restore
  page.importStorageState = async function (state) {
    let payload = state;
    if (typeof state === 'string') {
      payload = JSON.parse(fs.readFileSync(state, 'utf-8'));
    }
    const res = await httpPost(agentPort, '/storage/state', payload);
    return res?.data || {};
  };

  // 9. Double Click
  page.doubleClick = async function (target) {
    const res = await httpPost(agentPort, '/dblclick', { target: String(target) });
    return res?.data || {};
  };
  page.dblclick = page.doubleClick;

  // 10. Right Click / Context Menu
  page.rightClick = async function (target) {
    const res = await httpPost(agentPort, '/right_click', { target: String(target) });
    return res?.data || {};
  };
  page.contextmenu = page.rightClick;

  // 11. Extract Structured Table
  page.extractTable = async function (target) {
    const path = target ? `/table?target=${encodeURIComponent(target)}` : '/table';
    const res = await httpGet(agentPort, path);
    return res?.data || {};
  };

  // 12. File Downloads Telemetry
  page.getDownloads = async function (options = {}) {
    const clear = Boolean(options.clear);
    const res = await (clear ? httpPost(agentPort, '/downloads/clear', {}) : httpGet(agentPort, '/downloads'));
    return res?.data?.downloads || [];
  };

  // 13. Network Throttling / Emulation
  page.emulateNetwork = async function (options = {}) {
    const payload = {
      profile: options.profile || 'online',
      offline: Boolean(options.offline)
    };
    if (options.latency !== undefined) payload.latency = options.latency;
    if (options.download !== undefined) payload.download = options.download;
    if (options.upload !== undefined) payload.upload = options.upload;
    const res = await httpPost(agentPort, '/network/emulate', payload);
    return res?.data || {};
  };

  // 14. Custom CSS Injection
  page.injectCss = async function (css) {
    const res = await httpPost(agentPort, '/css', { css: String(css), remove: false });
    return res?.data || {};
  };
  page.clearCss = async function () {
    const res = await httpPost(agentPort, '/css/clear', {});
    return res?.data || {};
  };

  // 15. Batch Element Query
  page.queryElements = async function (selector, options = {}) {
    const payload = {
      selector: String(selector),
      max_items: options.maxItems || 50
    };
    if (options.attribute) payload.attribute = options.attribute;
    const res = await httpPost(agentPort, '/query', payload);
    return res?.data?.items || [];
  };

  // 16. Focus & Blur
  page.focus = async function (target) {
    const res = await httpPost(agentPort, '/focus', { target: String(target) });
    return res?.data || {};
  };
  page.blur = async function (target) {
    const res = await httpPost(agentPort, '/blur', { target: String(target) });
    return res?.data || {};
  };

  // 17. Page Metrics & Layout Info
  page.getPageInfo = async function () {
    const res = await httpGet(agentPort, '/page_info');
    return res?.data || {};
  };

  return page;
}

/**
 * Connect Playwright Chromium to a running Aerolith Agent browser.
 *
 * @param {object} playwright - The Playwright module (e.g. require('playwright'))
 * @param {object} [options]
 * @param {string} [options.cdpUrl='http://127.0.0.1:9222'] - CDP debugging endpoint
 * @param {number} [options.agentPort=8765] - Aerolith Agent HTTP server port
 * @returns {Promise<{ browser: object, page: object }>}
 */
async function connectAerolith(playwright, options = {}) {
  const cdpUrl = options.cdpUrl || 'http://127.0.0.1:9222';
  const agentPort = options.agentPort || 8765;

  const browser = await playwright.chromium.connectOverCDP(cdpUrl);
  const context = browser.contexts()[0];
  const pages = context.pages();
  const rawPage = pages.length > 0 ? pages[0] : await context.newPage();
  const page = extendPage(rawPage, agentPort);

  return { browser, page };
}

module.exports = {
  connectAerolith,
  extendPage,
};

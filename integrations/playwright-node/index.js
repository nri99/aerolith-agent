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
    const payload = { target: String(target) };
    if (options.padding !== undefined) payload.padding = Number(options.padding) || 0;
    const res = await httpPost(agentPort, '/screenshot/element', payload);
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

  // 18. Raw HTML Extraction
  page.getHtml = async function (target) {
    const path = target ? `/html?target=${encodeURIComponent(target)}` : '/html';
    const res = await httpGet(agentPort, path);
    return res?.data?.html || '';
  };

  // 19. Sticky-Header Compensated Scroll Into View
  page.scrollIntoView = async function (target, align = 'center') {
    const res = await httpPost(agentPort, '/scroll_into_view', { target: String(target), align: String(align) });
    return res?.data || {};
  };

  // 20. Dual-Layer Rendering Zoom
  page.setZoom = async function (zoomFactor) {
    const res = await httpPost(agentPort, '/zoom', { zoom_factor: Number(zoomFactor) });
    return res?.data || {};
  };

  // 21. Granular Web Storage Key Operations
  page.storageItem = async function (action, options = {}) {
    const payload = {
      action: String(action),
      storage_type: options.storageType || options.type || 'local'
    };
    if (options.key !== undefined) payload.key = String(options.key);
    if (options.value !== undefined) payload.value = String(options.value);
    const res = await httpPost(agentPort, '/storage/item', payload);
    return res?.data || {};
  };

  // 22. Declarative Element State Assertions
  page.assertElement = async function (target, assertions = {}) {
    const payload = {
      target: String(target),
      assertions: assertions
    };
    const res = await httpPost(agentPort, '/assert_element', payload);
    return res?.data || {};
  };

  // 23. Wait for Selector State
  page.waitForSelectorState = async function (target, options = {}) {
    const payload = {
      target: String(target),
      state: options.state || 'visible',
      timeout_ms: options.timeout || options.timeoutMs || 5000
    };
    const res = await httpPost(agentPort, '/wait_for_selector', payload);
    return res?.data || {};
  };

  // 24. Navigation & Resource Performance Metrics
  page.getPerformanceMetrics = async function () {
    const res = await httpGet(agentPort, '/performance');
    return res?.data?.metrics || {};
  };

  // 25. Absolute Coordinate Viewport Scrolling
  page.scrollToCoords = async function (x, y, options = {}) {
    const payload = {
      x: Number(x) || 0,
      y: Number(y) || 0,
      smooth: Boolean(options.smooth)
    };
    const res = await httpPost(agentPort, '/scroll_to', payload);
    return res?.data || {};
  };

  // 26. Compact Semantic Page Outline
  page.getPageOutline = async function (options = {}) {
    const payload = {
      max_links: options.maxLinks || 25,
      max_buttons: options.maxButtons || 25
    };
    const res = await httpPost(agentPort, '/outline', payload);
    return res?.data || {};
  };

  // 27. Wait for Network Inflight Idle
  page.waitForNetworkIdle = async function (options = {}) {
    const payload = {
      idle_time_ms: options.idleTimeMs || 500,
      timeout_ms: options.timeout || options.timeoutMs || 10000,
      max_inflight: options.maxInflight || 0
    };
    const res = await httpPost(agentPort, '/wait_for_idle', payload);
    return res?.data || {};
  };

  // 28. DOM Mutation Observer Buffer
  page.watchMutations = async function (action = 'get', options = {}) {
    const payload = {
      action: String(action)
    };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/mutations', payload);
    return res?.data || {};
  };

  // 29. Multi-Tab Direct Script Evaluation
  page.tabEval = async function (tabIndex, script) {
    const payload = {
      tab_index: Number(tabIndex) || 0,
      script: String(script)
    };
    const res = await httpPost(agentPort, '/tabs/eval', payload);
    return res?.data?.result;
  };

  // 30. Capture Network Response Body
  page.getResponseBody = async function (pattern = '', options = {}) {
    const payload = {
      pattern: String(pattern),
      clear: Boolean(options.clear)
    };
    const res = await httpPost(agentPort, '/network/body', payload);
    return res?.data || {};
  };

  // 31. Synthetic Drag-and-Drop File Upload
  page.dropFile = async function (target, options = {}) {
    const payload = {
      target: String(target),
      file_name: options.fileName || options.name || 'upload.bin',
      mime_type: options.mimeType || options.type || 'application/octet-stream',
      content: String(options.content || options.data || '')
    };
    const res = await httpPost(agentPort, '/drop_file', payload);
    return res?.data || {};
  };

  // 32. Hierarchical Nested DOM Tree
  page.getDomTree = async function (options = {}) {
    const payload = {
      max_depth: options.maxDepth || options.depth || 4
    };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/dom/tree', payload);
    return res?.data?.tree || {};
  };

  // 33. Emulate Offline / Online State
  page.setOffline = async function (offline = true) {
    const res = await httpPost(agentPort, '/network/offline', { offline: Boolean(offline) });
    return res?.data || {};
  };

  // 34. Wait for Text Substring
  page.waitForText = async function (text, options = {}) {
    const payload = {
      text: String(text),
      timeout_ms: options.timeout || options.timeoutMs || 5000
    };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/wait_for_text', payload);
    return res?.data || {};
  };

  // 35. Computed CSS Styles
  page.getComputedStyles = async function (target, properties = []) {
    const payload = {
      target: String(target)
    };
    if (Array.isArray(properties) && properties.length > 0) {
      payload.properties = properties;
    }
    const res = await httpPost(agentPort, '/computed_styles', payload);
    return res?.data?.styles || {};
  };

  // 36. Wait for Download
  page.waitForDownload = async function (options = {}) {
    const payload = {
      timeout_ms: options.timeout || options.timeoutMs || 15000
    };
    if (options.filePattern || options.pattern) {
      payload.file_pattern = String(options.filePattern || options.pattern);
    }
    const res = await httpPost(agentPort, '/wait_for_download', payload);
    return res?.data || {};
  };

  // 37. Set HTML Content Directly
  page.setContent = async function (html) {
    const res = await httpPost(agentPort, '/set_content', { html: String(html) });
    return res?.data || {};
  };

  // 38. Interactive Form Auto-Discovery
  page.getForms = async function (target) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/forms', payload);
    return res?.data?.forms || [];
  };

  // 39. Page Visibility Emulation
  page.setVisibility = async function (state = 'visible') {
    const res = await httpPost(agentPort, '/visibility', { state: String(state) });
    return res?.data || {};
  };

  // 40. Color Scheme & Media Emulation
  page.emulateMedia = async function (options = {}) {
    const payload = {};
    if (options.mediaType || options.media) payload.media_type = String(options.mediaType || options.media);
    if (options.colorScheme || options.scheme) payload.color_scheme = String(options.colorScheme || options.scheme);
    const res = await httpPost(agentPort, '/emulate/media', payload);
    return res?.data || {};
  };

  // 41. Exact Element Bounds & Visibility
  page.getElementBounds = async function (target) {
    const res = await httpPost(agentPort, '/element/bounds', { target: String(target) });
    return res?.data?.bounds || {};
  };

  // 42. Dynamic Network Route Mocking
  page.mockRoute = async function (pattern, options = {}) {
    const payload = {
      action: options.action || 'set',
      pattern: String(pattern),
      status: Number(options.status) || 200,
      content_type: options.contentType || options.type || 'application/json'
    };
    if (options.body !== undefined) payload.body = typeof options.body === 'string' ? options.body : JSON.stringify(options.body);
    const res = await httpPost(agentPort, '/network/mock', payload);
    return res?.data || {};
  };

  // 43. Clear Network Mock Routes
  page.clearMockRoutes = async function () {
    const res = await httpPost(agentPort, '/network/mock', { action: 'clear' });
    return res?.data || {};
  };

  // 44. Viewport Layout & Screen Metrics
  page.getViewport = async function () {
    const res = await httpGet(agentPort, '/viewport');
    return res?.data?.viewport || {};
  };

  // 45. Set or Remove Element HTML Attribute
  page.setElementAttribute = async function (target, attribute, value = null) {
    const payload = {
      target: String(target),
      attribute: String(attribute)
    };
    if (value !== null && value !== undefined) payload.value = String(value);
    const res = await httpPost(agentPort, '/element/attribute', payload);
    return res?.data || {};
  };

  // 46. Full-Page Visible Text Search
  page.searchPageText = async function (query, options = {}) {
    const payload = {
      query: String(query),
      max_matches: options.maxMatches !== undefined ? Number(options.maxMatches) : 20,
      case_sensitive: !!options.caseSensitive
    };
    const res = await httpPost(agentPort, '/search/text', payload);
    return res?.data?.matches || [];
  };

  // 47. Structured Storage Dump
  page.dumpStorage = async function (storageType = 'all') {
    const res = await httpGet(agentPort, `/storage/dump?type=${encodeURIComponent(storageType)}`);
    return res?.data || {};
  };

  // 48. Human Activity Jitter
  page.humanJitter = async function (options = {}) {
    const payload = {
      duration_ms: options.durationMs !== undefined ? Number(options.durationMs) : 800,
      move_mouse: options.moveMouse !== undefined ? !!options.moveMouse : true,
      micro_scroll: options.microScroll !== undefined ? !!options.microScroll : true
    };
    const res = await httpPost(agentPort, '/human_jitter', payload);
    return res?.data || {};
  };

  // 49. Realistic Character Keystrokes
  page.typeKeystrokes = async function (target, text, options = {}) {
    const payload = {
      target: String(target),
      text: String(text),
      delay_ms: options.delayMs !== undefined ? Number(options.delayMs) : 25,
      clear: !!options.clear,
      submit: !!options.submit
    };
    const res = await httpPost(agentPort, '/type/keystrokes', payload);
    return res?.data || {};
  };

  // 50. Canvas or Image Data URL Extraction
  page.getCanvasData = async function (target, options = {}) {
    const payload = {
      target: String(target),
      mime_type: options.mimeType || options.format || 'image/png',
      quality: options.quality !== undefined ? Number(options.quality) : 0.92
    };
    const res = await httpPost(agentPort, '/canvas/data', payload);
    return res?.data || {};
  };

  // 51. Resource Timing & Bandwidth Metrics
  page.getResourceTiming = async function (options = {}) {
    const payload = {
      max_entries: options.maxEntries !== undefined ? Number(options.maxEntries) : 50,
      min_duration_ms: options.minDurationMs !== undefined ? Number(options.minDurationMs) : 0.0
    };
    if (options.initiatorType) payload.initiator_type = String(options.initiatorType);
    const res = await httpPost(agentPort, '/performance/resources', payload);
    return res?.data?.resources || [];
  };

  // 52. Structural DOM Stats
  page.getDomStats = async function () {
    const res = await httpGet(agentPort, '/dom/stats');
    return res?.data || {};
  };

  // 53. Relative 2D Window Scroll
  page.scrollBy = async function (dx = 0, dy = 0, options = {}) {
    const payload = {
      dx: Number(dx),
      dy: Number(dy),
      smooth: !!options.smooth
    };
    const res = await httpPost(agentPort, '/scroll/by', payload);
    return res?.data || {};
  };

  // 54. Scroll Inner Container
  page.scrollContainer = async function (target, dx = 0, dy = 0, options = {}) {
    const payload = {
      target: String(target),
      dx: Number(dx),
      dy: Number(dy),
      smooth: !!options.smooth
    };
    const res = await httpPost(agentPort, '/scroll/container', payload);
    return res?.data || {};
  };

  // 55. Select Element Text Range
  page.selectElementText = async function (target, options = {}) {
    const payload = {
      target: String(target)
    };
    if (options.start !== undefined) payload.start = Number(options.start);
    if (options.length !== undefined) payload.length = Number(options.length);
    const res = await httpPost(agentPort, '/element/select_text', payload);
    return res?.data || {};
  };

  // 56. Extract Page Links Graph
  page.extractPageLinks = async function (options = {}) {
    const payload = {
      max_links: options.maxLinks !== undefined ? Number(options.maxLinks) : 100,
      external_only: !!options.externalOnly
    };
    if (options.filterPath) payload.filter_path = String(options.filterPath);
    const res = await httpPost(agentPort, '/links', payload);
    return res?.data || {};
  };

  // 57. Inspect Complete Element State
  page.getElementState = async function (target) {
    const res = await httpGet(agentPort, `/element/state?target=${encodeURIComponent(target)}`);
    return res?.data || {};
  };

  // 58. Add, Remove, Toggle or Check Element CSS Class
  page.toggleElementClass = async function (target, className, action = 'toggle') {
    const payload = {
      target: String(target),
      class_name: String(className),
      action: String(action)
    };
    const res = await httpPost(agentPort, '/element/class', payload);
    return res?.data || {};
  };

  // 59. Direct Text Content Overwrite
  page.setElementText = async function (target, text) {
    const payload = {
      target: String(target),
      text: String(text)
    };
    const res = await httpPost(agentPort, '/element/text/set', payload);
    return res?.data || {};
  };

  // 60. Dispatch Synthetic Custom Event
  page.dispatchCustomEvent = async function (target, eventType, options = {}) {
    const payload = {
      target: String(target),
      event_type: String(eventType),
      bubbles: options.bubbles !== undefined ? !!options.bubbles : true,
      cancelable: options.cancelable !== undefined ? !!options.cancelable : true
    };
    if (options.detail !== undefined) payload.detail = options.detail;
    const res = await httpPost(agentPort, '/element/dispatch_event', payload);
    return res?.data || {};
  };

  // 61. Query Focused Element
  page.getActiveElement = async function () {
    const res = await httpGet(agentPort, '/active_element');
    return res?.data || {};
  };

  // 62. Check Element Physical Clickability & Occlusion
  page.checkElementClickable = async function (target) {
    const res = await httpPost(agentPort, '/element/clickable', { target: String(target) });
    return res?.data || {};
  };

  // 63. Dispatch Synthetic Wheel Event
  page.dispatchWheelEvent = async function (target, options = {}) {
    const payload = {
      target: String(target),
      delta_x: options.deltaX !== undefined ? Number(options.deltaX) : 0.0,
      delta_y: options.deltaY !== undefined ? Number(options.deltaY) : 0.0,
      ctrl_key: !!options.ctrlKey
    };
    const res = await httpPost(agentPort, '/element/wheel', payload);
    return res?.data || {};
  };

  // 64. Resolve Computed CSS Variables
  page.getCssVariables = async function (options = {}) {
    const payload = {};
    if (options.target !== undefined && options.target !== null) {
      payload.target = String(options.target);
    }
    if (Array.isArray(options.variableNames)) {
      payload.variable_names = options.variableNames.map(String);
    } else if (Array.isArray(options.variables)) {
      payload.variable_names = options.variables.map(String);
    }
    const res = await httpPost(agentPort, '/css/variables', payload);
    return res?.data || {};
  };

  // 65. Clear Visual Highlight Badges
  page.clearHighlights = async function () {
    const res = await httpPost(agentPort, '/highlights/clear', {});
    return res?.data || {};
  };

  // 66. Simulate Center Pointer Click Sequence
  page.simulatePointerClick = async function (target, options = {}) {
    const payload = {
      target: String(target),
      click_count: options.clickCount !== undefined ? Number(options.clickCount) : (options.count !== undefined ? Number(options.count) : 1),
      button: options.button !== undefined ? Number(options.button) : 0
    };
    const res = await httpPost(agentPort, '/element/pointer_click', payload);
    return res?.data || {};
  };

  // 67. Extract Meta Tags, OpenGraph, Twitter Cards, and JSON-LD
  page.extractMetaTags = async function (options = {}) {
    const includeJsonLd = options.includeJsonLd !== undefined ? !!options.includeJsonLd : true;
    const res = await httpPost(agentPort, '/meta/tags', { include_json_ld: includeJsonLd });
    return res?.data || {};
  };

  // 68. Get Full Document & Viewport Geometry Matrix
  page.getDocumentGeometry = async function () {
    const res = await httpGet(agentPort, '/document/geometry');
    return res?.data || {};
  };

  // 69. Emulate Mobile Device Orientation
  page.setDeviceOrientation = async function (alpha = 0, beta = 0, gamma = 0, options = {}) {
    const payload = {
      alpha: Number(alpha),
      beta: Number(beta),
      gamma: Number(gamma),
      absolute: !!options.absolute
    };
    const res = await httpPost(agentPort, '/emulate/orientation', payload);
    return res?.data || {};
  };

  // 70. Purge Browser In-Memory/Disk Caches & Storage
  page.clearBrowserCache = async function (options = {}) {
    const payload = {
      clear_storage: !!options.clearStorage,
      clear_cookies: !!options.clearCookies
    };
    const res = await httpPost(agentPort, '/cache/clear', payload);
    return res?.data || {};
  };

  // 71. Inspect Element Box Model Computed Spacing
  page.getElementSpacing = async function (target) {
    const res = await httpPost(agentPort, '/element/spacing', { target: String(target) });
    return res?.data || {};
  };

  // 72. Wait for URL Substring or Pattern
  page.waitForUrl = async function (pattern, options = {}) {
    const payload = {
      pattern: String(pattern),
      timeout_ms: options.timeoutMs || options.timeout || 5000
    };
    const res = await httpPost(agentPort, '/wait_for_url', payload);
    return res?.data || {};
  };

  // 73. Dispatch Synthetic Touch Event
  page.dispatchTouchEvent = async function (eventType = 'touchstart', touches = []) {
    const payload = {
      event_type: String(eventType),
      touches: Array.isArray(touches) ? touches : []
    };
    const res = await httpPost(agentPort, '/touch', payload);
    return res?.data || {};
  };

  // 74. Extract Computed A11y Tree
  page.getA11yTree = async function (options = {}) {
    const payload = {
      max_depth: options.maxDepth !== undefined ? Number(options.maxDepth) : (options.depth !== undefined ? Number(options.depth) : 5)
    };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/a11y', payload);
    return res?.data?.tree || {};
  };

  // 75. Simulate Keyboard Chord
  page.simulateKeyboardChord = async function (chord, options = {}) {
    const payload = { chord: String(chord) };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/keyboard/chord', payload);
    return res?.data || {};
  };

  // 76. Render Highlight Bounding Boxes
  page.highlightBoundingBoxes = async function (boxes = []) {
    const res = await httpPost(agentPort, '/highlights/boxes', { boxes: Array.isArray(boxes) ? boxes : [] });
    return res?.data || {};
  };

  // 77. Extract Element HTML Attributes
  page.getElementAttributes = async function (target) {
    const res = await httpPost(agentPort, '/element/attributes', { target: String(target) });
    return res?.data?.attributes || {};
  };

  // 78. Inspect CSS Filters & Ancestor Occlusion
  page.getElementFilter = async function (target) {
    const res = await httpPost(agentPort, '/element/filter', { target: String(target) });
    return res?.data || {};
  };

  // 79. Dismiss All Modals & Cookie Overlays
  page.dismissAllModals = async function () {
    const res = await httpPost(agentPort, '/modals/dismiss', {});
    return res?.data || {};
  };

  // 80. Extract Page Color Palette
  page.getColorPalette = async function (limit = 15) {
    const res = await httpPost(agentPort, '/palette', { limit: Number(limit) || 15 });
    return res?.data || {};
  };

  // 81. Drag and Drop Element
  page.dragAndDropElement = async function (source, target, options = {}) {
    const payload = {
      source: String(source),
      target: String(target),
      steps: Number(options.steps) || 10,
      hold_ms: Number(options.holdMs || options.hold_ms) || 0,
    };
    const res = await httpPost(agentPort, '/drag_and_drop', payload);
    return res?.data || {};
  };

  // 82. Audit Form Controls & Constraint Validation
  page.auditForms = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/forms/audit', payload);
    return res?.data || {};
  };

  // 83. Calculate Viewport Multi-Page Paging Metrics
  page.getViewportPaging = async function () {
    const res = await httpPost(agentPort, '/viewport/paging', {});
    return res?.data || {};
  };

  // 84. Inspect Computed Pseudo-Element Styles
  page.getComputedPseudoElement = async function (target, options = {}) {
    const payload = {
      target: String(target),
      pseudo: options.pseudo ? String(options.pseudo) : '::before',
      properties: Array.isArray(options.properties || options.props) ? (options.properties || options.props) : [],
    };
    const res = await httpPost(agentPort, '/element/pseudo', payload);
    return res?.data || {};
  };

  // 85. Simulate Clipboard Paste Event
  page.simulateClipboardPaste = async function (text, options = {}) {
    const payload = { text: String(text) };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/clipboard/paste', payload);
    return res?.data || {};
  };

  // 86. Smart Auto-Fill Form Controls by Semantic Mapping
  page.smartFill = async function (mapping, options = {}) {
    const payload = {
      mapping: typeof mapping === 'object' && mapping !== null ? mapping : {},
      submit: Boolean(options.submit),
    };
    const res = await httpPost(agentPort, '/forms/smart_fill', payload);
    return res?.data || {};
  };

  // 87. Observe Streaming Text Changes Until Settle
  page.observeText = async function (target, options = {}) {
    const payload = {
      target: String(target),
      timeout_ms: Number(options.timeoutMs || options.timeout_ms || options.timeout) || 10000,
      settle_ms: Number(options.settleMs || options.settle_ms || options.settle) || 1000,
    };
    const res = await httpPost(agentPort, '/observe/text', payload);
    return res?.data || {};
  };

  // 88. Emulate Page Visibility State
  page.setVisibilityState = async function (state = 'visible') {
    const res = await httpPost(agentPort, '/emulate/visibility', { state: String(state) });
    return res?.data || {};
  };

  // 89. Extract Computed Font Typography and Metrics
  page.getComputedFont = async function (target) {
    const res = await httpPost(agentPort, '/element/font', { target: String(target) });
    return res?.data || {};
  };

  // 90. Extract Interactive Action Map with Centroid Matrix
  page.getActionMap = async function (options = {}) {
    const payload = {
      viewport_only: options.viewportOnly !== undefined ? Boolean(options.viewportOnly) : true,
      limit: Number(options.limit || options.max || 50),
    };
    const res = await httpPost(agentPort, '/action/map', payload);
    return res?.data || {};
  };

  // 91. Inspect CSS Animation & Transition Motion State
  page.getMotionState = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/motion/state', payload);
    return res?.data || {};
  };

  // 92. Inspect Element Event Listeners & Interactive Handlers
  page.getElementListeners = async function (target) {
    const res = await httpPost(agentPort, '/element/listeners', { target: String(target) });
    return res?.data || {};
  };

  // 93. Extract Core Web Vitals & Navigation Timings
  page.getWebVitals = async function () {
    const res = await httpPost(agentPort, '/performance/vitals', {});
    return res?.data || {};
  };

  // 94. Inspect Select Options and Option Groups
  page.getSelectOptions = async function (target) {
    const res = await httpPost(agentPort, '/select/options', { target: String(target) });
    return res?.data || {};
  };

  // 95. Fuzzy Find Elements by Approximate Content / Attributes
  page.fuzzyFindElements = async function (query, options = {}) {
    const payload = {
      query: String(query),
      limit: Number(options.limit || options.max || 10),
    };
    if (options.container) payload.container = String(options.container);
    const res = await httpPost(agentPort, '/elements/fuzzy', payload);
    return res?.data || {};
  };

  // 96. Extract Hierarchical Heading Outline (h1-h6)
  page.getHeadingHierarchy = async function (options = {}) {
    const payload = {};
    if (options.container) payload.container = String(options.container);
    const res = await httpPost(agentPort, '/headings', payload);
    return res?.data || {};
  };

  // 97. Inspect CSS Stacking Context and Z-Index Hierarchy
  page.getElementStacking = async function (target) {
    const res = await httpPost(agentPort, '/element/stacking', { target: String(target) });
    return res?.data || {};
  };

  // 98. Query Elements Across Shadow DOM Subtrees
  page.queryShadowElements = async function (selector, options = {}) {
    const payload = {
      selector: String(selector),
      limit: Number(options.limit || options.max || 50),
    };
    if (options.container) payload.container = String(options.container);
    const res = await httpPost(agentPort, '/elements/shadow', payload);
    return res?.data || {};
  };

  // 99. Test 5-Point Visibility and Occlusion Matrix
  page.getElementVisibilityMatrix = async function (target) {
    const res = await httpPost(agentPort, '/element/visibility_matrix', { target: String(target) });
    return res?.data || {};
  };

  // 100. Retrieve Error and Unhandled Rejection Diagnostics Digest
  page.getErrorDigest = async function (options = {}) {
    const payload = {
      max_errors: Number(options.maxErrors || options.limit || 50),
      clear: Boolean(options.clear),
    };
    const res = await httpPost(agentPort, '/errors/digest', payload);
    return res?.data || {};
  };

  // 101. Extract Structured Records from Tables and ARIA Grids
  page.extractGridData = async function (options = {}) {
    const payload = {
      max_rows: Number(options.maxRows || options.limit || 100),
    };
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/grid/data', payload);
    return res?.data || {};
  };

  // 102. Inspect Pristine vs Dirty Form Inputs
  page.diffFormInputs = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/forms/diff', payload);
    return res?.data || {};
  };

  // 103. Neutralize Scroll Locks and Modal Traps
  page.unstickScrollLocks = async function () {
    const res = await httpPost(agentPort, '/viewport/unstick', {});
    return res?.data || {};
  };

  // 104. Diff Visible DOM Text from Checkpoint
  page.diffDomText = async function (options = {}) {
    const payload = {};
    if (options.baseline) payload.baseline = String(options.baseline);
    if (options.target) payload.target = String(options.target);
    const res = await httpPost(agentPort, '/dom/diff', payload);
    return res?.data || {};
  };

  // 105. Inspect Embedded Frame and Iframe Hierarchy
  page.getFrameHierarchy = async function () {
    const res = await httpGet(agentPort, '/frames/hierarchy');
    return res?.data || {};
  };

  // 106. Filter Interactive Elements by Semantic Role
  page.getRoleElements = async function (options = {}) {
    const payload = {
      roles: Array.isArray(options.roles) ? options.roles : [],
      viewport_only: options.viewportOnly !== undefined ? Boolean(options.viewportOnly) : true,
      limit: Number(options.limit || options.max || 50),
    };
    const res = await httpPost(agentPort, '/elements/roles', payload);
    return res?.data || {};
  };

  // 107. Harvest Page Resources with Resolved URLs
  page.harvestResources = async function (options = {}) {
    const payload = {
      resource_types: Array.isArray(options.types || options.resourceTypes) ? (options.types || options.resourceTypes) : [],
    };
    const res = await httpPost(agentPort, '/page/resources', payload);
    return res?.data || {};
  };

  // 108. Scan for Active Bot Challenges and CAPTCHAs
  page.detectBotChallenges = async function () {
    const res = await httpGet(agentPort, '/security/challenges');
    return res?.data || {};
  };

  // 109. Inspect Scrollability Metrics and Boundaries
  page.getScrollState = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/scroll/state', payload);
    return res?.data || {};
  };

  // 110. Find Nearby Elements by Viewport Proximity
  page.getProximityElements = async function (options = {}) {
    const payload = {
      limit: Number(options.limit || options.max || 10),
      interactive_only: options.interactiveOnly !== undefined ? Boolean(options.interactiveOnly) : true,
    };
    if (options.target || options.ref) payload.target = String(options.target || options.ref);
    if (options.x !== undefined) payload.x = Number(options.x);
    if (options.y !== undefined) payload.y = Number(options.y);
    if (options.radius !== undefined) payload.radius = Number(options.radius);
    const res = await httpPost(agentPort, '/elements/proximity', payload);
    return res?.data || {};
  };

  // 111. Emulate Viewport Metrics and Media Features
  page.emulateScreenMetrics = async function (options = {}) {
    const payload = {
      mobile: Boolean(options.mobile),
    };
    if (options.width !== undefined) payload.width = Number(options.width);
    if (options.height !== undefined) payload.height = Number(options.height);
    if (options.deviceScaleFactor !== undefined || options.scale !== undefined) {
      payload.device_scale_factor = Number(options.deviceScaleFactor || options.scale);
    }
    if (options.orientation) payload.orientation = String(options.orientation);
    if (options.colorScheme || options.theme) payload.color_scheme = String(options.colorScheme || options.theme);
    const res = await httpPost(agentPort, '/screen/metrics', payload);
    return res?.data || {};
  };

  // 112. Export Table/Grid directly to RFC 4180 CSV
  page.exportTableCsv = async function (options = {}) {
    const payload = {
      include_headers: options.includeHeaders !== undefined ? Boolean(options.includeHeaders) : true,
      delimiter: options.delimiter ? String(options.delimiter) : ',',
    };
    if (options.target || options.ref) payload.target = String(options.target || options.ref);
    const res = await httpPost(agentPort, '/table/csv', payload);
    return res?.data || {};
  };

  // 113. Reset Form Controls to Pristine Defaults
  page.resetForm = async function (options = {}) {
    const payload = {
      trigger_events: options.triggerEvents !== undefined ? Boolean(options.triggerEvents) : true,
      clear_custom_validations: options.clearValidations !== undefined ? Boolean(options.clearValidations) : true,
    };
    if (typeof options === 'string') {
      payload.target = options;
    } else if (options.target || options.ref) {
      payload.target = String(options.target || options.ref);
    }
    const res = await httpPost(agentPort, '/form/reset', payload);
    return res?.data || {};
  };

  // 114. Inspect Datalist or Autocomplete Combobox Options
  page.getDatalistOptions = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/elements/datalist', payload);
    return res?.data || {};
  };

  // 115. Inspect Canvas Context and Hardware Metrics
  page.getCanvasContext = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/canvas/context', payload);
    return res?.data || {};
  };

  // 116. Control HTML5 Media Playback and Telemetry
  page.controlMediaPlayback = async function (options = {}) {
    const payload = {
      action: String(options.action || 'status'),
    };
    if (options.target || options.ref) payload.target = String(options.target || options.ref);
    if (options.value !== undefined || options.val !== undefined) {
      payload.value = Number(options.value !== undefined ? options.value : options.val);
    }
    const res = await httpPost(agentPort, '/media/playback', payload);
    return res?.data || {};
  };

  // 117. Inspect DOM Ancestry and Selector Hierarchy Path
  page.getElementAncestry = async function (target, options = {}) {
    const payload = {
      target: String(target),
      max_depth: Number(options.maxDepth || options.depth || 20),
    };
    const res = await httpPost(agentPort, '/elements/ancestry', payload);
    return res?.data || {};
  };

  // 118. Inspect CSS Grid and Flexbox Layout Metrics
  page.getLayoutMetrics = async function (target = null) {
    const payload = {};
    if (target) payload.target = String(target);
    const res = await httpPost(agentPort, '/layout/metrics', payload);
    return res?.data || {};
  };

  // 119. Compute WCAG Contrast Ratio and Compliance
  page.auditContrastRatio = async function (target) {
    const payload = {
      target: String(target),
    };
    const res = await httpPost(agentPort, '/accessibility/contrast', payload);
    return res?.data || {};
  };

  // 120. Dispatch Advanced Event with Synthetic Modifiers
  page.dispatchAdvancedEvent = async function (target, options = {}) {
    const payload = {
      target: String(target),
      event_type: String(options.eventType || options.event || 'click'),
      modifiers: Array.isArray(options.modifiers) ? options.modifiers : [],
      button: Number(options.button !== undefined ? options.button : 0),
      pressure: Number(options.pressure !== undefined ? options.pressure : 0.5),
      detail: Number(options.detail !== undefined ? options.detail : 1),
    };
    const res = await httpPost(agentPort, '/events/dispatch_advanced', payload);
    return res?.data || {};
  };

  // 121. Harvest Network Activity Logs in HAR 1.2 Format
  page.getNetworkHar = async function (options = {}) {
    const payload = {
      clear: Boolean(options.clear),
    };
    const res = await httpPost(agentPort, '/network/har', payload);
    return res?.data || {};
  };

  // 122. Inspect or Set Caret Position and Selection Offsets
  page.getCaretPosition = async function (target, options = {}) {
    const payload = {
      target: String(target),
    };
    if (options.setStart !== undefined) payload.set_start = Number(options.setStart);
    if (options.setEnd !== undefined) payload.set_end = Number(options.setEnd);
    const res = await httpPost(agentPort, '/caret/position', payload);
    return res?.data || {};
  };

  // 123. Dismiss GDPR / CCPA Cookie Consent Banners
  page.dismissCookieBanners = async function (options = {}) {
    const payload = {
      click_accept: options.clickAccept !== false,
    };
    const res = await httpPost(agentPort, '/consent/dismiss', payload);
    return res?.data || {};
  };

  // 124. Emulate Web Speech API
  page.emulateSpeechApi = async function (options = {}) {
    const payload = {};
    if (options.action) payload.action = String(options.action);
    if (options.mockTranscript || options.transcript) {
      payload.mock_transcript = String(options.mockTranscript || options.transcript);
    }
    const res = await httpPost(agentPort, '/speech/emulate', payload);
    return res?.data || {};
  };

  // 125. Harvest Real-Time DOM Mutation Stream
  page.getDomMutations = async function (options = {}) {
    const payload = {
      limit: options.limit ? Number(options.limit) : 50,
      clear: options.clear !== false,
    };
    const res = await httpPost(agentPort, '/dom/mutations', payload);
    return res?.data || {};
  };

  // 126. Inspect Active Route Interception Rules
  page.getActiveRoutes = async function () {
    const res = await httpGet(agentPort, '/routes/active');
    return res?.data || {};
  };

  // 127. Execute Native Rich-Text Editor Command
  page.execEditorCommand = async function (command, options = {}) {
    const payload = {
      command: String(command),
    };
    if (options.value !== undefined) payload.value = String(options.value);
    const res = await httpPost(agentPort, '/editor/command', payload);
    return res?.data || {};
  };

  // 128. Force Preload and Hydration of Lazy-Loaded Resources
  page.preloadLazyResources = async function (options = {}) {
    const payload = {
      force_all: options.forceAll !== false,
    };
    if (options.container) payload.container = String(options.container);
    const res = await httpPost(agentPort, '/resources/preload-lazy', payload);
    return res?.data || {};
  };

  // 129. Emulate Window and Document Focus State
  page.emulateWindowFocus = async function (options = {}) {
    const payload = {
      focused: options.focused !== false,
      emit_events: options.emitEvents !== false,
    };
    const res = await httpPost(agentPort, '/window/focus', payload);
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

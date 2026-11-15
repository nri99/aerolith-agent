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

  // 4d. Network Activity Inspection
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

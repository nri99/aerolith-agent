const { test } = require('node:test');
const assert = require('node:assert/strict');
const http = require('node:http');
const { extendPage, connectAerolith } = require('./index');

process.env.AEROLITH_AGENT_TOKEN = 'test-only-agent-secret-with-at-least-32-characters';

async function withServer(handler, run) {
  const server = http.createServer(handler);
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try { await run(server.address().port); }
  finally {
    server.closeIdleConnections();
    await new Promise(resolve => server.close(resolve));
  }
}

test('native Playwright methods remain intact; extensions are namespaced', () => {
  const hover = () => 'native';
  const selectOption = () => 'native select';
  const native = { hover, selectOption };
  assert.equal(extendPage(native), native);
  assert.equal(native.hover, hover);
  assert.equal(native.selectOption, selectOption);
  assert.equal(typeof native.aerolith.hover, 'function');
  assert.equal(native.semanticSnapshot, native.aerolith.semanticSnapshot);
});

test('SDK authenticates requests and preserves selector encoding', async () => {
  await withServer((req, res) => {
    assert.equal(req.headers.authorization, 'Bearer ' + process.env.AEROLITH_AGENT_TOKEN);
    assert.equal(new URL(req.url, 'http://localhost').searchParams.get('target'), '#email[name="a&b"]');
    res.end(JSON.stringify({ success: true, data: { text: 'hello' } }));
  }, async port => {
    assert.equal(await extendPage({}, port).getText('#email[name="a&b"]'), 'hello');
  });
});

test('HTTP errors reject instead of becoming empty snapshots', async () => {
  await withServer((_req, res) => {
    res.writeHead(500);
    res.end(JSON.stringify({ success: false, error: 'Action timed out' }));
  }, async port => {
    await assert.rejects(extendPage({}, port).semanticSnapshot(), /Action timed out/);
  });
});

test('failed command envelopes reject even when HTTP status is 200', async () => {
  await withServer((_req, res) => res.end(JSON.stringify({ success: false, error: 'No target' })), async port => {
    await assert.rejects(extendPage({}, port).highlight('1'), /No target/);
  });
});

test('connection selects the active content page rather than browser chrome', async () => {
  const chrome = { evaluate: async () => undefined };
  const background = { evaluate: async () => 1 };
  const active = { evaluate: async () => 2 };
  const browser = { contexts: () => [{ pages: () => [chrome, background, active] }] };
  await withServer((_req, res) => res.end(JSON.stringify({ success: true, data: { tabs: [{ id: 2, is_active: true }] } })), async port => {
    const result = await connectAerolith({ chromium: { connectOverCDP: async () => browser } }, { agentPort: port });
    assert.equal(result.page, active);
  });
});

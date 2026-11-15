/**
 * Aerolith Agent + Node.js Playwright Example.
 */

const playwright = require('playwright');
const { connectAerolith } = require('./index');

(async () => {
  console.log('Connecting Playwright to Aerolith Agent...');
  const { browser, page } = await connectAerolith(playwright, {
    cdpUrl: 'http://127.0.0.1:9222',
    agentPort: 8765
  });

  console.log('Navigating to example page...');
  await page.goto('https://duckduckgo.com');
  await page.waitForLoadState('networkidle');

  console.log('\n--- Semantic Accessibility Snapshot ---');
  const snapshot = await page.semanticSnapshot({ viewportOnly: true });
  console.log(snapshot.slice(0, 500) + '...');

  console.log('\nHighlighting search input...');
  await page.highlight('input[name=q]');

  console.log('Filling search query atomically...');
  await page.fillForm([{ target: 'input[name=q]', value: 'Aerolith Agent' }], { submit: true });

  await page.waitForTimeout(2000);

  console.log('Capturing Set-Of-Marks screenshot...');
  await page.screenshotSom({ path: 'duckduckgo_som.png' });

  console.log('Done! Node.js Playwright session finished successfully.');
  await browser.close();
})();

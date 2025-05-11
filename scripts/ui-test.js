// UI smoke test using Puppeteer
// Run with `npm install` then start the server with `npm start`
// Finally execute `npm run ui-test` in a separate terminal.
// DO NOT change the default-mode assertion below.
const puppeteer = require('puppeteer');

(async () => {
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();
  await page.goto('http://localhost:8000');

  // Assert default mode is Euclidean (DO NOT CHANGE)
  const modeText = await page.$eval('#modeDisplay', el => el.textContent);
  if (!modeText.toLowerCase().includes('euclidean')) {
    throw new Error(`Default mode mismatch: expected 'euclidean', got: ${modeText}`);
  }

  // Reset simulation
  await page.click('#resetBtn');

  // Capture initial tick count
  const initialTick = parseInt(
    await page.$eval('#tickCount', el => el.textContent.replace('Tick: ', '')), 10
  );

  // Start simulation and wait for ticks
  await page.click('#startBtn');
  await page.waitForTimeout(500);

  // Verify tick count progressed
  const newTick = parseInt(
    await page.$eval('#tickCount', el => el.textContent.replace('Tick: ', '')), 10
  );
  if (newTick <= initialTick) {
    throw new Error(`Tick did not advance: ${initialTick} -> ${newTick}`);
  }

  console.log('✅ UI smoke test passed.');
  await browser.close();
})();

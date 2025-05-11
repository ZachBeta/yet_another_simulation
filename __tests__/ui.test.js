/**
 * @jest-environment jsdom
 */
/**
 * @jest-environment jsdom
 */

const fs = require('fs');
const path = require('path');

test('default mode is euclidean', () => {
  // Load HTML fixture
  const html = fs.readFileSync(path.resolve(__dirname, '../index.html'), 'utf8');
  document.documentElement.innerHTML = html;

  // Check display text for default mode
  const modeDisplay = document.getElementById('modeDisplay');
  expect(modeDisplay).not.toBeNull();
  expect(modeDisplay.textContent.toLowerCase()).toContain('euclidean');
});

test('buttons exist and click without error', () => {
  // Load HTML fixture
  const html = fs.readFileSync(path.resolve(__dirname, '../index.html'), 'utf8');
  document.documentElement.innerHTML = html;

  const resetBtn = document.getElementById('resetBtn');
  const startBtn = document.getElementById('startBtn');
  const pauseBtn = document.getElementById('pauseBtn');

  expect(resetBtn).not.toBeNull();
  expect(startBtn).not.toBeNull();
  expect(pauseBtn).not.toBeNull();

  // Clicking should not throw
  expect(() => resetBtn.click()).not.toThrow();
  expect(() => startBtn.click()).not.toThrow();
  expect(() => pauseBtn.click()).not.toThrow();
});

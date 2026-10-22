import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readTheme, applyTheme } from './theme.ts';

test('default, persistence, live system changes, overrides, invalid storage and cleanup', () => {
  let saved = null;
  const listeners = new Set();
  const media = { matches: false, addEventListener: (_, f) => listeners.add(f), removeEventListener: (_, f) => listeners.delete(f) };
  globalThis.localStorage = { getItem: () => saved };
  globalThis.window = { matchMedia: () => media };
  globalThis.document = { documentElement: { dataset: {}, style: {} } };
  assert.equal(readTheme(), 'auto');
  let cleanup = applyTheme(readTheme());
  assert.equal(document.documentElement.dataset.theme, 'light');
  media.matches = true; listeners.forEach(f => f());
  assert.equal(document.documentElement.dataset.theme, 'dark');
  cleanup(); assert.equal(listeners.size, 0);
  saved = 'light'; cleanup = applyTheme(readTheme());
  assert.equal(document.documentElement.dataset.theme, 'light');
  media.matches = false; listeners.forEach(f => f());
  media.matches = true; listeners.forEach(f => f());
  assert.equal(document.documentElement.dataset.theme, 'light'); cleanup();
  saved = 'dark'; cleanup = applyTheme(readTheme());
  assert.equal(document.documentElement.style.colorScheme, 'dark'); cleanup();
  saved = 'invalid'; assert.equal(readTheme(), 'auto');
  localStorage.getItem = () => { throw Error('blocked'); };
  assert.equal(readTheme(), 'auto');
});

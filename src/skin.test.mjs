import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readSkin, applySkin, SKIN_KEY } from './skin.ts';

test('skin defaults preserve classic installations, restores opt-in, tolerates blocked storage', () => {
  let saved = null;
  globalThis.localStorage = { getItem: key => { assert.equal(key, SKIN_KEY); return saved; } };
  globalThis.document = { documentElement: { dataset: { theme: 'dark' } } };
  assert.equal(readSkin(), 'classic');
  saved = 'compact'; applySkin(readSkin());
  assert.equal(document.documentElement.dataset.skin, 'compact');
  assert.equal(document.documentElement.dataset.theme, 'dark');
  saved = 'invalid'; applySkin(readSkin());
  assert.equal(document.documentElement.dataset.skin, 'classic');
  localStorage.getItem = () => { throw Error('blocked'); };
  assert.equal(readSkin(), 'classic');
});

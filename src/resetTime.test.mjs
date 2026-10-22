import { test } from 'node:test';
import assert from 'node:assert/strict';
import { formatResetTime, toResetDate } from './resetTime.ts';

test('reset timestamps use local timezone, including DST and provider formats', () => {
  const previous = process.env.TZ;
  try {
    const instant = '2026-11-02T08:53:16Z';
    const milliseconds = Date.parse(instant);
    for (const [zone, expected] of [
      ['UTC', '2026.11.02 08:53:16'],
      ['Asia/Shanghai', '2026.11.02 16:53:16'],
      ['America/New_York', '2026.11.02 03:53:16'],
    ]) {
      process.env.TZ = zone;
      for (const input of [instant, '2026-11-02T16:53:16+08:00', milliseconds, milliseconds / 1000, String(milliseconds), String(milliseconds / 1000)]) {
        assert.equal(formatResetTime(input), expected);
      }
    }
    process.env.TZ = 'America/New_York';
    assert.equal(formatResetTime('2026-11-01T05:59:59Z'), '2026.11.01 01:59:59');
    assert.equal(formatResetTime('2026-11-01T06:00:00Z'), '2026.11.01 01:00:00');
    process.env.TZ = 'Asia/Shanghai';
    assert.equal(formatResetTime('2026-11-01T16:00:00Z'), '2026.11.02 00:00:00');
    for (const input of [null, '', '  ', 'invalid', NaN, Infinity, 1e20]) {
      assert.equal(toResetDate(input), null);
      assert.equal(formatResetTime(input), null);
    }
  } finally {
    if (previous == null) delete process.env.TZ;
    else process.env.TZ = previous;
  }
});

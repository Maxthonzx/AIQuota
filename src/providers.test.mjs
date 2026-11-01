import test from "node:test";
import assert from "node:assert/strict";
import {
  readConnections,
  serializeConnections,
  requestIsCurrent,
  makeConnection,
  PROVIDERS,
} from "./providers.ts";
const saved = (value) => ({ getItem: () => value });
test("first run differs from deliberately removing every provider", () => {
  assert.equal(readConnections(saved(null)), null);
  assert.deepEqual(readConnections(saved("[]")), []);
  assert.throws(() => readConnections(saved("broken")));
  assert.throws(() => readConnections(saved("{}")));
});
test("round trips provider configuration but never serializes an injected secret", () => {
  const connection = {
    ...makeConnection(PROVIDERS[0]),
    key: "must-never-be-stored",
  };
  const custom = {
    id: "custom-test",
    provider: "custom",
    name: "My plan",
    type: "subscription",
    custom: {
      id: "custom-test",
      endpoint: "https://example.com/quota",
      auth: "bearer",
      header: "x-api-key",
      metric: "balance",
      value_path: "data.balance",
      total_path: "",
      reset_path: "",
      unit: "USD",
      key: "also-never-stored",
    },
  };
  const serialized = serializeConnections([connection, custom]);
  assert.ok(!serialized.includes("never-stored"));
  assert.ok(!serialized.includes("must-never-be-stored"));
  assert.equal(
    readConnections(saved(serialized))[1].custom.value_path,
    "data.balance",
  );
  assert.throws(() =>
    readConnections(saved(JSON.stringify([connection, connection]))),
  );
});
test("deleted, edited, and superseded requests cannot resurrect stale cards", () => {
  const snapshot = makeConnection(PROVIDERS[0]);
  assert.equal(requestIsCurrent(snapshot, [], 1, 1), false);
  assert.equal(
    requestIsCurrent(snapshot, [{ ...snapshot, name: "edited" }], 1, 1),
    false,
  );
  assert.equal(requestIsCurrent(snapshot, [snapshot], 1, 2), false);
  assert.equal(requestIsCurrent(snapshot, [snapshot], 2, 2), true);
});

test("plan expiry fields round trip and legacy configs without them still load", () => {
  const custom = {
    id: "custom-test",
    provider: "custom",
    name: "My plan",
    type: "subscription",
    plan_expiry: "2026-05-12T14:30",
    custom: {
      id: "custom-test",
      endpoint: "https://example.com/quota",
      auth: "none",
      header: "x-api-key",
      metric: "balance",
      value_path: "data.balance",
      total_path: "",
      reset_path: "",
      expiry_path: "data.expire_at",
      unit: "USD",
    },
  };
  const loaded = readConnections(saved(serializeConnections([custom])));
  assert.equal(loaded[0].plan_expiry, "2026-05-12T14:30");
  assert.equal(loaded[0].custom.expiry_path, "data.expire_at");
  // 旧配置没有这两个字段也必须能读；读取后 custom.expiry_path 规范化为空串
  const legacy = JSON.parse(JSON.stringify(custom));
  delete legacy.plan_expiry;
  delete legacy.custom.expiry_path;
  const legacyLoaded = readConnections(saved(JSON.stringify([legacy])));
  assert.equal(legacyLoaded[0].plan_expiry, undefined);
  assert.equal(legacyLoaded[0].custom.expiry_path, "");
  // 字段类型不对要报错，不能静默接受
  const bad = JSON.parse(JSON.stringify(legacy));
  bad.plan_expiry = 123;
  assert.throws(() => readConnections(saved(JSON.stringify([bad]))));
  const badPath = JSON.parse(JSON.stringify(legacy));
  badPath.custom.expiry_path = 123;
  assert.throws(() => readConnections(saved(JSON.stringify([badPath]))));
});

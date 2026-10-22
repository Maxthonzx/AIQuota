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

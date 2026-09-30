const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const helper = path.join(__dirname, "../native/target/release/phone-passkey-helper");

test("native helper validates RP IDs before using Bluetooth", { skip: !fs.existsSync(helper) && "Build the native helper to run its integration checks" }, () => {
  for (const [origin, rpId, expected] of [
    ["https://login.microsoftonline.com", "login.microsoftonline.com", "validated"],
    ["https://login.microsoftonline.com", "microsoftonline.com", "validated"],
    ["https://login.microsoftonline.com", "evil.example", "error"],
    ["https://login.microsoftonline.com", "com", "error"],
    ["http://login.microsoftonline.com", "login.microsoftonline.com", "error"],
  ]) {
    const result = spawnSync(helper, [], { input: JSON.stringify({
      origin, publicKey: { challenge: "AQID", rpId, timeout: 5000 }, validateOnly: true,
    }), encoding: "utf8", timeout: 6000 });
    assert.equal(result.stderr, "");
    assert.equal(JSON.parse(result.stdout).type, expected);
  }
});

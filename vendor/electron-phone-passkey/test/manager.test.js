const test = require("node:test");
const assert = require("node:assert/strict");
const { EventEmitter } = require("node:events");
const path = require("node:path");
const fs = require("node:fs");
const { createPhonePasskeySupport, CHANNELS } = require("..");

test("IPC authenticates registered frames and ignores renderer-supplied origins", async () => {
  const ipcMain = new EventEmitter();
  const handlers = new Map();
  ipcMain.handle = (channel, handler) => handlers.set(channel, handler);
  ipcMain.removeHandler = (channel) => handlers.delete(channel);
  let registered = 0;
  let unregistered = 0;
  const session = {
    registerPreloadScript: () => { registered++; return "test-preload"; },
    unregisterPreloadScript: () => { unregistered++; },
  };
  const contents = Object.assign(new EventEmitter(), { id: 1, session, isDestroyed: () => false });
  const frame = { url: "https://login.microsoftonline.com/test", processId: 1, routingId: 2, parent: null };
  frame.top = frame;
  const event = { sender: contents, senderFrame: frame };
  const helper = path.join(__dirname, "../scripts/fixtures/fake-passkey-helper.cjs");
  fs.chmodSync(helper, 0o755);
  const support = createPhonePasskeySupport({ electron: { ipcMain, BrowserWindow: {} }, helperPath: helper });
  const get = handlers.get(CHANNELS.get);
  const request = { id: "request", origin: "https://evil.example", publicKey: { challenge: "AQID", timeout: 3000 } };
  try {
    assert.equal((await get(event, request)).error.name, "SecurityError");
    support.attach(contents);
    support.attach(contents);
    assert.equal(registered, 1);
    ipcMain.emit(CHANNELS.config, event);
    assert.equal(event.returnValue.enabled, true);
    const result = await get(event, request);
    assert.equal(result.ok, true);
    const client = JSON.parse(Buffer.from(result.credential.response.clientDataJSON, "base64url"));
    assert.equal(client.origin, "https://login.microsoftonline.com");
    frame.url = "https://evil.example/test";
    assert.equal((await get(event, request)).error.name, "SecurityError");
    ipcMain.emit(CHANNELS.config, event);
    assert.equal(event.returnValue, null);
    frame.url = "https://login.microsoftonline.com/test";
    contents.emit("destroyed");
    assert.equal((await get(event, request)).error.name, "SecurityError");
  } finally {
    support.dispose();
  }
  assert.equal(unregistered, 1);
  assert.equal(handlers.size, 0);
  assert.equal(ipcMain.listenerCount(CHANNELS.config), 0);
});

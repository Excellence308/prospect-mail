const test = require('node:test');
const assert = require('node:assert/strict');
const { LoginNotificationPolicy } = require('../src/login-notification-policy');

test('initial sign-in and visible authentication never notify', () => {
  const policy = new LoginNotificationPolicy();
  assert.equal(policy.shouldNotify('login-page', { visible: false, now: 0 }), false);
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 0 }), false);
  policy.markMailboxReady();
  assert.equal(policy.shouldNotify('login-page', { visible: true, now: 0 }), false);
  assert.equal(policy.shouldNotify('session-expired', { visible: true, now: 0 }), false);
  assert.equal(policy.shouldNotify('unknown', { visible: false, now: 0 }), false);
});

test('hidden expired sessions notify once across repeated login documents', () => {
  const policy = new LoginNotificationPolicy();
  policy.markMailboxReady();
  assert.equal(policy.shouldNotify('login-page', { visible: false, now: 0 }), true);
  assert.equal(policy.shouldNotify('login-page', { visible: false, now: 240000 }), false);
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 240000 }), false);
  policy.markMailboxReady();
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 240000 }), true);
});

test('a rapidly recovering session cannot repeatedly notify', () => {
  const policy = new LoginNotificationPolicy();
  policy.markMailboxReady();
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 0 }), true);
  policy.markMailboxReady();
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 1000 }), false);
  assert.equal(policy.shouldNotify('session-expired', { visible: false, now: 180000 }), true);
});

# Electron phone passkeys (experimental)

This opt-in component routes **sign-in assertions** to a pinned build of
[libwebauthn](https://github.com/linux-credentials/libwebauthn). It displays a
local QR window and returns the phone's signed response to the original app.
It does not import cookies, store passkeys, change browser profiles, or implement
passkey enrolment. `navigator.credentials.create()` remains native.

## Build and enable

```bash
cargo build --release --locked --manifest-path native/Cargo.toml
```

Keep `native/target/release/public_suffix_list.dat` next to
`phone-passkey-helper`. A valid system list takes precedence; the bundled
MPL-2.0 snapshot is pinned in `native/data/SOURCE.json`. Refresh the snapshot
with releases. The backend source revision and Rust dependencies are pinned
in `native/Cargo.toml` and `native/Cargo.lock`.

Requirements: Linux, a recent Rust toolchain, libudev/dbus development libraries
at build time, a powered Bluetooth adapter and BlueZ at runtime, phone camera,
an existing phone passkey accepted by the relying party, and network access for
the caBLE tunnel. The phone must be nearby. No passkey registration or changes
to an employer's authentication policy are performed.

The main process calls `createPhonePasskeySupport({ electron, helperPath,
extraOrigins })`, then `attach(window.webContents)` **before loading any remote
page**. Use an absolute helper path. Call `dispose()` before quitting.
Enable `nodeIntegrationInSubFrames` only for this opt-in mode, keep
`nodeIntegration: false`, and guard the application's ordinary preload with
`process.isMainFrame`. The session preload exposes only credential request and
cancellation operations. Tested remote main frames and subframes have no
`require` or `process` globals. Do not combine windows with different security
preferences in the same Electron session; the two apps use separate sessions.

Default calling origins are the exact HTTPS Microsoft login origins. Federated
identity providers require explicit `extraOrigins`, without paths or wildcards.

## Security boundaries

- Requests are restricted to registered app WebContents and allowlisted frames.
- Signing origin and cross-origin/top-origin metadata come from Electron's
  frame tree. Renderer-supplied origin fields are never used.
- The backend checks the RP ID against the origin and public suffix list.
  Related-origin exceptions are disabled.
- The isolated preload checks WebAuthn Permissions Policy for iframe calls.
  The Teams integration retains its existing non-isolated renderer model.
- Background/conditional/silent requests fall through without showing a QR.
- Requests are bounded in size, duration and concurrency. Page aborts, window
  destruction, renderer loss and navigation cancel the backend.
- The helper runs without a shell; requests and responses use anonymous pipes.
  Credential material, QR payloads, backend stderr and configured paths are not
  logged by this component.
- The QR window uses a separate nonpersistent session, context isolation,
  sandboxing, no page Node integration, a restrictive CSP, and no navigation.
- Local configuration chooses the helper executable. Treat that executable as
  trusted authentication code; a website cannot supply its path.

## Checks and limitations

```bash
node --test --test-isolation=none test/*.test.js
cargo fmt --manifest-path native/Cargo.toml -- --check
cargo clippy --locked --manifest-path native/Cargo.toml -- -D warnings
```

Unit tests cover bounds, exact-origin gates, fragmented protocol output,
timeout, cancellation and helper failure. Synthetic Electron checks cover
credential prototypes, cross-origin login frames and absence of page Node
globals. The native helper has been checked with valid and rejected RP IDs.
Those checks **do not prove a successful company login**. A real phone
transaction, relying-party acceptance and post-login app behaviour still need
interactive validation.

This is an experimental local integration, not an upstream-supported feature.
The native helper statically links an LGPL-2.1-or-later library: distribute its
corresponding pinned source, build files, notices and materials needed to
relink it. The JavaScript bridge and helper wrapper are MIT licensed. The
bundled public suffix data carries its MPL-2.0 notice in the file header.

## IPC contract

| Channel | Direction | Purpose |
| --- | --- | --- |
| `phone-passkey:config` | renderer → main, synchronous | Credential-free document-start feature gate |
| `phone-passkey:get` | renderer → main, invoke | Bounded assertion request; returns IDL credential JSON or a DOM error |
| `phone-passkey:cancel` | renderer → main | Cancel the same frame's active request by ID |
| `phone-passkey:prompt-cancel` | local prompt → main | Cancel the request belonging to that prompt |
| `phone-passkey:qr` | main → local prompt | QR SVG and validated sign-in origin |

The native helper consumes one `{origin, topOrigin?, publicKey}` JSON object on
stdin and emits newline-delimited `{type: "qr", svg}`, followed by exactly one
`{type: "result", credential}` or `{type: "error", name, message}`. `validateOnly`
is a native diagnostic mode used by tests; it is not exposed to remote pages.

The Electron renderer smoke test uses synthetic assertions only (no real authentication). Run it with the consuming application's Electron executable:

```sh
./node_modules/.bin/electron vendor/electron-phone-passkey/scripts/renderer-smoke.cjs
```

It checks isolated and non-isolated main frames and cross-origin iframes, credential prototypes, signed origin context, and absence of Node globals. Native tests require building the helper first.

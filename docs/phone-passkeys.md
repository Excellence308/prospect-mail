# Experimental phone-passkey sign-in on Linux

The optional phone backend displays the QR prompt missing from Electron's
Linux login flow. The phone's assertion returns to Prospect's own session;
mail links still open in your configured external browser. Helium's profiles,
closed tabs and restart history remain separate.

Build the helper from this checkout:

```bash
cargo build --release --locked --manifest-path vendor/electron-phone-passkey/native/Cargo.toml
```

Use the tray menu to open `settings.json`, add the following, set `helperPath`
to the **absolute** path of the resulting executable, then restart Prospect:

```json
{
  "phonePasskey": {
    "enabled": true,
    "helperPath": "/absolute/path/to/phone-passkey-helper",
    "extraOrigins": []
  }
}
```

Keep `public_suffix_list.dat` next to the helper. A phone passkey already
registered with your organisation, Bluetooth and nearby phone are required.
An organisation's own sign-in host needs its exact HTTPS origin added to
`extraOrigins`. No wildcards, paths or HTTP origins are accepted.

Use `PROSPECT_MAIL_USER_DATA_DIR=/absolute/path/to/test-profile` to run a
prototype with separate app data, leaving your installed app's cookies and
settings untouched. Ordinary launches continue to use the existing profile.

> [!IMPORTANT]
> This is an opt-in prototype. Real tenant sign-in has not been validated.
> Passkey enrolment is not implemented. macOS and Windows retain native
> authentication. Disabling `phonePasskey.enabled` restores the existing flow.

See the [shared component](../vendor/electron-phone-passkey/README.md) for
security boundaries, dependencies, build/licensing requirements and tests.

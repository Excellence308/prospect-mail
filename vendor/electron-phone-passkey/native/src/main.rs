use std::io::{self, Read, Write};
use std::time::Duration;

use libwebauthn::ops::webauthn::{
    DatFilePublicSuffixList, GetAssertionRequest, JsonFormat, OriginValidation, PublicSuffixList,
    RelatedOrigins, RequestOrigin, RequestSettings, SystemPublicSuffixList, WebAuthnIDLResponse,
};
use libwebauthn::transport::cable::qr_code_device::{
    CableQrCodeDevice, CableTransports, QrCodeOperationHint,
};
use libwebauthn::transport::{ChannelSettings, Device};
use libwebauthn::webauthn::WebAuthn;
use qrcode::{render::svg, QrCode};
use serde::Deserialize;
use serde_json::{json, Value};

const MAX_REQUEST: u64 = 1024 * 1024;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Request {
    origin: String,
    top_origin: Option<String>,
    public_key: Value,
    validate_only: Option<bool>,
}

fn emit(message: Value) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &message)?;
    writeln!(stdout)?;
    stdout.flush()
}

async fn authenticate(input: Request) -> Result<(), (&'static str, &'static str)> {
    let mut origin: RequestOrigin = input
        .origin
        .as_str()
        .try_into()
        .map_err(|_| ("SecurityError", "Invalid sign-in origin."))?;
    if !input.origin.starts_with("https://") {
        return Err(("SecurityError", "Sign-in requires HTTPS."));
    }
    if let Some(top) = input.top_origin {
        if !top.starts_with("https://") {
            return Err(("SecurityError", "Sign-in requires HTTPS."));
        }
        origin.top_origin = Some(
            top.as_str()
                .try_into()
                .map_err(|_| ("SecurityError", "Invalid top-level origin."))?,
        );
    }
    let psl: Box<dyn PublicSuffixList> = match SystemPublicSuffixList::auto() {
        Ok(psl) => Box::new(psl),
        Err(_) => {
            let bundled = std::env::current_exe()
                .map_err(|_| {
                    (
                        "NotSupportedError",
                        "Could not locate the public suffix list.",
                    )
                })?
                .with_file_name("public_suffix_list.dat");
            Box::new(DatFilePublicSuffixList::from_path(bundled).map_err(|_| {
                (
                    "NotSupportedError",
                    "The public suffix list is missing or invalid.",
                )
            })?)
        }
    };
    let request = GetAssertionRequest::prepare(
        &origin,
        &input.public_key.to_string(),
        &RequestSettings {
            origin: OriginValidation::Validate {
                public_suffix_list: psl.as_ref(),
                related_origins: RelatedOrigins::Disabled,
            },
        },
    )
    .await
    .map_err(|_| {
        (
            "SecurityError",
            "Invalid passkey request or relying-party origin.",
        )
    })?;
    if input.validate_only == Some(true) {
        return emit(json!({"type": "validated"}))
            .map_err(|_| ("OperationError", "Could not return validation result."));
    }
    if !libwebauthn::transport::cable::is_available().await {
        return Err((
            "NotSupportedError",
            "Turn on Bluetooth to use a phone passkey.",
        ));
    }
    let mut device = CableQrCodeDevice::new_persistent(
        QrCodeOperationHint::GetAssertionRequest,
        std::sync::Arc::new(
            libwebauthn::transport::cable::known_devices::EphemeralDeviceInfoStore::default(),
        ),
        CableTransports::CloudAssistedOnly,
    )
    .map_err(|_| ("OperationError", "Could not start phone authentication."))?;
    let qr = QrCode::new(device.qr_code.to_string())
        .map_err(|_| ("OperationError", "Could not create the QR code."))?
        .render::<svg::Color>()
        .min_dimensions(320, 320)
        .build();
    emit(json!({"type": "qr", "svg": qr}))
        .map_err(|_| ("OperationError", "Could not display the QR code."))?;
    let mut channel = device
        .channel(ChannelSettings::default())
        .await
        .map_err(|_| {
            (
                "NotAllowedError",
                "Could not connect to your phone. Try again.",
            )
        })?;
    let response = channel
        .webauthn_get_assertion(&request)
        .await
        .map_err(|_| ("NotAllowedError", "Phone authentication did not complete."))?;
    if response.assertions.len() != 1 {
        return Err((
            "NotAllowedError",
            "Choose one sign-in account on your phone.",
        ));
    }
    let credential = response.assertions[0]
        .to_json_string(&request, JsonFormat::default())
        .map_err(|_| {
            (
                "OperationError",
                "Could not return the authentication response.",
            )
        })?;
    let credential: Value = serde_json::from_str(&credential).map_err(|_| {
        (
            "OperationError",
            "Could not encode the authentication response.",
        )
    })?;
    emit(json!({"type": "result", "credential": credential})).map_err(|_| {
        (
            "OperationError",
            "Could not return the authentication response.",
        )
    })
}

#[tokio::main]
async fn main() {
    // No request, credential, QR payload or backend error is written to logs.
    let mut bytes = Vec::new();
    let parsed = io::stdin()
        .take(MAX_REQUEST + 1)
        .read_to_end(&mut bytes)
        .ok()
        .filter(|_| bytes.len() as u64 <= MAX_REQUEST)
        .and_then(|_| serde_json::from_slice::<Request>(&bytes).ok());
    let result = match parsed {
        Some(input) => {
            let timeout = input
                .public_key
                .get("timeout")
                .and_then(Value::as_u64)
                .unwrap_or(120_000)
                .clamp(1, 120_000);
            match tokio::time::timeout(Duration::from_millis(timeout), authenticate(input)).await {
                Ok(result) => result,
                Err(_) => Err((
                    "NotAllowedError",
                    "Phone authentication timed out. Try again.",
                )),
            }
        }
        None => Err(("TypeError", "Invalid authentication request.")),
    };
    if let Err((name, message)) = result {
        let _ = emit(json!({"type": "error", "name": name, "message": message}));
        std::process::exit(1);
    }
}

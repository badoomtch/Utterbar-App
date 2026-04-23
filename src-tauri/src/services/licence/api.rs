//! HTTP client for the four licence endpoints (docs/LICENCE_APP_HANDOFF §4).
//!
//! One shared reqwest client per process — configured with rustls + HTTP/2 to
//! match the website's ALPN, and a User-Agent of
//! `Utterbar/{version} (windows; x86_64)` so server logs can correlate by build.
//!
//! Error mapping: the server returns `{ error, message, meta }` on 4xx/5xx. We
//! translate the `error` slug into an `OrliqError::Licence { kind }` so the UI
//! can switch without parsing human text.

use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::util::result::OrliqError;

/// Production and sandbox share the same origin; Stripe keys + `STRIPE_ENABLED`
/// env var gate the behaviour on the server side. Override with
/// `UTTERBAR_LICENCE_BASE_URL` for local Laravel dev pointing at a tunnel.
const DEFAULT_BASE_URL: &str = "https://utterbar.com";

/// Envelope returned on all non-2xx server responses (§4 preamble). `meta` is
/// kept on the struct so future server-side diagnostics (e.g. device list on a
/// 409 device_limit_reached) deserialize cleanly even before the UI consumes
/// them; we surface `error` + `message` into `OrliqError::Licence` today.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: String,
    message: String,
    #[serde(default)]
    meta: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct FetchRequest<'a> {
    key: &'a str,
}

/// Response shape for `/api/licence/fetch`. `issued_at` comes back as an
/// ISO-8601 string; we don't surface it to the UI today because the signed
/// blob's payload carries the authoritative epoch ms (and we'd rather the UI
/// trust what actually got signed). Kept here so the wire contract round-trips.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct FetchResponse {
    /// Base64-encoded 96-byte signed blob.
    pub signed_blob: String,
    pub licence_id: u64,
    pub sku: String,
    pub issued_at: String,
    pub max_devices: u16,
}

#[derive(Debug, Serialize)]
struct ActivateRequest<'a> {
    key: &'a str,
    fingerprint: &'a str,
    device_name: &'a str,
    platform: &'a str,
    app_version: &'a str,
}

/// `remaining_slots` isn't consumed in the activate path today — the UI uses
/// `devices.len()` vs `max_devices` to render the slot indicator. Keeping the
/// field so a future "activate a 2nd device" UI flow that shows remaining slots
/// in a toast can wire to it without a round-trip through the contract.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ActivateResponse {
    pub activation_token: String,
    pub devices: Vec<ApiDevice>,
    pub remaining_slots: i32,
}

#[derive(Debug, Deserialize)]
pub struct ApiDevice {
    pub fingerprint: String,
    pub device_name: String,
    pub first_seen: String,
    pub this_device: bool,
}

#[derive(Debug, Serialize)]
struct DeactivateRequest<'a> {
    key: &'a str,
    fingerprint: &'a str,
}

/// Mirrors the server contract's `{ ok, remaining_slots }`. Neither field is
/// read by the current deactivate flow — local clear always proceeds — but
/// `remaining_slots` will feed a confirmation toast once the UI lands.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DeactivateResponse {
    pub ok: bool,
    pub remaining_slots: i32,
}

#[derive(Debug, Serialize)]
struct HeartbeatRequest<'a> {
    key: &'a str,
    fingerprint: &'a str,
    app_version: &'a str,
}

/// Heartbeat is fire-and-forget from the UI's perspective — we only care
/// whether the HTTP call succeeded (→ bump the next-due timer) or came back as
/// `revoked` (→ clear cache). The `status` body string is kept so a future
/// diagnostics surface can show it verbatim.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    pub status: String,
}

pub struct LicenceApi {
    client: Client,
    base_url: String,
}

impl LicenceApi {
    pub fn new() -> Self {
        let user_agent = format!("Utterbar/{} (windows; x86_64)", env!("CARGO_PKG_VERSION"));

        // Modest timeouts: users are usually on residential broadband, but a
        // flaky hotel wifi shouldn't pin the paste flow forever. Connect is
        // short so a wrong DNS / blocked host surfaces quickly.
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            // The stack is already on `rustls-tls` (see Cargo.toml); reqwest
            // honours that via the feature flag. No extra builder call needed.
            .build()
            .expect("reqwest client builds with the compile-time TLS backend");

        let base_url = std::env::var("UTTERBAR_LICENCE_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        Self { client, base_url }
    }

    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    pub async fn fetch(&self, display_key: &str) -> Result<FetchResponse, OrliqError> {
        let url = format!("{}/api/licence/fetch", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&FetchRequest { key: display_key })
            .send()
            .await
            .map_err(network_err)?;
        parse_response(resp).await
    }

    pub async fn activate(
        &self,
        display_key: &str,
        fingerprint: &str,
        device_name: &str,
    ) -> Result<ActivateResponse, OrliqError> {
        let url = format!("{}/api/licence/activate", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&ActivateRequest {
                key: display_key,
                fingerprint,
                device_name,
                platform: "windows",
                app_version: env!("CARGO_PKG_VERSION"),
            })
            .send()
            .await
            .map_err(network_err)?;
        parse_response(resp).await
    }

    pub async fn deactivate(
        &self,
        display_key: &str,
        fingerprint: &str,
    ) -> Result<DeactivateResponse, OrliqError> {
        let url = format!("{}/api/licence/deactivate", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&DeactivateRequest {
                key: display_key,
                fingerprint,
            })
            .send()
            .await
            .map_err(network_err)?;
        parse_response(resp).await
    }

    pub async fn heartbeat(
        &self,
        display_key: &str,
        fingerprint: &str,
    ) -> Result<HeartbeatResponse, OrliqError> {
        let url = format!("{}/api/licence/heartbeat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&HeartbeatRequest {
                key: display_key,
                fingerprint,
                app_version: env!("CARGO_PKG_VERSION"),
            })
            .send()
            .await
            .map_err(network_err)?;
        parse_response(resp).await
    }
}

impl Default for LicenceApi {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify reqwest transport errors as `"network"` — the service layer
/// translates this into "silently retry next launch" for heartbeat and
/// "show inline error, don't cache" for activate.
fn network_err(e: reqwest::Error) -> OrliqError {
    OrliqError::Licence {
        code: "network".to_string(),
        message: e.to_string(),
    }
}

/// Decode a server response into `T` or an `OrliqError::Licence` whose `kind`
/// matches the server's `error` slug (404 → `"not_found"`, 410 →
/// `"revoked"`, 409 → `"device_limit"`, etc.).
async fn parse_response<T: for<'de> Deserialize<'de>>(
    resp: reqwest::Response,
) -> Result<T, OrliqError> {
    let status = resp.status();
    if status.is_success() {
        return resp.json::<T>().await.map_err(|e| OrliqError::Licence {
            code: "internal".to_string(),
            message: format!("server returned invalid JSON: {e}"),
        });
    }

    // Attempt to parse the structured envelope first; fall back to status code
    // if the server returned something unexpected (HTML error page, etc.).
    let body_bytes = resp.bytes().await.unwrap_or_default();
    if let Ok(env) = serde_json::from_slice::<ErrorEnvelope>(&body_bytes) {
        return Err(map_server_error(status, &env));
    }
    Err(OrliqError::Licence {
        code: map_status_kind(status).to_string(),
        message: format!(
            "server returned {} with non-JSON body ({} bytes)",
            status.as_u16(),
            body_bytes.len()
        ),
    })
}

fn map_server_error(status: StatusCode, env: &ErrorEnvelope) -> OrliqError {
    let kind = match env.error.as_str() {
        "licence_not_found" => "not_found",
        "licence_revoked" => "revoked",
        "device_limit_reached" => "device_limit",
        // Anything the server has a specific slug for but we haven't mapped yet
        // passes through as-is — UI can display the message.
        other if !other.is_empty() => other,
        _ => map_status_kind(status),
    };
    OrliqError::Licence {
        code: kind.to_string(),
        message: env.message.clone(),
    }
}

fn map_status_kind(status: StatusCode) -> &'static str {
    match status.as_u16() {
        404 => "not_found",
        410 => "revoked",
        409 => "device_limit",
        429 => "network", // rate-limit — treat as transient network class
        500..=599 => "network",
        _ => "internal",
    }
}

//! Release-email subscription commands.
//!
//! One verb today — `subscribe_to_release_updates` — which POSTs the user's
//! email to `https://utterbar.com/api/updates/subscribe`. The server replies
//! with `{ ok: true, message }` on success (either a freshly-sent confirmation
//! mail or an "already on the list" idempotent no-op) and we pass `message`
//! back to the UI verbatim to avoid bifurcating the "confirmed vs pending"
//! copy between client and server — the server owns the double-opt-in flow
//! via a mailed confirmation link and we deliberately don't try to mirror
//! that state locally.
//!
//! The reqwest client matches the licence module's shape: rustls, UA-stamped
//! `Utterbar/{version} (windows; x86_64)`, short connect timeout. We keep the
//! client one-shot (built per call) rather than wiring it into AppState
//! because subscription is rare — users opt in once and forget — and the
//! process-wide client would sit idle holding a TLS pool for no benefit.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::util::result::{CmdResult, OrliqError};

/// Base URL for the subscription endpoint. Overridable via env var so local
/// Laravel dev against a tunnel (cloudflared / ngrok) works without a rebuild.
const DEFAULT_BASE_URL: &str = "https://utterbar.com";

#[derive(Debug, Serialize)]
struct SubscribeRequest<'a> {
    email: &'a str,
    /// Server contract: `source` is optional but we always send `"app"` so the
    /// downstream list-curation UI can tell app-opt-ins from
    /// waitlist-migrations and website-form signups.
    source: &'a str,
}

/// Successful response envelope. The server always returns `{ ok: true,
/// message }` on 200 — we surface `message` verbatim to the UI (per the
/// handoff: "show the message verbatim; don't add UI state about confirmed
/// vs pending").
#[derive(Debug, Deserialize)]
struct SubscribeOkResponse {
    #[allow(dead_code)]
    ok: bool,
    message: String,
}

/// Error envelope for 4xx — mirrors the licence API's `{ error, message }`
/// shape. `429` is a rate-limit from the server's `throttle('5,10')` rule.
#[derive(Debug, Deserialize)]
struct SubscribeErrorResponse {
    #[allow(dead_code)]
    error: String,
    message: String,
}

/// Subscribe to release-announcement email.
///
/// Returns the server's human-readable `message` on success — the UI shows it
/// verbatim under the email input so the copy stays in one place (the server).
/// Errors are classified into `OrliqError` codes:
///   • `malformed`     — 422, client-side validation bounce
///   • `rate_limited`  — 429, server-side throttle
///   • `network`       — transport-level failure or 5xx
///   • `internal`      — unexpected body shape
#[tauri::command]
pub async fn subscribe_to_release_updates(email: String) -> CmdResult<String> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(OrliqError::Licence {
            code: "malformed".into(),
            message: "Email is required.".into(),
        });
    }

    let user_agent = format!("Utterbar/{} (windows; x86_64)", env!("CARGO_PKG_VERSION"));
    let client = Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| OrliqError::Licence {
            code: "internal".into(),
            message: format!("http client build failed: {e}"),
        })?;

    let base_url =
        std::env::var("UTTERBAR_LICENCE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let url = format!("{}/api/updates/subscribe", base_url);

    let resp = client
        .post(&url)
        .json(&SubscribeRequest {
            email: trimmed,
            source: "app",
        })
        .send()
        .await
        .map_err(|e| OrliqError::Licence {
            code: "network".into(),
            message: e.to_string(),
        })?;

    let status = resp.status();
    if status.is_success() {
        // 200 or 201 — both envelopes are `{ ok, message }`.
        let body: SubscribeOkResponse = resp.json().await.map_err(|e| OrliqError::Licence {
            code: "internal".into(),
            message: format!("server returned invalid JSON: {e}"),
        })?;
        return Ok(body.message);
    }

    // Try the structured envelope first; fall back to the status code so a
    // Laravel HTML error page (502 from Cloudflare, etc.) still surfaces
    // something the UI can show.
    let code = match status {
        StatusCode::UNPROCESSABLE_ENTITY => "malformed",
        StatusCode::TOO_MANY_REQUESTS => "rate_limited",
        s if s.is_server_error() => "network",
        _ => "internal",
    };
    let bytes = resp.bytes().await.unwrap_or_default();
    let message = serde_json::from_slice::<SubscribeErrorResponse>(&bytes)
        .map(|e| e.message)
        .unwrap_or_else(|_| match code {
            "malformed" => "That email doesn't look right.".into(),
            "rate_limited" => "Too many attempts — try again in a few minutes.".into(),
            "network" => "Couldn't reach the server. Check your connection and try again.".into(),
            _ => format!("Server returned {}.", status.as_u16()),
        });

    Err(OrliqError::Licence {
        code: code.into(),
        message,
    })
}

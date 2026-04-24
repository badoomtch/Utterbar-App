//! Pro-tier pricing lookup.
//!
//! One verb: `get_pro_price`. GETs `https://utterbar.com/api/pricing` and
//! returns a `{ amount_usd, period, currency, display, source }` struct. If
//! the server is unreachable, returns an offline fallback so the in-app
//! upgrade modal always has a price to show — a modal that says "couldn't
//! load pricing" reads like a broken feature and hurts conversion more than
//! a slightly stale number ever could. The fallback is `$29 one-time`, the
//! launch price at v0.6.
//!
//! Rationale for baking a fallback rather than erroring:
//!   • Offline users aren't edge cases — dictation is a headphones-on, on-
//!     a-plane use case by design. An upgrade modal that 404s on a
//!     transatlantic flight is worse than one that shows last-known price.
//!   • The real purchase happens on utterbar.com. The price shown in-app is
//!     a teaser; the checkout page is the source of truth. If the two
//!     disagree the user sees the right one at the point of sale.
//!   • A cache-miss/network-fail branch that returns nothing would force
//!     every call site to re-implement the fallback. One place is enough.
//!
//! The server endpoint is not strictly required to exist yet — if it returns
//! 404, we fall back. The command is forward-compatible with a future
//! Laravel route that returns `{ amount_usd, period, currency, display }`.

use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::util::result::CmdResult;

const DEFAULT_BASE_URL: &str = "https://utterbar.com";

/// Flat pricing descriptor. `display` is a pre-formatted string the UI can
/// render directly (`"$29"`, `"$29 / month"`, `"£24"`). Keeps currency +
/// period rendering on the server, where it's easier to iterate without an
/// app release. `source` lets the UI reveal whether we're showing live or
/// fallback copy — set to `"server"` on a successful fetch and `"fallback"`
/// on any failure path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProPrice {
    pub amount_usd: f64,
    pub currency: String,
    pub period: String, // "one_time" | "monthly" | "yearly"
    pub display: String,
    pub source: String, // "server" | "fallback"
}

impl ProPrice {
    /// Offline fallback. Keep this in sync with the live Stripe price
    /// on `utterbar.com/pricing` — currently £39.99 GBP one-time. If
    /// they diverge the checkout flow still wins (the server is the
    /// source of truth), but the in-app modal must never advertise a
    /// price lower than the real one. The `amount_usd` wire field is
    /// named for legacy reasons; it carries the amount in whichever
    /// currency `currency` specifies.
    fn fallback() -> Self {
        ProPrice {
            amount_usd: 39.99,
            currency: "GBP".into(),
            period: "one_time".into(),
            display: "£39.99".into(),
            source: "fallback".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ServerPricing {
    amount_usd: f64,
    #[serde(default = "default_currency")]
    currency: String,
    #[serde(default = "default_period")]
    period: String,
    #[serde(default)]
    display: Option<String>,
}

fn default_currency() -> String {
    "USD".into()
}
fn default_period() -> String {
    "one_time".into()
}

/// Fetch the Pro-tier price. Never fails — errors fold into the fallback.
#[tauri::command]
pub async fn get_pro_price() -> CmdResult<ProPrice> {
    let user_agent = format!("Utterbar/{} (windows; x86_64)", env!("CARGO_PKG_VERSION"));
    let client = match Client::builder()
        .user_agent(user_agent)
        .timeout(Duration::from_secs(6))
        .connect_timeout(Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(_) => return Ok(ProPrice::fallback()),
    };

    let base_url =
        std::env::var("UTTERBAR_LICENCE_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let url = format!("{}/api/pricing", base_url);

    // A single short-timeout attempt. Retries belong at the UI layer (user
    // can close & reopen the modal) — don't burn 30s of waitime here trying
    // to be clever about transient failures.
    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Ok(ProPrice::fallback()),
    };

    let body: ServerPricing = match resp.json().await {
        Ok(b) => b,
        Err(_) => return Ok(ProPrice::fallback()),
    };

    let display = body
        .display
        .unwrap_or_else(|| format_price(body.amount_usd, &body.currency, &body.period));

    Ok(ProPrice {
        amount_usd: body.amount_usd,
        currency: body.currency,
        period: body.period,
        display,
        source: "server".into(),
    })
}

/// Server-missing-display-field fallback formatter. Keeps the UI from having
/// to reconstruct "$29 / month" out of parts when the server speaks the
/// older contract.
fn format_price(amount: f64, currency: &str, period: &str) -> String {
    let symbol = match currency {
        "USD" => "$",
        "GBP" => "£",
        "EUR" => "€",
        _ => "",
    };
    let amount_s = if amount.fract() == 0.0 {
        format!("{}{:.0}", symbol, amount)
    } else {
        format!("{}{:.2}", symbol, amount)
    };
    match period {
        "monthly" => format!("{} / month", amount_s),
        "yearly" => format!("{} / year", amount_s),
        _ => amount_s,
    }
}

use serde::Serialize;
use thiserror::Error;

/// User-facing error envelope surfaced to the frontend.
#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OrliqError {
    #[error("{message}")]
    User { message: String },
    #[error("{message}")]
    Device { message: String },
    #[error("{message}")]
    Model { message: String },
    #[error("{message}")]
    Db { message: String },
    #[error("{message}")]
    Io { message: String },
    #[error("{message}")]
    Internal { message: String },
    /// Licence errors are split into a discriminator slug (`code`) so the UI
    /// can switch on the cause without parsing human text. Codes:
    /// `malformed`, `not_found`, `revoked`, `device_limit`, `network`,
    /// `verification_failed`, `already_activated`, `rate_limited`,
    /// `pro_required`, `internal`. The field is named `code` rather than
    /// `kind` because the enum's serde tag is itself `kind` — a colliding
    /// variant field makes the derive panic.
    ///
    /// `pro_required` is raised by backend gates (file transcription,
    /// fp16 install, custom modes, vocab/replacement caps, audio retention,
    /// re-transcribe) when the user is on the Free tier. The frontend
    /// surfaces a `<ProNudge>` next to the disabled control rather than
    /// relying on this error — the error exists so the backend can reject
    /// any bypass attempts and so the UI has a single slug to match on if
    /// it needs to react.
    #[error("{message}")]
    Licence { code: String, message: String },
}

pub type CmdResult<T> = Result<T, OrliqError>;

impl From<anyhow::Error> for OrliqError {
    fn from(err: anyhow::Error) -> Self {
        OrliqError::Internal {
            message: format!("{err:#}"),
        }
    }
}

impl From<rusqlite::Error> for OrliqError {
    fn from(err: rusqlite::Error) -> Self {
        OrliqError::Db {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for OrliqError {
    fn from(err: std::io::Error) -> Self {
        OrliqError::Io {
            message: err.to_string(),
        }
    }
}

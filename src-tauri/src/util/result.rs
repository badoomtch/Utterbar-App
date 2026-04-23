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

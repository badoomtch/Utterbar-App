//! Canonical data paths for Utterbar. Everything user-visible is under `%APPDATA%/Utterbar/`.

use std::path::PathBuf;

use anyhow::{anyhow, Result};

pub fn data_dir() -> Result<PathBuf> {
    let base = dirs::data_dir().ok_or_else(|| anyhow!("no data dir"))?;
    let path = base.join("Utterbar");
    ensure_dir(&path)?;
    Ok(path)
}

pub fn config_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("config");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn db_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("data");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn models_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("models");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn recordings_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("recordings");
    ensure_dir(&p)?;
    Ok(p)
}

pub fn logs_dir() -> Result<PathBuf> {
    let p = data_dir()?.join("logs");
    ensure_dir(&p)?;
    Ok(p)
}

fn ensure_dir(p: &std::path::Path) -> Result<()> {
    if !p.exists() {
        std::fs::create_dir_all(p)?;
    }
    Ok(())
}

// Tiny inline replacement for `dirs` so we don't add another dep.
// %APPDATA% on Windows; XDG_DATA_HOME on Linux; ~/Library/Application Support on macOS.
mod dirs {
    use std::path::PathBuf;

    pub fn data_dir() -> Option<PathBuf> {
        #[cfg(windows)]
        {
            std::env::var_os("APPDATA").map(PathBuf::from)
        }
        #[cfg(target_os = "macos")]
        {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::env::var_os("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        }
    }
}

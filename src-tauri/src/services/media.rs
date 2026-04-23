//! Media auto-pause: detect any system media sessions that are currently playing,
//! remember them, and pause them while the user is dictating. On `resume_paused`, only
//! the sessions we paused get played back — we never start something that wasn't already
//! playing.
//!
//! Implemented on top of SMTC (Windows System Media Transport Controls), which is the
//! same API Windows itself uses for the media overlay. Spotify, YouTube in
//! Edge/Chrome/Firefox, Apple Music, VLC, and any app that registers a media session all
//! participate automatically.

use std::sync::Mutex;

pub struct MediaService {
    // IDs of sessions we paused, so we know exactly which ones to resume. Stored as
    // SourceAppUserModelId strings since that's stable per app across our
    // pause/resume window.
    paused_ids: Mutex<Vec<String>>,
}

impl MediaService {
    pub fn new() -> Self {
        Self {
            paused_ids: Mutex::new(Vec::new()),
        }
    }

    /// Pause every session currently in the Playing state. Remembers which ones so
    /// `resume_paused` only touches those.
    pub fn pause_playing(&self) {
        #[cfg(windows)]
        {
            match pause_windows_sessions() {
                Ok(ids) => {
                    if let Ok(mut guard) = self.paused_ids.lock() {
                        *guard = ids;
                    }
                }
                Err(e) => tracing::warn!(?e, "SMTC pause failed"),
            }
        }
    }

    /// Resume only the sessions that we paused earlier.
    pub fn resume_paused(&self) {
        #[cfg(windows)]
        {
            let ids = {
                let Ok(mut guard) = self.paused_ids.lock() else {
                    return;
                };
                std::mem::take(&mut *guard)
            };
            if ids.is_empty() {
                return;
            }
            if let Err(e) = resume_windows_sessions(&ids) {
                tracing::warn!(?e, "SMTC resume failed");
            }
        }
    }
}

#[cfg(windows)]
fn pause_windows_sessions() -> anyhow::Result<Vec<String>> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Mgr,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };

    let manager = Mgr::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;
    let mut paused = Vec::new();

    for i in 0..sessions.Size()? {
        let Ok(session) = sessions.GetAt(i) else { continue };
        let Ok(playback_info) = session.GetPlaybackInfo() else { continue };
        let Ok(status) = playback_info.PlaybackStatus() else { continue };
        if status != Status::Playing {
            continue;
        }
        // Only touch sessions that actually expose a pause command — skips broken /
        // half-registered sources.
        let can_pause = playback_info
            .Controls()
            .and_then(|c| c.IsPauseEnabled())
            .unwrap_or(false);
        if !can_pause {
            continue;
        }

        let Ok(id_hstring) = session.SourceAppUserModelId() else { continue };
        let id = id_hstring.to_string();

        match session.TryPauseAsync() {
            Ok(op) => {
                let ok = op.get().unwrap_or(false);
                if ok {
                    paused.push(id);
                }
            }
            Err(e) => tracing::debug!(?e, id = %id, "TryPauseAsync failed"),
        }
    }
    Ok(paused)
}

#[cfg(windows)]
fn resume_windows_sessions(ids: &[String]) -> anyhow::Result<()> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Mgr;

    let manager = Mgr::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;

    for i in 0..sessions.Size()? {
        let Ok(session) = sessions.GetAt(i) else { continue };
        let Ok(id_hstring) = session.SourceAppUserModelId() else { continue };
        let id = id_hstring.to_string();
        if !ids.iter().any(|x| x == &id) {
            continue;
        }
        if let Err(e) = session.TryPlayAsync().map(|op| op.get().unwrap_or(false)) {
            tracing::debug!(?e, id = %id, "TryPlayAsync failed");
        }
    }
    Ok(())
}

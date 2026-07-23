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

#[derive(Debug, Default)]
pub struct PauseReport {
    pub sessions_seen: usize,
    pub playing_sessions: usize,
    pub pause_enabled_sessions: usize,
    pub paused_sessions: usize,
}

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
    pub fn pause_playing(&self) -> PauseReport {
        #[cfg(windows)]
        {
            match pause_windows_sessions() {
                Ok((ids, report)) => {
                    if let Ok(mut guard) = self.paused_ids.lock() {
                        *guard = ids;
                    }
                    tracing::info!(
                        sessions_seen = report.sessions_seen,
                        playing_sessions = report.playing_sessions,
                        pause_enabled_sessions = report.pause_enabled_sessions,
                        paused_sessions = report.paused_sessions,
                        "media pause attempt completed"
                    );
                    report
                }
                Err(e) => {
                    // A failed query must not leave IDs from an earlier recording behind.
                    // Otherwise a later stop could restart media that this recording never paused.
                    if let Ok(mut guard) = self.paused_ids.lock() {
                        guard.clear();
                    }
                    tracing::warn!(?e, "media pause request failed");
                    PauseReport::default()
                }
            }
        }

        #[cfg(not(windows))]
        {
            PauseReport::default()
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
            match resume_windows_sessions(&ids) {
                Ok(resumed_sessions) => tracing::info!(
                    requested_sessions = ids.len(),
                    resumed_sessions,
                    "media resume attempt completed"
                ),
                Err(e) => tracing::warn!(?e, "media resume request failed"),
            }
        }
    }
}

#[cfg(windows)]
fn pause_windows_sessions() -> anyhow::Result<(Vec<String>, PauseReport)> {
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Mgr,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };

    let manager = Mgr::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;
    let mut paused = Vec::new();
    let mut report = PauseReport {
        sessions_seen: sessions.Size()? as usize,
        ..PauseReport::default()
    };

    for i in 0..sessions.Size()? {
        let Ok(session) = sessions.GetAt(i) else {
            continue;
        };
        let Ok(playback_info) = session.GetPlaybackInfo() else {
            continue;
        };
        let Ok(status) = playback_info.PlaybackStatus() else {
            continue;
        };
        if status != Status::Playing {
            continue;
        }
        report.playing_sessions += 1;
        // Only touch sessions that actually expose a pause command — skips broken /
        // half-registered sources.
        let can_pause = playback_info
            .Controls()
            .and_then(|c| c.IsPauseEnabled())
            .unwrap_or(false);
        if !can_pause {
            continue;
        }
        report.pause_enabled_sessions += 1;

        let Ok(id_hstring) = session.SourceAppUserModelId() else {
            continue;
        };
        let id = id_hstring.to_string();

        match session.TryPauseAsync() {
            Ok(op) => {
                let ok = op.get().unwrap_or(false);
                if ok {
                    paused.push(id);
                    report.paused_sessions += 1;
                }
            }
            Err(e) => tracing::debug!(?e, id = %id, "TryPauseAsync failed"),
        }
    }
    Ok((paused, report))
}

#[cfg(windows)]
fn resume_windows_sessions(ids: &[String]) -> anyhow::Result<usize> {
    use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as Mgr;

    let manager = Mgr::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;
    let mut resumed = 0;

    for i in 0..sessions.Size()? {
        let Ok(session) = sessions.GetAt(i) else {
            continue;
        };
        let Ok(id_hstring) = session.SourceAppUserModelId() else {
            continue;
        };
        let id = id_hstring.to_string();
        if !ids.iter().any(|x| x == &id) {
            continue;
        }
        match session.TryPlayAsync() {
            Ok(op) => {
                if op.get().unwrap_or(false) {
                    resumed += 1;
                }
            }
            Err(e) => tracing::debug!(?e, id = %id, "TryPlayAsync failed"),
        }
    }
    Ok(resumed)
}

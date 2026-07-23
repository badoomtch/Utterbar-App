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

/// `pause_playing` and `resume_paused` run on Tokio blocking workers. Those
/// threads are not guaranteed to have a COM apartment, but Windows media-session
/// discovery is a WinRT call and needs one before it can reliably see browser and
/// desktop-player sessions.
#[cfg(windows)]
struct ComGuard {
    initialised_here: bool,
}

#[cfg(windows)]
impl ComGuard {
    fn new() -> Self {
        use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

        let hr = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let initialised_here = hr.is_ok();
        if !initialised_here {
            tracing::debug!(
                ?hr,
                "media session worker is using an existing COM apartment"
            );
        }
        Self { initialised_here }
    }
}

#[cfg(windows)]
impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialised_here {
            use windows::Win32::System::Com::CoUninitialize;

            unsafe { CoUninitialize() };
        }
    }
}

#[derive(Debug, Default)]
pub struct PauseReport {
    pub sessions_seen: usize,
    pub used_current_session_fallback: bool,
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
                        used_current_session_fallback = report.used_current_session_fallback,
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

    let _com = ComGuard::new();
    let manager = Mgr::RequestAsync()?.get()?;
    let sessions = manager.GetSessions()?;
    let mut paused = Vec::new();
    let mut candidates = Vec::with_capacity(sessions.Size()? as usize);
    for i in 0..sessions.Size()? {
        if let Ok(session) = sessions.GetAt(i) {
            candidates.push(session);
        }
    }

    // On some Windows builds the global list can be transiently empty even
    // though the system still exposes its selected playback session. That is
    // the session a hardware media key would control, so it is the right
    // single-session fallback for a browser video or music player.
    let used_current_session_fallback = if candidates.is_empty() {
        if let Ok(session) = manager.GetCurrentSession() {
            candidates.push(session);
            true
        } else {
            false
        }
    } else {
        false
    };
    let mut report = PauseReport {
        sessions_seen: candidates.len(),
        used_current_session_fallback,
        ..PauseReport::default()
    };

    for session in candidates {
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

    let _com = ComGuard::new();
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

#[cfg(test)]
mod tests {
    #[test]
    fn media_session_workers_initialise_com_before_querying_windows() {
        let source = include_str!("media.rs");
        for function in ["fn pause_windows_sessions()", "fn resume_windows_sessions("] {
            let start = source
                .find(function)
                .expect("media session helper must remain present");
            let body = &source[start..];
            let com_init = body
                .find("let _com = ComGuard::new();")
                .expect("media session helper must initialise COM");
            let request = body
                .find("Mgr::RequestAsync()")
                .expect("media session helper must request the Windows session manager");
            assert!(
                com_init < request,
                "COM must be initialised before requesting Windows media sessions"
            );
        }
        assert!(
            source.contains("manager.GetCurrentSession()"),
            "an empty session list must fall back to Windows' current media session"
        );
    }
}

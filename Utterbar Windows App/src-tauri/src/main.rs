#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_state;
mod commands;
mod db;
mod domain;
mod platform;
mod services;
mod util;

use std::sync::Arc;
use tauri::{Emitter, Manager, RunEvent};
use tracing::info;

use crate::app_state::AppState;
use crate::util::logging;
use crate::util::paths;

fn main() {
    // Surface full Rust backtraces into the panic hook below.
    std::env::set_var("RUST_BACKTRACE", "full");

    let data_dir = paths::data_dir().expect("data dir");
    logging::init(&data_dir);
    info!(version = env!("CARGO_PKG_VERSION"), "Utterbar starting");

    // Surface any panic from plugin init / builder / setup into the log instead of
    // letting the `panic=abort` profile swallow the process silently. We also write
    // the panic to a crash file so even a crash-on-exit after the tracing subscriber
    // drops still leaves a visible trace on disk.
    let crash_dir = data_dir.join("logs");
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(%info, "Utterbar panic");
        eprintln!("Utterbar panic: {info}");
        let path = crash_dir.join("crash.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            use std::io::Write;
            let _ = writeln!(
                f,
                "[{}] PANIC: {info}",
                time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default()
            );
            let _ = writeln!(
                f,
                "backtrace:\n{}",
                std::backtrace::Backtrace::force_capture()
            );
            let _ = f.flush();
        }
    }));

    // Windows SEH — sherpa-onnx / ORT can raise access violations that Rust's panic
    // hook never sees. Vectored-exception handler catches those before the default
    // WER abort and writes them to crash.log.
    #[cfg(windows)]
    install_seh_handler(data_dir.join("logs"));

    let state = Arc::new(AppState::bootstrap(&data_dir).expect("bootstrap AppState"));
    info!("AppState bootstrapped");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        // Deep link — `utterbar://licence/{TOKEN}` routes to the licence
        // activation flow. The scheme is registered by the NSIS installer;
        // the plugin dispatches incoming URLs into the running process.
        .plugin(tauri_plugin_deep_link::init())
        // Updater plugin needs a real pubkey to init; disabled until we sign releases.
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            // window
            commands::window::show_floating,
            commands::window::hide_floating,
            commands::window::show_main,
            commands::window::show_main_at,
            commands::window::show_onboarding,
            commands::window::set_floating_position,
            commands::window::clamp_floating_position,
            commands::window::reset_floating_position,
            commands::window::set_floating_passthrough,
            commands::window::get_cursor_pos,
            commands::window::finish_onboarding,
            // settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::settings::get_data_paths,
            commands::settings::get_default_conferencing_apps,
            commands::settings::validate_conferencing_app_name,
            // audio
            commands::audio::list_input_devices,
            commands::audio::start_meter,
            commands::audio::stop_meter,
            commands::audio::record_test_clip,
            // dictation
            commands::dictation::start_recording,
            commands::dictation::stop_recording,
            commands::dictation::cancel_recording,
            commands::dictation::get_phase,
            // modes
            commands::modes::list_modes,
            commands::modes::upsert_mode,
            commands::modes::delete_mode,
            commands::modes::duplicate_mode,
            commands::modes::set_active_mode,
            commands::modes::get_active_mode,
            commands::modes::current_focused_window,
            // models
            commands::models::list_installed_models,
            commands::models::list_available_models,
            commands::models::install_model,
            commands::models::remove_model,
            commands::models::model_status,
            commands::models::asr_runtime_status,
            // shortcuts
            commands::shortcuts::list_shortcuts,
            commands::shortcuts::bind_shortcut,
            commands::shortcuts::unbind_shortcut,
            commands::shortcuts::detect_conflicts,
            // vocabulary
            commands::vocabulary::list_vocabulary,
            commands::vocabulary::upsert_vocabulary,
            commands::vocabulary::delete_vocabulary,
            commands::vocabulary::list_replacements,
            commands::vocabulary::upsert_replacement,
            commands::vocabulary::delete_replacement,
            commands::vocabulary::import_vocabulary,
            commands::vocabulary::export_vocabulary,
            commands::vocabulary::open_vocab_quick_add,
            // history
            commands::history::list_history,
            commands::history::get_history_entry,
            commands::history::delete_history_entry,
            commands::history::reprocess_history,
            commands::history::list_history_apps,
            commands::history::export_history,
            // meetings (v0.15.0 speaker rename)
            commands::meetings::set_meeting_speaker_names,
            commands::meetings::clear_meeting_speaker_names,
            // files
            commands::files::enqueue_file,
            commands::files::enqueue_folder,
            commands::files::enqueue_path,
            commands::files::list_jobs,
            commands::files::cancel_job,
            commands::files::read_job_transcript,
            commands::files::reveal_job_in_folder,
            commands::files::delete_job,
            // licence
            commands::licence::licence_status,
            commands::licence::licence_activate,
            commands::licence::licence_deactivate,
            commands::licence::licence_is_managed,
            commands::licence::licence_export_request_file,
            commands::licence::licence_import_managed_file,
            // analytics
            commands::analytics::get_stats_summary,
            commands::analytics::get_stats_series,
            // updater — see `commands/updater.rs` module docstring for the
            // server-endpoint contract and why we rolled our own rather than
            // adopting `tauri-plugin-updater`.
            commands::updater::check_update,
            commands::updater::install_update,
            commands::updater::consume_post_update_marker,
            // release-email subscription
            commands::updates::subscribe_to_release_updates,
            // release notes for the in-app Updates page
            commands::release_notes::current_release_notes,
            // pricing lookup (for the in-app upgrade modal)
            commands::pricing::get_pro_price,
            // group-policy snapshot for the settings UI hints
            commands::policy::policy_snapshot,
            // diagnostics
            commands::diagnostics::get_diagnostics,
            commands::diagnostics::probe_gpu,
            commands::diagnostics::runtime_capabilities,
            commands::diagnostics::open_path,
            // backup
            commands::backup::export_user_data,
            commands::backup::import_user_data,
        ])
        .setup(move |app| {
            info!("running setup hook");
            if let Err(e) = services::tray::install(app.handle()) {
                tracing::warn!(?e, "tray install failed");
            }

            // Intercept [X] on the main window. When `close_to_tray` is on (the default),
            // hide instead of destroying so the next `show_main` invocation from the
            // floating bar or tray actually re-reveals it — without this, the first
            // close permanently removes the window handle and `get_webview_window("main")`
            // returns None on every subsequent click. When the user has explicitly
            // turned `close_to_tray` off in settings, let the close proceed: they
            // expect [X] to actually exit the app (tray icon remains as re-entry).
            if let Some(main_w) = app.get_webview_window("main") {
                let win_clone = main_w.clone();
                let state_close = state.clone();
                let app_handle_close = app.handle().clone();
                main_w.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let close_to_tray =
                            tauri::async_runtime::block_on(state_close.settings.snapshot())
                                .general
                                .close_to_tray;
                        if close_to_tray {
                            api.prevent_close();
                            let _ = win_clone.hide();
                        } else {
                            // Honour the user's "actually close" preference. Tray +
                            // floating windows would otherwise keep the process alive
                            // after [X] — not what they asked for. Explicit exit keeps
                            // behaviour predictable.
                            api.prevent_close();
                            app_handle_close.exit(0);
                        }
                    }
                });
            }
            if let Some(onb) = app.get_webview_window("onboarding") {
                let win_clone = onb.clone();
                onb.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            // Synchronous window choice so the user always sees something immediately,
            // independent of any async worker spinning up.
            let handle = app.handle().clone();
            let first_run = tauri::async_runtime::block_on(state.is_first_run());
            info!(first_run, "deciding initial window");
            if first_run {
                match handle.get_webview_window("onboarding") {
                    Some(w) => {
                        let _ = w.show();
                        let _ = w.set_focus();
                        info!("onboarding window shown");
                    }
                    None => tracing::error!("onboarding window not found"),
                }
                if let Some(w) = handle.get_webview_window("floating") {
                    let _ = w.hide();
                }
            } else {
                if let Some(w) = handle.get_webview_window("floating") {
                    // v0.6.7: restore the user's last good position
                    // BEFORE showing the window, so the bar appears
                    // directly where they left it with no visible
                    // snap from the default top-centre location.
                    // Validity (monitor still attached, within work
                    // area, etc.) is checked inside the function; a
                    // stale save falls through to the tauri.conf.json
                    // default coords.
                    if let Err(e) = commands::window::restore_floating_position(&w) {
                        tracing::warn!(?e, "restore_floating_position failed at startup");
                    }
                    let _ = w.show();
                    info!("floating bar shown");
                }
                if let Some(w) = handle.get_webview_window("onboarding") {
                    let _ = w.hide();
                }

                // Honour `general.start_minimised`: when the user has unchecked it, pop the
                // main window on launch so they land in Dashboard instead of having to
                // click the tray icon. Default remains minimised-to-tray so the app's
                // out-of-box behaviour doesn't change.
                let snap = tauri::async_runtime::block_on(state.settings.snapshot());
                let start_minimised = snap.general.start_minimised;

                // Repair-on-launch for HKCU\\...\\Run. The toggle
                // in Settings → General writes settings.json AND
                // the registry, but a stale registry value can
                // outlive the binary it points at (third-party
                // cleaners, manual reinstall to a different path,
                // a portable copy followed by a fresh install).
                // Re-asserting on every launch heals the entry
                // without making the user toggle off and back on.
                // Repair is a no-op when the registry already
                // matches the intended state. Failures are logged,
                // never propagated — startup must not fail because
                // a registry write failed.
                #[cfg(windows)]
                crate::platform::windows::startup::repair_launch_on_login(
                    snap.general.launch_on_login,
                );

                // Post-migration prompt. When the v0.8.0 v2 retirement
                // migration has cleared `default_asr_model_id`, the user
                // is now past onboarding (otherwise we'd be in the
                // onboarding window above) but has no working model.
                // Pop the main window straight to Settings → Models so
                // they see the picker instead of silently dictating
                // against the stub provider. Only fires when there's
                // something to fix; otherwise the start-minimised
                // preference wins.
                let no_default_model = snap.models.default_asr_model_id.is_none();

                // Post-update relaunch. When the previous run installed
                // an update and exited, NSIS re-launches utterbar.exe.
                // The updater drops a `pending_relaunch.json` marker
                // before exiting; if we see the marker on this launch
                // and it matches our running version, the user expects
                // to see the app come back visibly — not silently into
                // the tray. We peek (don't consume) so the frontend's
                // `consume_post_update_marker` still owns the toast
                // contract. Override `start_minimised` for this single
                // launch only; ordinary launch-at-login behaviour is
                // unchanged.
                let post_update_force_show =
                    crate::commands::updater::peek_pending_relaunch_for_force_show();

                if !start_minimised || no_default_model || post_update_force_show {
                    if let Some(w) = handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                        if no_default_model {
                            // Fire the navigate event so MainShell lands
                            // on the Models settings page on this open.
                            // Same pattern the floating bar uses for
                            // History / Settings shortcuts.
                            let _ = handle.emit(
                                crate::domain::events::EVT_MAIN_NAVIGATE,
                                "/settings/models",
                            );
                            info!("main window shown -> /settings/models (no default model after v2 retirement)");
                        } else if post_update_force_show {
                            info!("main window shown (post-update relaunch overrode start_minimised)");
                        } else {
                            info!("main window shown (start_minimised=false)");
                        }
                    }
                }
            }

            // Background: attach the state handle, arm hotkeys, start the file worker,
            // populate the tray submenu. None of these block the visible UI.
            let state_bg = state.clone();
            let handle_bg = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                state_bg.attach_app_handle(handle_bg.clone()).await;
                if let Err(e) = services::hotkeys::arm_from_settings(&handle_bg, &state_bg).await {
                    tracing::warn!(?e, "failed to arm global hotkeys");
                }
                state_bg
                    .files
                    .clone()
                    .start_worker(state_bg.clone(), handle_bg.clone())
                    .await;
                if let Err(e) = services::tray::rebuild(&handle_bg, &state_bg).await {
                    tracing::warn!(?e, "initial tray rebuild failed");
                }

                // Floating-bar watchdog. Polls every 2s and repairs the pill if the
                // system has hidden or minimised it behind our back — Win+D, Win+M,
                // shell-level SC_MINIMIZE, UAC dim, and virtual-desktop churn can all
                // leave the window invisible despite `minimizable: false`. We only
                // repair when the user has `bar_visibility == Always`; other modes
                // legitimately hide the pill between phases so we mustn't fight them.
                services::window_watchdog::spawn(handle_bg.clone(), state_bg.clone());

                // Per-app auto-activation watcher. Polls the foreground
                // window every 750ms and switches the active mode to
                // match the user's auto-activation rules. Idle-only —
                // never changes mode while the user is recording or
                // mid-transcribe. See services/auto_activation.rs for
                // the full design rationale.
                services::auto_activation::spawn(handle_bg.clone(), state_bg.clone());

                // Idle-unload sweeper for the ASR model. Honours
                // `models.warm_duration_secs`: 0 unloads immediately after each
                // transcription (handled inline in the orchestrator), > 0 keeps
                // the model warm for N seconds and this task evicts it.
                state_bg.inference.clone().spawn_idle_sweeper();

                // Background prewarm of the default ASR model when the user
                // has set `warm_duration_secs == u32::MAX` ("keep loaded
                // forever"). Without this, the first dictation after launch
                // pays the full native sherpa-onnx cold-load cost (~0.3–4 s
                // depending on model + provider + disk cache). The prewarm
                // task is fire-and-forget; failures are logged and do not
                // affect startup. See `services::inference::prewarm` for the
                // full rationale, locking-safety contract, and pure decision
                // helper.
                let prewarm_snap = state_bg.settings.snapshot().await;
                if let Some(model_id) =
                    services::inference::prewarm::should_prewarm_default(&prewarm_snap)
                {
                    services::inference::prewarm::spawn(state_bg.inference.clone(), model_id);
                } else {
                    tracing::info!(
                        warm_duration_secs = prewarm_snap.models.warm_duration_secs,
                        default_asr_model_id = ?prewarm_snap.models.default_asr_model_id,
                        "asr_prewarm: skipped"
                    );
                }

                // Licence launch check: reads the cached blob, verifies
                // offline, emits `licence_changed`. The heartbeat (if the
                // window has elapsed) runs on its own spawn inside the
                // service so it never blocks UI.
                state_bg
                    .licence
                    .clone()
                    .launch_check(handle_bg.clone())
                    .await;

                info!("background services ready");
            });

            // Deep-link routing. Tauri v2's plugin dispatches the URL via a
            // callback — we parse `utterbar://licence/{TOKEN}`, normalise the
            // token through the keyfmt rules, and kick the activate flow.
            // Any error is surfaced via the `licence_changed` event; the
            // settings page listens and shows the error inline.
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let handle_dl = app.handle().clone();
                let state_dl = state.clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        if url.scheme() != "utterbar" {
                            continue;
                        }
                        // Path form: `/licence/{TOKEN}` — url.path() includes the
                        // leading slash on web URLs but Tauri parses custom
                        // schemes as opaque, so try both.
                        let host = url.host_str().unwrap_or("");
                        let path = url.path();
                        let token = if host == "licence" {
                            path.trim_start_matches('/')
                        } else if path.starts_with("/licence/") {
                            path.trim_start_matches("/licence/")
                        } else {
                            continue;
                        };
                        if token.is_empty() {
                            continue;
                        }
                        let svc = state_dl.licence.clone();
                        let handle_inner = handle_dl.clone();
                        let key = token.to_string();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = svc.activate(handle_inner, &key).await {
                                tracing::warn!(error = ?e, "deep-link activation failed");
                            }
                        });
                    }
                });
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error building tauri app")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { .. } = event {
                info!("Utterbar exiting");
            }
        });
}

#[cfg(windows)]
fn install_seh_handler(log_dir: std::path::PathBuf) {
    use std::io::Write;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HANDLE, HMODULE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_WRITE, FILE_SHARE_NONE, OPEN_ALWAYS,
    };
    use windows::Win32::System::Diagnostics::Debug::{
        AddVectoredExceptionHandler, MiniDumpNormal, MiniDumpWithDataSegs, MiniDumpWithHandleData,
        MiniDumpWithProcessThreadData, MiniDumpWithThreadInfo, MiniDumpWriteDump,
        EXCEPTION_POINTERS, MINIDUMP_EXCEPTION_INFORMATION, MINIDUMP_TYPE,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, GetCurrentProcessId};

    static LOG_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    let _ = LOG_DIR.set(log_dir);

    // Reentry guard. If the dump-writing path itself crashes (unlikely but
    // it's our-own-process writing to disk during an exception), we don't
    // want to recursively spawn more dumps. AtomicBool set true on first
    // entry, never cleared; second entry bails straight to CONTINUE_SEARCH.
    static IN_HANDLER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        // EXCEPTION_CONTINUE_SEARCH = 0 — we only LOG + write the dump,
        // then let Windows' normal exception path (WER, terminate) run.
        if info.is_null() {
            return 0;
        }
        let rec = (*info).ExceptionRecord;
        if rec.is_null() {
            return 0;
        }
        let code = (*rec).ExceptionCode.0 as u32;
        let addr = (*rec).ExceptionAddress as usize;

        // Ignore C++ exception codes and other benign noise — only log real faults.
        // 0xC0000005 = access violation, 0xC0000094 = div-by-zero, 0xC0000409 = stack
        // buffer overrun. 0xE06D7363 is the MSVC C++ throw marker — skip it.
        const MS_CPP_EH: u32 = 0xE06D7363;
        if code == MS_CPP_EH {
            return 0;
        }
        // Also skip the benign debugger-name / RPC-server-unavailable noise
        // seen in earlier crash.log entries — they fire during normal
        // startup and aren't faults.
        const DBG_THREAD_NAME: u32 = 0x406D1388;
        const RPC_SERVER_UNAVAILABLE: u32 = 0x000006BA;
        if code == DBG_THREAD_NAME || code == RPC_SERVER_UNAVAILABLE {
            return 0;
        }

        // Reentry guard — CAS this to true. If we lost the race, bail out.
        if IN_HANDLER
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .is_err()
        {
            return 0;
        }

        // Timestamp for both the log line and the dump filename, so a user
        // can pair them up.
        let ts = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        // RFC3339 has colons which Windows filesystems don't like — replace.
        let ts_safe = ts.replace(':', "-");
        let pid = GetCurrentProcessId();

        if let Some(dir) = LOG_DIR.get() {
            let _ = std::fs::create_dir_all(dir);

            // Log line first (cheap, always works).
            let dump_name = format!("crash-{ts_safe}-pid{pid}.dmp");
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("crash.log"))
            {
                let _ = writeln!(
                    f,
                    "[{ts}] SEH exception code=0x{code:08X} address=0x{addr:016X} dump={dump_name}"
                );
                let _ = f.flush();
            }

            // MiniDumpWriteDump requires:
            //   * HANDLE to a newly-created file, writable, exclusive
            //   * GetCurrentProcess() + GetCurrentProcessId()
            //   * MINIDUMP_EXCEPTION_INFORMATION with the pointers we got
            //   * A dump type. We want enough to walk the stack and inspect
            //     heap regions the crashing thread was touching — Normal
            //     alone is too thin for heap-corruption analysis.
            let dump_path = dir.join(&dump_name);
            // Build a null-terminated UTF-16 path for CreateFileW.
            let wide: Vec<u16> = dump_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let pcw = PCWSTR::from_raw(wide.as_ptr());

            let file = CreateFileW(
                pcw,
                FILE_GENERIC_WRITE.0,
                FILE_SHARE_NONE,
                None,
                OPEN_ALWAYS,
                FILE_ATTRIBUTE_NORMAL,
                HANDLE::default(),
            );

            if let Ok(file_handle) = file {
                if !file_handle.is_invalid() {
                    let mut exc_info = MINIDUMP_EXCEPTION_INFORMATION {
                        ThreadId: windows::Win32::System::Threading::GetCurrentThreadId(),
                        ExceptionPointers: info,
                        ClientPointers: false.into(),
                    };

                    // Dump type: enough context for post-mortem without
                    // bloating to multi-GB heap dumps. WithDataSegs +
                    // ProcessThreadData + HandleData + ThreadInfo gives us
                    // thread stacks, module list, loaded heap metadata
                    // blocks, and per-thread TEB info — plenty to
                    // reconstruct the crashing call stack with full Rust
                    // symbolisation via cdb/windbg.
                    let dump_type = MINIDUMP_TYPE(
                        MiniDumpNormal.0
                            | MiniDumpWithDataSegs.0
                            | MiniDumpWithHandleData.0
                            | MiniDumpWithProcessThreadData.0
                            | MiniDumpWithThreadInfo.0,
                    );

                    let _ = MiniDumpWriteDump(
                        GetCurrentProcess(),
                        pid,
                        file_handle,
                        dump_type,
                        Some(&mut exc_info as *mut _),
                        None,
                        None,
                    );
                    let _ = windows::Win32::Foundation::CloseHandle(file_handle);
                }
            }
        }
        // Keep IN_HANDLER set — a reentrant fault while we're dying is
        // exactly when we want to NOT recurse. The process is about to
        // exit anyway.
        let _ = HMODULE::default(); // silence unused import on some feature sets
        0
    }

    unsafe {
        // first=1 → install at the front of the chain so we run before WER.
        AddVectoredExceptionHandler(1, Some(handler));
    }
}

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
use tauri::{Manager, RunEvent};
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
            commands::window::finish_onboarding,
            // settings
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            commands::settings::get_data_paths,
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
            // models
            commands::models::list_installed_models,
            commands::models::list_available_models,
            commands::models::install_model,
            commands::models::remove_model,
            commands::models::model_status,
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
            // history
            commands::history::list_history,
            commands::history::get_history_entry,
            commands::history::delete_history_entry,
            commands::history::reprocess_history,
            commands::history::export_history,
            // files
            commands::files::enqueue_file,
            commands::files::list_jobs,
            commands::files::cancel_job,
            // analytics
            commands::analytics::get_stats_summary,
            commands::analytics::get_stats_series,
            // updater (no-op stubs until signing is wired)
            commands::updater::check_update,
            commands::updater::install_update,
            // diagnostics
            commands::diagnostics::get_diagnostics,
            commands::diagnostics::probe_gpu,
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
                let start_minimised = tauri::async_runtime::block_on(state.settings.snapshot())
                    .general
                    .start_minimised;
                if !start_minimised {
                    if let Some(w) = handle.get_webview_window("main") {
                        let _ = w.show();
                        let _ = w.set_focus();
                        info!("main window shown (start_minimised=false)");
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

                // Idle-unload sweeper for the ASR model. Honours
                // `models.warm_duration_secs`: 0 unloads immediately after each
                // transcription (handled inline in the orchestrator), > 0 keeps
                // the model warm for N seconds and this task evicts it.
                state_bg.inference.clone().spawn_idle_sweeper();

                info!("background services ready");
            });

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
    use windows::Win32::System::Diagnostics::Debug::{
        AddVectoredExceptionHandler, EXCEPTION_POINTERS,
    };

    static LOG_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    let _ = LOG_DIR.set(log_dir);

    unsafe extern "system" fn handler(info: *mut EXCEPTION_POINTERS) -> i32 {
        // EXCEPTION_CONTINUE_SEARCH = 0 — we only LOG, then let Windows handle it.
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

        if let Some(dir) = LOG_DIR.get() {
            let _ = std::fs::create_dir_all(dir);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("crash.log"))
            {
                let ts = time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default();
                let _ = writeln!(
                    f,
                    "[{ts}] SEH exception code=0x{code:08X} address=0x{addr:016X}"
                );
                let _ = f.flush();
            }
        }
        0
    }

    unsafe {
        // first=1 → install at the front of the chain so we run before WER.
        AddVectoredExceptionHandler(1, Some(handler));
    }
}

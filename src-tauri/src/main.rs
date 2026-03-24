use std::{sync::Mutex, thread, time::Duration};

use state::app::AppState;
use tauri::{Emitter, Manager, WindowEvent};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_CONTROL, MOD_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

mod clipboard;
mod commands;
mod state;

const HOTKEY_ID: i32 = 1;
const CLIPBOARD_POLL_MS: u64 = 400;

fn start_clipboard_monitor<R: tauri::Runtime + 'static>(app: tauri::AppHandle<R>) {
    thread::spawn(move || loop {
        if let Some(text) = clipboard::reader::read_clipboard_text() {
            let maybe_item = {
                let app_state = app.state::<Mutex<AppState>>();
                let mut state = app_state.lock().expect("app state poisoned");

                match state.clipboard.capture_text(text, None) {
                    Ok(item) => item,
                    Err(error) => {
                        eprintln!("failed to persist clipboard item: {error}");
                        None
                    }
                }
            };

            if let Some(item) = maybe_item {
                let _ = app.emit("history-updated", &item);
            }
        }

        thread::sleep(Duration::from_millis(CLIPBOARD_POLL_MS));
    });
}

fn start_hotkey_listener<R: tauri::Runtime + 'static>(app: tauri::AppHandle<R>) {
    thread::spawn(move || unsafe {
        if RegisterHotKey(None, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT, 'V' as u32).is_err() {
            return;
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
            if msg.message == WM_HOTKEY && msg.wParam.0 as i32 == HOTKEY_ID {
                let handle = app.clone();
                let runner = app.clone();
                let _ = runner.run_on_main_thread(move || {
                    let _ = commands::window::show_main_window(&handle);
                });
            }
        }

        let _ = UnregisterHotKey(None, HOTKEY_ID);
    });
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::history::get_history,
            commands::history::read_current_clipboard,
            commands::history::paste_history_item,
            commands::history::seed_demo_history,
            commands::history::toggle_pin,
            commands::window::hide_window
        ])
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            match event {
                WindowEvent::Focused(false) => {
                    let _ = window.hide();
                }
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                _ => {}
            }
        })
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
            let db_path = app_data_dir.join("clipboard-history.sqlite3");
            let app_state = AppState::new(db_path)?;

            app.manage(Mutex::new(app_state));

            let handle = app.handle().clone();
            commands::window::position_main_window(&handle);

            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.hide();
            }

            start_clipboard_monitor(handle.clone());
            start_hotkey_listener(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running AI Paste");
}

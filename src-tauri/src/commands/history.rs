use std::{mem::size_of, ptr, sync::Mutex, thread, time::Duration};

use tauri::{State, WebviewWindow};
use windows::Win32::Foundation::{HANDLE, HGLOBAL};
use windows::Win32::System::{
    DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        SetClipboardData,
    },
    Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYBD_EVENT_FLAGS,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};

use crate::clipboard::{reader::read_clipboard_text, types::ClipboardItem};
use crate::state::app::AppState;

const CF_UNICODETEXT_FORMAT: u32 = 13;
const CLIPBOARD_WRITE_RETRIES: usize = 8;
const CLIPBOARD_WRITE_RETRY_DELAY_MS: u64 = 20;
const PASTE_AFTER_HIDE_DELAY_MS: u64 = 260;

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

fn read_clipboard_text_direct() -> Option<String> {
    unsafe {
        OpenClipboard(None).ok()?;
        let _guard = ClipboardGuard;

        IsClipboardFormatAvailable(CF_UNICODETEXT_FORMAT).ok()?;
        let handle = GetClipboardData(CF_UNICODETEXT_FORMAT).ok()?;
        let memory = HGLOBAL(handle.0);
        let ptr = GlobalLock(memory) as *const u16;
        if ptr.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let text = String::from_utf16_lossy(slice);
        let _ = GlobalUnlock(memory);

        Some(text)
    }
}

fn write_clipboard_text(content: &str) -> Result<(), String> {
    let utf16: Vec<u16> = content.encode_utf16().chain(std::iter::once(0)).collect();
    let bytes_len = utf16.len() * size_of::<u16>();

    for _ in 0..CLIPBOARD_WRITE_RETRIES {
        let result = unsafe {
            OpenClipboard(None)
                .map_err(|error| error.to_string())
                .and_then(|_| {
                    let _guard = ClipboardGuard;
                    EmptyClipboard().map_err(|error| error.to_string())?;

                    let memory = GlobalAlloc(GMEM_MOVEABLE, bytes_len).map_err(|error| error.to_string())?;
                    let ptr = GlobalLock(memory) as *mut u8;
                    if ptr.is_null() {
                        return Err("failed to lock global memory for clipboard write".to_string());
                    }

                    ptr::copy_nonoverlapping(utf16.as_ptr() as *const u8, ptr, bytes_len);
                    let _ = GlobalUnlock(memory);
                    SetClipboardData(CF_UNICODETEXT_FORMAT, HANDLE(memory.0)).map_err(|error| error.to_string())?;
                    Ok(())
                })
        };

        if result.is_ok() {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(CLIPBOARD_WRITE_RETRY_DELAY_MS));
    }

    Err("failed to write text into Windows clipboard".to_string())
}

fn press_ctrl_v() {
    unsafe {
        let inputs = [
            keyboard_input(VK_CONTROL, KEYBD_EVENT_FLAGS(0)),
            keyboard_input(VK_V, KEYBD_EVENT_FLAGS(0)),
            keyboard_input(VK_V, KEYEVENTF_KEYUP),
            keyboard_input(VK_CONTROL, KEYEVENTF_KEYUP),
        ];
        let _ = SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

fn keyboard_input(key: VIRTUAL_KEY, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: flags,
                ..Default::default()
            },
        },
    }
}

#[tauri::command]
pub fn get_history(state: State<'_, Mutex<AppState>>, query: Option<String>) -> Vec<ClipboardItem> {
    let mut state = state.lock().expect("app state poisoned");

    if let Some(text) = read_clipboard_text() {
        if let Err(error) = state.clipboard.capture_text(text, None) {
            eprintln!("failed to persist clipboard item from get_history: {error}");
        }
    }

    state.clipboard.history(query.as_deref())
}

#[tauri::command]
pub fn read_current_clipboard() -> Option<String> {
    read_clipboard_text_direct()
}

#[tauri::command]
pub fn seed_demo_history(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut state = state.lock().expect("app state poisoned");
    state.clipboard.seed_demo_entries()
}

#[tauri::command]
pub fn toggle_pin(
    state: State<'_, Mutex<AppState>>,
    item_id: String,
) -> Result<Option<ClipboardItem>, String> {
    let mut state = state.lock().expect("app state poisoned");
    state.clipboard.toggle_pin(&item_id)
}

#[tauri::command]
pub fn paste_history_item(
    state: State<'_, Mutex<AppState>>,
    window: WebviewWindow,
    item_id: String,
) -> Result<(), String> {
    let content = {
        let state = state.lock().expect("app state poisoned");
        state
            .clipboard
            .find_by_id(&item_id)
            .map(|item| item.content.clone())
            .ok_or_else(|| "history item not found".to_string())?
    };

    write_clipboard_text(&content)?;
    window.hide().map_err(|error| error.to_string())?;
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(PASTE_AFTER_HIDE_DELAY_MS));
        press_ctrl_v();
    });
    Ok(())
}

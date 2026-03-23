use std::{thread, time::Duration};

use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::{
    DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard},
    Memory::{GlobalLock, GlobalUnlock},
};

const CF_UNICODETEXT_FORMAT: u32 = 13;
const CLIPBOARD_OPEN_RETRIES: usize = 8;
const CLIPBOARD_OPEN_RETRY_DELAY_MS: u64 = 20;

struct ClipboardGuard;

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

pub fn read_clipboard_text() -> Option<String> {
    for _ in 0..CLIPBOARD_OPEN_RETRIES {
        if let Some(text) = try_read_clipboard_text_once() {
            return Some(text);
        }

        thread::sleep(Duration::from_millis(CLIPBOARD_OPEN_RETRY_DELAY_MS));
    }

    None
}

fn try_read_clipboard_text_once() -> Option<String> {
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

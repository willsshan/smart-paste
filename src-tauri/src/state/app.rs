use super::overlay::OverlayState;
use crate::clipboard::manager::ClipboardManager;

#[derive(Debug)]
pub struct AppState {
    pub clipboard: ClipboardManager,
    pub overlay: OverlayState,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clipboard: ClipboardManager::new(),
            overlay: OverlayState::default(),
        }
    }
}


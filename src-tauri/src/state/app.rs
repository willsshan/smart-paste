use std::path::PathBuf;

use super::overlay::OverlayState;
use crate::clipboard::manager::ClipboardManager;

pub struct AppState {
    pub clipboard: ClipboardManager,
    pub overlay: OverlayState,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        Ok(Self {
            clipboard: ClipboardManager::new(db_path)?,
            overlay: OverlayState::default(),
        })
    }
}

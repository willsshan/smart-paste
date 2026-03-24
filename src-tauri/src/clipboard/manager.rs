use std::{collections::VecDeque, path::PathBuf};

use super::{storage::ClipboardStore, types::ClipboardItem};

const DEFAULT_HISTORY_LIMIT: usize = 100;

pub struct ClipboardManager {
    history: VecDeque<ClipboardItem>,
    history_limit: usize,
    last_captured_text: Option<String>,
    store: ClipboardStore,
}

impl ClipboardManager {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let history_limit = DEFAULT_HISTORY_LIMIT;
        let store = ClipboardStore::new(db_path, history_limit)?;
        let history = VecDeque::from(store.load_history()?);
        let last_captured_text = history.front().map(|item| item.content.clone());

        Ok(Self {
            history,
            history_limit,
            last_captured_text,
            store,
        })
    }

    pub fn history(&self, query: Option<&str>) -> Vec<ClipboardItem> {
        let mut items: Vec<_> = self.history.iter().cloned().collect();

        items.sort_by(|left, right| {
            right
                .is_pinned
                .cmp(&left.is_pinned)
                .then_with(|| right.created_at.cmp(&left.created_at))
        });

        if let Some(query) = query {
            let query = query.trim().to_lowercase();
            if !query.is_empty() {
                items.retain(|item| {
                    let haystack = format!(
                        "{} {} {}",
                        item.preview,
                        item.content,
                        item.source_app.clone().unwrap_or_default()
                    )
                    .to_lowercase();

                    haystack.contains(&query)
                });
            }
        }

        items
    }

    pub fn capture_text(
        &mut self,
        content: impl Into<String>,
        source_app: Option<String>,
    ) -> Result<Option<ClipboardItem>, String> {
        let content = content.into().trim().to_string();
        if content.is_empty() {
            return Ok(None);
        }

        if self.last_captured_text.as_deref() == Some(content.as_str()) {
            return Ok(None);
        }

        self.last_captured_text = Some(content.clone());
        let item = self.push_text(content, source_app)?;
        Ok(Some(item))
    }

    pub fn push_text(
        &mut self,
        content: impl Into<String>,
        source_app: Option<String>,
    ) -> Result<ClipboardItem, String> {
        let item = ClipboardItem::new_text(content, source_app);

        self.store.insert_item(&item)?;
        self.dedup_same_text(&item.content);
        self.history.push_front(item.clone());
        self.truncate();

        Ok(item)
    }

    pub fn find_by_id(&self, item_id: &str) -> Option<ClipboardItem> {
        self.history.iter().find(|item| item.id == item_id).cloned()
    }

    pub fn toggle_pin(&mut self, item_id: &str) -> Result<Option<ClipboardItem>, String> {
        let item = match self.history.iter_mut().find(|item| item.id == item_id) {
            Some(item) => item,
            None => return Ok(None),
        };

        item.is_pinned = !item.is_pinned;
        self.store.update_pin(item_id, item.is_pinned)?;
        Ok(Some(item.clone()))
    }

    pub fn seed_demo_entries(&mut self) -> Result<(), String> {
        let samples = [
            ("cargo tauri dev", Some("Windows Terminal".to_string())),
            ("https://tauri.app/start/", Some("Microsoft Edge".to_string())),
            (
                "Rust + Tauri MVP milestone list",
                Some("Visual Studio Code".to_string()),
            ),
            ("给项目补充 Ctrl+Shift+V 历史面板", Some("Notion".to_string())),
        ];

        for (content, app) in samples {
            let _ = self.capture_text(content, app)?;
        }

        Ok(())
    }

    fn dedup_same_text(&mut self, content: &str) {
        self.history.retain(|item| item.content != content);
    }

    fn truncate(&mut self) {
        while self.history.len() > self.history_limit {
            self.history.pop_back();
        }
    }
}

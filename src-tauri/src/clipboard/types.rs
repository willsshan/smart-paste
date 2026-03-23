use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardItemKind {
    Text,
    Image,
    File,
    Html,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClipboardItem {
    pub id: String,
    pub kind: ClipboardItemKind,
    pub content: String,
    pub preview: String,
    pub source_app: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_pinned: bool,
}

impl ClipboardItem {
    pub fn new_text(content: impl Into<String>, source_app: Option<String>) -> Self {
        let content = content.into();
        let preview = build_preview(&content);

        Self {
            id: Uuid::new_v4().to_string(),
            kind: ClipboardItemKind::Text,
            content,
            preview,
            source_app,
            created_at: Utc::now(),
            is_pinned: false,
        }
    }
}

fn build_preview(content: &str) -> String {
    let sanitized = content.replace('\n', " ").replace('\r', "");
    let preview: String = sanitized.chars().take(64).collect();

    if sanitized.chars().count() > 64 {
      format!("{preview}...")
    } else {
      preview
    }
}


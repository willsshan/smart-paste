use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};

use super::types::ClipboardItem;

pub struct ClipboardStore {
    connection: Connection,
    history_limit: usize,
}

impl ClipboardStore {
    pub fn new(db_path: PathBuf, history_limit: usize) -> Result<Self, String> {
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        let connection = Connection::open(db_path).map_err(|error| error.to_string())?;
        let store = Self {
            connection,
            history_limit,
        };
        store.initialize()?;
        Ok(store)
    }

    pub fn load_history(&self) -> Result<Vec<ClipboardItem>, String> {
        let mut statement = self
            .connection
            .prepare(
                "
                SELECT id, kind, content, preview, source_app, created_at, is_pinned
                FROM clipboard_history
                ORDER BY created_at DESC
                LIMIT ?1
                ",
            )
            .map_err(|error| error.to_string())?;

        let mut rows = statement
            .query(params![self.history_limit as i64])
            .map_err(|error| error.to_string())?;

        let mut items = Vec::new();
        while let Some(row) = rows.next().map_err(|error| error.to_string())? {
            let kind: String = row.get(1).map_err(|error| error.to_string())?;
            let created_at_raw: String = row.get(5).map_err(|error| error.to_string())?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_raw)
                .map_err(|error| error.to_string())?
                .with_timezone(&Utc);

            let item = ClipboardItem::from_persisted(
                row.get(0).map_err(|error| error.to_string())?,
                &kind,
                row.get(2).map_err(|error| error.to_string())?,
                row.get(3).map_err(|error| error.to_string())?,
                row.get(4).map_err(|error| error.to_string())?,
                created_at,
                row.get::<_, i64>(6).map_err(|error| error.to_string())? != 0,
            )
            .ok_or_else(|| format!("unsupported clipboard item kind in database: {kind}"))?;

            items.push(item);
        }

        Ok(items)
    }

    pub fn insert_item(&mut self, item: &ClipboardItem) -> Result<(), String> {
        let transaction = self
            .connection
            .transaction()
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "DELETE FROM clipboard_history WHERE content = ?1",
                params![item.content],
            )
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "
                INSERT INTO clipboard_history (
                    id,
                    kind,
                    content,
                    preview,
                    source_app,
                    created_at,
                    is_pinned
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                params![
                    item.id,
                    item.kind.as_db_value(),
                    item.content,
                    item.preview,
                    item.source_app,
                    item.created_at.to_rfc3339(),
                    if item.is_pinned { 1 } else { 0 },
                ],
            )
            .map_err(|error| error.to_string())?;

        transaction
            .execute(
                "
                DELETE FROM clipboard_history
                WHERE id NOT IN (
                    SELECT id
                    FROM clipboard_history
                    ORDER BY created_at DESC
                    LIMIT ?1
                )
                ",
                params![self.history_limit as i64],
            )
            .map_err(|error| error.to_string())?;

        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn update_pin(&self, item_id: &str, is_pinned: bool) -> Result<(), String> {
        self.connection
            .execute(
                "UPDATE clipboard_history SET is_pinned = ?1 WHERE id = ?2",
                params![if is_pinned { 1 } else { 0 }, item_id],
            )
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    fn initialize(&self) -> Result<(), String> {
        self.connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS clipboard_history (
                    id TEXT PRIMARY KEY,
                    kind TEXT NOT NULL,
                    content TEXT NOT NULL,
                    preview TEXT NOT NULL,
                    source_app TEXT,
                    created_at TEXT NOT NULL,
                    is_pinned INTEGER NOT NULL DEFAULT 0
                );

                CREATE INDEX IF NOT EXISTS idx_clipboard_history_created_at
                    ON clipboard_history(created_at DESC);

                CREATE INDEX IF NOT EXISTS idx_clipboard_history_is_pinned
                    ON clipboard_history(is_pinned DESC, created_at DESC);
                ",
            )
            .map_err(|error| error.to_string())
    }
}

use crate::managers::transcription::TranscriptionOutcome;
use crate::storage::migrations::{MigrationRunner, HISTORY_MIGRATIONS};

fn recording_file_name(file_name: &str) -> Result<&Path> {
    let candidate = Path::new(file_name);
    if candidate.file_name().and_then(|name| name.to_str()) != Some(file_name) {
        return Err(anyhow!("invalid recording file name"));
    }
    Ok(candidate)
}
use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};
use log::{debug, error, info};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_specta::Event;

const HISTORY_COLUMNS: &str = "id, file_name, timestamp, saved, title, transcription_text,
    raw_transcript, post_processed_text, post_process_prompt, post_process_requested,
    model_id, requested_language, effective_language, detected_language,
    audio_duration_ms, transcription_ms, transcription_status, transcription_error,
    application_id, window_title";

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct PaginatedHistory {
    pub entries: Vec<HistoryEntry>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type, tauri_specta::Event)]
#[serde(tag = "action")]
pub enum HistoryUpdatePayload {
    #[serde(rename = "added")]
    Added { entry: HistoryEntry },
    #[serde(rename = "updated")]
    Updated { entry: HistoryEntry },
    #[serde(rename = "deleted")]
    Deleted { id: i64 },
    #[serde(rename = "cleared")]
    Cleared,
    #[serde(rename = "toggled")]
    Toggled { id: i64 },
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct HistoryEntry {
    pub id: i64,
    pub file_name: String,
    pub timestamp: i64,
    pub saved: bool,
    pub title: String,
    pub transcription_text: String,
    pub raw_transcript: String,
    pub post_processed_text: Option<String>,
    pub post_process_prompt: Option<String>,
    pub post_process_requested: bool,
    pub model_id: Option<String>,
    pub requested_language: Option<String>,
    pub effective_language: Option<String>,
    pub detected_language: Option<String>,
    pub audio_duration_ms: Option<i64>,
    pub transcription_ms: Option<i64>,
    pub transcription_status: String,
    pub transcription_error: Option<String>,
    pub application_id: Option<String>,
    pub window_title: Option<String>,
}

pub struct HistoryManager {
    app_handle: AppHandle,
    recordings_dir: PathBuf,
    db_path: PathBuf,
}

impl HistoryManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        // Create recordings directory in app data dir
        let app_data_dir = crate::portable::app_data_dir(app_handle)?;
        let recordings_dir = app_data_dir.join("recordings");
        let db_path = app_data_dir.join("history.db");

        // Ensure recordings directory exists
        if !recordings_dir.exists() {
            fs::create_dir_all(&recordings_dir)?;
            debug!("Created recordings directory: {:?}", recordings_dir);
        }

        let manager = Self {
            app_handle: app_handle.clone(),
            recordings_dir,
            db_path,
        };

        // Initialize database and run migrations synchronously
        manager.init_database()?;
        let recovered = if crate::settings::get_history_storage_mode(app_handle)
            == crate::settings::HistoryStorageMode::NeverStore
        {
            manager.purge_transient_recordings()?;
            0
        } else {
            manager.recover_retryable_recordings()?
        };
        if recovered > 0 {
            info!(
                "Recovered {} finalized recording(s) without a history row",
                recovered
            );
        }

        Ok(manager)
    }

    fn init_database(&self) -> Result<()> {
        info!("Initializing database at {:?}", self.db_path);

        let mut conn = Connection::open(&self.db_path)?;

        // Preserve migration state from the legacy SQL plugin before the
        // reversible FreeFlow runner takes ownership of user_version.
        self.migrate_from_tauri_plugin_sql(&conn)?;

        let migrations = MigrationRunner::new(HISTORY_MIGRATIONS)?;

        // Get current version before migration
        let version_before: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
        debug!("Database version before migration: {}", version_before);

        // Apply any pending migrations
        migrations.migrate_to_latest(&mut conn)?;

        // Get version after migration
        let version_after: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if version_after > version_before {
            info!(
                "Database migrated from version {} to {}",
                version_before, version_after
            );
        } else {
            debug!("Database already at latest version {}", version_after);
        }

        Ok(())
    }

    /// Migrate from the legacy SQL plugin's tracking to FreeFlow's runner.
    /// The old plugin used a _sqlx_migrations table, while FreeFlow uses SQLite's
    /// user_version pragma. This function checks if the old system was in use
    /// and sets the user_version accordingly so migrations don't re-run.
    fn migrate_from_tauri_plugin_sql(&self, conn: &Connection) -> Result<()> {
        // Check if the old _sqlx_migrations table exists
        let has_sqlx_migrations: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='_sqlx_migrations'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !has_sqlx_migrations {
            return Ok(());
        }

        // Check current user_version
        let current_version: i32 =
            conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

        if current_version > 0 {
            // Already migrated to the FreeFlow runner.
            return Ok(());
        }

        // Get the highest version from the old migrations table
        let old_version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations WHERE success = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if old_version > 0 {
            info!(
                "Migrating legacy SQL state (version {}) to the FreeFlow runner",
                old_version
            );

            // Set user_version to match the old migration state
            conn.pragma_update(None, "user_version", old_version)?;

            // Optionally drop the old migrations table (keeping it doesn't hurt)
            // conn.execute("DROP TABLE IF EXISTS _sqlx_migrations", [])?;

            info!(
                "Migration tracking converted: user_version set to {}",
                old_version
            );
        }

        Ok(())
    }

    fn get_connection(&self) -> Result<Connection> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn map_history_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
        Ok(HistoryEntry {
            id: row.get("id")?,
            file_name: row.get("file_name")?,
            timestamp: row.get("timestamp")?,
            saved: row.get("saved")?,
            title: row.get("title")?,
            transcription_text: row.get("transcription_text")?,
            raw_transcript: row.get("raw_transcript")?,
            post_processed_text: row.get("post_processed_text")?,
            post_process_prompt: row.get("post_process_prompt")?,
            post_process_requested: row.get("post_process_requested")?,
            model_id: row.get("model_id")?,
            requested_language: row.get("requested_language")?,
            effective_language: row.get("effective_language")?,
            detected_language: row.get("detected_language")?,
            audio_duration_ms: row.get("audio_duration_ms")?,
            transcription_ms: row.get("transcription_ms")?,
            transcription_status: row.get("transcription_status")?,
            transcription_error: row.get("transcription_error")?,
            application_id: row.get("application_id")?,
            window_title: row.get("window_title")?,
        })
    }

    pub fn recordings_dir(&self) -> &std::path::Path {
        &self.recordings_dir
    }

    /// Reconcile captures that were atomically finalized before a crash but did
    /// not yet receive their retryable history row. Invalid, empty, and hidden
    /// in-progress files are never imported.
    fn recover_retryable_recordings(&self) -> Result<usize> {
        for entry in fs::read_dir(&self.recordings_dir)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with(".freeflow-capture-") && name.ends_with(".part") {
                if let Err(error) = fs::remove_file(&path) {
                    error!("Failed to remove interrupted capture {:?}: {}", path, error);
                }
            }
        }

        let conn = self.get_connection()?;
        let mut statement = conn.prepare("SELECT file_name FROM transcription_history")?;
        let referenced_names = statement
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<HashSet<_>>>()?;
        drop(statement);
        drop(conn);

        let candidates = crate::audio_toolkit::retryable_wav_candidates(
            &self.recordings_dir,
            &referenced_names,
        )?;
        let mut recovered = 0;
        for path in candidates {
            let Some(file_name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
            else {
                continue;
            };
            self.save_entry(file_name, String::new(), false, None, None)?;
            recovered += 1;
        }
        Ok(recovered)
    }

    /// Never-store may still use an empty retry row while ASR is running. If
    /// the process exits mid-operation, remove that row and its audio on the
    /// next launch instead of recovering content the user chose not to retain.
    fn purge_transient_recordings(&self) -> Result<()> {
        for entry in fs::read_dir(&self.recordings_dir)? {
            let path = entry?.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with(".freeflow-capture-") && name.ends_with(".part") {
                fs::remove_file(path)?;
            }
        }

        let entries = {
            let conn = self.get_connection()?;
            let mut statement = conn.prepare(
                "SELECT id, file_name FROM transcription_history
                 WHERE saved = 0 AND transcription_status != 'completed'",
            )?;
            let rows = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<Vec<(i64, String)>>>()?;
            rows
        };
        self.delete_entries_and_files(&entries)?;

        let referenced = {
            let conn = self.get_connection()?;
            let mut statement = conn.prepare("SELECT file_name FROM transcription_history")?;
            let names = statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<rusqlite::Result<HashSet<_>>>()?;
            names
        };
        for path in
            crate::audio_toolkit::retryable_wav_candidates(&self.recordings_dir, &referenced)?
        {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Save a new history entry to the database.
    /// The WAV file should already have been written to the recordings directory.
    pub fn save_entry(
        &self,
        file_name: String,
        transcription_text: String,
        post_process_requested: bool,
        post_processed_text: Option<String>,
        post_process_prompt: Option<String>,
    ) -> Result<HistoryEntry> {
        self.save_entry_with_context(
            file_name,
            transcription_text,
            post_process_requested,
            post_processed_text,
            post_process_prompt,
            None,
        )
    }

    pub fn save_entry_with_context(
        &self,
        file_name: String,
        transcription_text: String,
        post_process_requested: bool,
        post_processed_text: Option<String>,
        post_process_prompt: Option<String>,
        context: Option<&crate::contracts::PlatformContext>,
    ) -> Result<HistoryEntry> {
        let timestamp = Utc::now().timestamp();
        let title = self.format_timestamp_title(timestamp);
        let application_id = context.and_then(|value| value.application_id.clone());
        let window_title = context.and_then(|value| value.window_title.clone());

        let conn = self.get_connection()?;
        conn.execute(
            "INSERT INTO transcription_history (
                file_name,
                timestamp,
                saved,
                title,
                transcription_text,
                post_processed_text,
                post_process_prompt,
                post_process_requested,
                application_id,
                window_title
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                &file_name,
                timestamp,
                false,
                &title,
                &transcription_text,
                &post_processed_text,
                &post_process_prompt,
                post_process_requested,
                &application_id,
                &window_title,
            ],
        )?;

        let entry = HistoryEntry {
            id: conn.last_insert_rowid(),
            file_name,
            timestamp,
            saved: false,
            title,
            transcription_text,
            raw_transcript: String::new(),
            post_processed_text,
            post_process_prompt,
            post_process_requested,
            model_id: None,
            requested_language: None,
            effective_language: None,
            detected_language: None,
            audio_duration_ms: None,
            transcription_ms: None,
            transcription_status: "pending".to_string(),
            transcription_error: None,
            application_id,
            window_title,
        };

        debug!("Saved history entry with id {}", entry.id);

        self.cleanup_old_entries()?;

        // Emit typed event for real-time frontend updates
        if let Err(e) = (HistoryUpdatePayload::Added {
            entry: entry.clone(),
        })
        .emit(&self.app_handle)
        {
            error!("Failed to emit history-updated event: {}", e);
        }

        Ok(entry)
    }

    /// Update an existing history entry with new transcription results (used by retry).
    pub fn complete_transcription(
        &self,
        id: i64,
        outcome: &TranscriptionOutcome,
        post_processed_text: Option<String>,
        post_process_prompt: Option<String>,
    ) -> Result<HistoryEntry> {
        let conn = self.get_connection()?;
        let updated = conn.execute(
            "UPDATE transcription_history
             SET transcription_text = ?1,
                 raw_transcript = ?2,
                 post_processed_text = ?3,
                 post_process_prompt = ?4,
                 model_id = ?5,
                 requested_language = ?6,
                 effective_language = ?7,
                 detected_language = ?8,
                 audio_duration_ms = ?9,
                 transcription_ms = ?10,
                 transcription_status = 'completed',
                 transcription_error = NULL
             WHERE id = ?11",
            params![
                &outcome.text,
                &outcome.raw_text,
                post_processed_text,
                post_process_prompt,
                &outcome.model_id,
                &outcome.requested_language,
                &outcome.effective_language,
                &outcome.detected_language,
                i64::try_from(outcome.audio_duration_ms).unwrap_or(i64::MAX),
                i64::try_from(outcome.transcription_ms).unwrap_or(i64::MAX),
                id
            ],
        )?;

        if updated == 0 {
            return Err(anyhow!("History entry {} not found", id));
        }

        let entry = conn.query_row(
            &format!("SELECT {HISTORY_COLUMNS} FROM transcription_history WHERE id = ?1"),
            params![id],
            Self::map_history_entry,
        )?;

        debug!("Updated transcription for history entry {}", id);

        if let Err(e) = (HistoryUpdatePayload::Updated {
            entry: entry.clone(),
        })
        .emit(&self.app_handle)
        {
            error!("Failed to emit history-updated event: {}", e);
        }

        Ok(entry)
    }

    pub fn mark_transcription_failed(&self, id: i64, message: &str) -> Result<HistoryEntry> {
        let conn = self.get_connection()?;
        let updated = conn.execute(
            "UPDATE transcription_history
             SET transcription_status = 'failed', transcription_error = ?1
             WHERE id = ?2",
            params![message, id],
        )?;
        if updated == 0 {
            return Err(anyhow!("History entry {} not found", id));
        }
        let entry = conn.query_row(
            &format!("SELECT {HISTORY_COLUMNS} FROM transcription_history WHERE id = ?1"),
            params![id],
            Self::map_history_entry,
        )?;
        if let Err(error) = (HistoryUpdatePayload::Updated {
            entry: entry.clone(),
        })
        .emit(&self.app_handle)
        {
            error!("Failed to emit failed-history update: {}", error);
        }
        Ok(entry)
    }

    pub fn cleanup_old_entries(&self) -> Result<()> {
        let retention_period = crate::settings::get_recording_retention_period(&self.app_handle);

        match retention_period {
            crate::settings::RecordingRetentionPeriod::Never => {
                // Don't delete anything
                Ok(())
            }
            crate::settings::RecordingRetentionPeriod::PreserveLimit => {
                // Use the old count-based logic with history_limit
                let limit = crate::settings::get_history_limit(&self.app_handle);
                self.cleanup_by_count(limit)
            }
            _ => {
                // Use time-based logic
                self.cleanup_by_time(retention_period)
            }
        }
    }

    fn delete_entries_and_files(&self, entries: &[(i64, String)]) -> Result<usize> {
        let deleted =
            Self::delete_entries_and_files_at(&self.recordings_dir, &self.db_path, entries)?;
        for (id, _) in entries {
            if let Err(error) = (HistoryUpdatePayload::Deleted { id: *id }).emit(&self.app_handle) {
                error!("Failed to emit deleted-history update: {error}");
            }
        }
        Ok(deleted)
    }

    fn delete_entries_and_files_at(
        recordings_dir: &Path,
        db_path: &Path,
        entries: &[(i64, String)],
    ) -> Result<usize> {
        if entries.is_empty() {
            return Ok(0);
        }

        let paths = entries
            .iter()
            .map(|(_, file_name)| {
                recording_file_name(file_name).map(|name| recordings_dir.join(name))
            })
            .collect::<Result<Vec<_>>>()?;

        for ((_, file_name), file_path) in entries.iter().zip(&paths) {
            if file_path.exists() {
                fs::remove_file(file_path)
                    .map_err(|error| anyhow!("failed to delete recording {file_name}: {error}"))?;
            }
            if file_path.exists() {
                return Err(anyhow!(
                    "recording deletion could not be verified: {file_name}"
                ));
            }
        }

        let mut conn = Connection::open(db_path)?;
        let transaction = conn.transaction()?;
        for (id, _) in entries {
            transaction.execute(
                "DELETE FROM transcription_history WHERE id = ?1",
                params![id],
            )?;
        }
        transaction.commit()?;

        Ok(entries.len())
    }

    fn cleanup_by_count(&self, limit: usize) -> Result<()> {
        let conn = self.get_connection()?;
        let entries_to_delete = Self::entries_past_count(&conn, limit)?;

        if !entries_to_delete.is_empty() {
            let deleted_count = self.delete_entries_and_files(&entries_to_delete)?;

            if deleted_count > 0 {
                debug!("Cleaned up {} old history entries by count", deleted_count);
            }
        }

        Ok(())
    }

    fn entries_past_count(conn: &Connection, limit: usize) -> Result<Vec<(i64, String)>> {
        let mut statement = conn.prepare(
            "SELECT id, file_name FROM transcription_history
             WHERE saved = 0 ORDER BY timestamp DESC LIMIT -1 OFFSET ?1",
        )?;
        let rows = statement
            .query_map(params![i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    fn cleanup_by_time(
        &self,
        retention_period: crate::settings::RecordingRetentionPeriod,
    ) -> Result<()> {
        let conn = self.get_connection()?;

        // Calculate cutoff timestamp (current time minus retention period)
        let now = Utc::now().timestamp();
        let cutoff_timestamp = Self::retention_cutoff(now, retention_period)
            .ok_or_else(|| anyhow!("retention period has no time cutoff"))?;

        let entries_to_delete = Self::entries_before(&conn, cutoff_timestamp)?;
        let deleted_count = self.delete_entries_and_files(&entries_to_delete)?;

        if deleted_count > 0 {
            debug!(
                "Cleaned up {} old history entries based on retention period",
                deleted_count
            );
        }

        Ok(())
    }

    fn entries_before(conn: &Connection, cutoff: i64) -> Result<Vec<(i64, String)>> {
        let mut statement = conn.prepare(
            "SELECT id, file_name FROM transcription_history
             WHERE saved = 0 AND timestamp < ?1 ORDER BY timestamp",
        )?;
        let rows = statement
            .query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(rows)
    }

    fn retention_cutoff(
        now: i64,
        retention_period: crate::settings::RecordingRetentionPeriod,
    ) -> Option<i64> {
        match retention_period {
            crate::settings::RecordingRetentionPeriod::Days3 => Some(now - (3 * 24 * 60 * 60)),
            crate::settings::RecordingRetentionPeriod::Weeks2 => Some(now - (2 * 7 * 24 * 60 * 60)),
            crate::settings::RecordingRetentionPeriod::Months3 => {
                Some(now - (3 * 30 * 24 * 60 * 60))
            }
            crate::settings::RecordingRetentionPeriod::Never
            | crate::settings::RecordingRetentionPeriod::PreserveLimit => None,
        }
    }

    pub async fn get_history_entries(
        &self,
        cursor: Option<i64>,
        limit: Option<usize>,
        query: Option<String>,
    ) -> Result<PaginatedHistory> {
        let conn = self.get_connection()?;
        Self::get_history_entries_with_conn(&conn, cursor, limit, query)
    }

    fn get_history_entries_with_conn(
        conn: &Connection,
        cursor: Option<i64>,
        limit: Option<usize>,
        query: Option<String>,
    ) -> Result<PaginatedHistory> {
        let limit = limit.map(|l| l.min(100));
        let fetch_count = limit.map_or(-1, |value| (value + 1) as i64);
        let query = query.unwrap_or_default().trim().to_string();
        let sql = format!(
            "SELECT {HISTORY_COLUMNS} FROM transcription_history
             WHERE (?1 IS NULL OR id < ?1)
               AND (?2 = ''
                 OR instr(lower(transcription_text), lower(?2)) > 0
                 OR instr(lower(raw_transcript), lower(?2)) > 0
                 OR instr(lower(COALESCE(post_processed_text, '')), lower(?2)) > 0
                 OR instr(lower(COALESCE(application_id, '')), lower(?2)) > 0
                 OR instr(lower(COALESCE(window_title, '')), lower(?2)) > 0
                 OR instr(lower(COALESCE(model_id, '')), lower(?2)) > 0)
             ORDER BY id DESC LIMIT ?3"
        );
        let mut stmt = conn.prepare(&sql)?;
        let mut entries = stmt
            .query_map(params![cursor, query, fetch_count], Self::map_history_entry)?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let has_more = limit.is_some_and(|lim| entries.len() > lim);
        if has_more {
            entries.pop();
        }

        Ok(PaginatedHistory { entries, has_more })
    }

    #[cfg(test)]
    fn get_latest_entry_with_conn(conn: &Connection) -> Result<Option<HistoryEntry>> {
        let mut stmt = conn.prepare(&format!(
            "SELECT {HISTORY_COLUMNS} FROM transcription_history ORDER BY timestamp DESC LIMIT 1"
        ))?;

        let entry = stmt.query_row([], Self::map_history_entry).optional()?;
        Ok(entry)
    }

    /// Get the latest entry with non-empty transcription text.
    pub fn get_latest_completed_entry(&self) -> Result<Option<HistoryEntry>> {
        let conn = self.get_connection()?;
        Self::get_latest_completed_entry_with_conn(&conn)
    }

    fn get_latest_completed_entry_with_conn(conn: &Connection) -> Result<Option<HistoryEntry>> {
        let mut stmt = conn.prepare(
            &format!("SELECT {HISTORY_COLUMNS} FROM transcription_history WHERE transcription_text != '' ORDER BY timestamp DESC LIMIT 1"),
        )?;

        let entry = stmt.query_row([], Self::map_history_entry).optional()?;
        Ok(entry)
    }

    pub async fn toggle_saved_status(&self, id: i64) -> Result<()> {
        let conn = self.get_connection()?;

        // Get current saved status
        let current_saved: bool = conn.query_row(
            "SELECT saved FROM transcription_history WHERE id = ?1",
            params![id],
            |row| row.get("saved"),
        )?;

        let new_saved = !current_saved;

        conn.execute(
            "UPDATE transcription_history SET saved = ?1 WHERE id = ?2",
            params![new_saved, id],
        )?;

        debug!("Toggled saved status for entry {}: {}", id, new_saved);

        // Emit history updated event
        if let Err(e) = (HistoryUpdatePayload::Toggled { id }).emit(&self.app_handle) {
            error!("Failed to emit history-updated event: {}", e);
        }

        Ok(())
    }

    pub fn get_audio_file_path(&self, file_name: &str) -> Result<PathBuf> {
        Ok(self.recordings_dir.join(recording_file_name(file_name)?))
    }

    pub fn delete_untracked_recording(&self, file_name: &str) -> Result<()> {
        let path = self.get_audio_file_path(file_name)?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        if path.exists() {
            return Err(anyhow!(
                "recording deletion could not be verified: {file_name}"
            ));
        }
        Ok(())
    }

    pub async fn get_entry_by_id(&self, id: i64) -> Result<Option<HistoryEntry>> {
        let conn = self.get_connection()?;
        let mut stmt = conn.prepare(&format!(
            "SELECT {HISTORY_COLUMNS} FROM transcription_history WHERE id = ?1"
        ))?;

        let entry = stmt.query_row([id], Self::map_history_entry).optional()?;

        Ok(entry)
    }

    pub async fn delete_entry(&self, id: i64) -> Result<()> {
        let entry = self
            .get_entry_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("History entry {id} not found"))?;
        self.delete_entries_and_files(&[(id, entry.file_name)])?;

        debug!("Deleted history entry with id: {}", id);

        Ok(())
    }

    pub async fn clear_history(&self) -> Result<usize> {
        let entries = {
            let conn = self.get_connection()?;
            let mut statement = conn.prepare("SELECT id, file_name FROM transcription_history")?;
            let rows = statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<rusqlite::Result<Vec<(i64, String)>>>()?;
            rows
        };
        let deleted = self.delete_entries_and_files(&entries)?;
        if let Err(error) = HistoryUpdatePayload::Cleared.emit(&self.app_handle) {
            error!("Failed to emit cleared-history update: {error}");
        }
        Ok(deleted)
    }

    fn format_timestamp_title(&self, timestamp: i64) -> String {
        if let Some(utc_datetime) = DateTime::from_timestamp(timestamp, 0) {
            // Convert UTC to local timezone
            let local_datetime = utc_datetime.with_timezone(&Local);
            local_datetime.format("%B %e, %Y - %l:%M%p").to_string()
        } else {
            format!("Recording {}", timestamp)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        conn.execute_batch(
            "CREATE TABLE transcription_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_name TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                saved BOOLEAN NOT NULL DEFAULT 0,
                title TEXT NOT NULL,
                transcription_text TEXT NOT NULL,
                raw_transcript TEXT NOT NULL DEFAULT '',
                post_processed_text TEXT,
                post_process_prompt TEXT,
                post_process_requested BOOLEAN NOT NULL DEFAULT 0,
                model_id TEXT,
                requested_language TEXT,
                effective_language TEXT,
                detected_language TEXT,
                audio_duration_ms INTEGER,
                transcription_ms INTEGER,
                transcription_status TEXT NOT NULL DEFAULT 'pending',
                transcription_error TEXT,
                application_id TEXT,
                window_title TEXT
            );",
        )
        .expect("create transcription_history table");
        conn
    }

    fn insert_entry(conn: &Connection, timestamp: i64, text: &str, post_processed: Option<&str>) {
        conn.execute(
            "INSERT INTO transcription_history (
                file_name,
                timestamp,
                saved,
                title,
                transcription_text,
                raw_transcript,
                post_processed_text,
                post_process_prompt,
                post_process_requested
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                format!("freeflow-{}.wav", timestamp),
                timestamp,
                false,
                format!("Recording {}", timestamp),
                text,
                text,
                post_processed,
                Option::<String>::None,
                false,
            ],
        )
        .expect("insert history entry");
    }

    #[test]
    fn get_latest_entry_returns_none_when_empty() {
        let conn = setup_conn();
        let entry = HistoryManager::get_latest_entry_with_conn(&conn).expect("fetch latest entry");
        assert!(entry.is_none());
    }

    #[test]
    fn recording_file_name_rejects_path_traversal() {
        assert!(recording_file_name("../settings_store.json").is_err());
        assert!(recording_file_name("nested/recording.wav").is_err());
        assert_eq!(
            recording_file_name("freeflow-123.wav")
                .expect("plain recording file name")
                .to_string_lossy(),
            "freeflow-123.wav"
        );
    }

    #[test]
    fn get_latest_entry_returns_newest_entry() {
        let conn = setup_conn();
        insert_entry(&conn, 100, "first", None);
        insert_entry(&conn, 200, "second", Some("processed"));

        let entry = HistoryManager::get_latest_entry_with_conn(&conn)
            .expect("fetch latest entry")
            .expect("entry exists");

        assert_eq!(entry.timestamp, 200);
        assert_eq!(entry.transcription_text, "second");
        assert_eq!(entry.post_processed_text.as_deref(), Some("processed"));
    }

    #[test]
    fn get_latest_completed_entry_skips_empty_entries() {
        let conn = setup_conn();
        insert_entry(&conn, 100, "completed", None);
        insert_entry(&conn, 200, "", None);

        let entry = HistoryManager::get_latest_completed_entry_with_conn(&conn)
            .expect("fetch latest completed entry")
            .expect("completed entry exists");

        assert_eq!(entry.timestamp, 100);
        assert_eq!(entry.transcription_text, "completed");
    }

    #[test]
    fn search_matches_text_and_application_metadata_with_pagination() {
        let conn = setup_conn();
        insert_entry(&conn, 100, "quarterly review", None);
        insert_entry(
            &conn,
            200,
            "ship the release",
            Some("Ship the release today"),
        );
        conn.execute(
            "UPDATE transcription_history
             SET application_id = 'Notes.exe', window_title = 'Launch checklist'
             WHERE timestamp = 200",
            [],
        )
        .expect("add application metadata");

        let text = HistoryManager::get_history_entries_with_conn(
            &conn,
            None,
            Some(1),
            Some("release".into()),
        )
        .expect("search final text");
        assert_eq!(text.entries.len(), 1);
        assert_eq!(text.entries[0].timestamp, 200);
        assert!(!text.has_more);

        let app = HistoryManager::get_history_entries_with_conn(
            &conn,
            None,
            Some(30),
            Some("notes.exe".into()),
        )
        .expect("search application metadata");
        assert_eq!(app.entries.len(), 1);
        assert_eq!(
            app.entries[0].window_title.as_deref(),
            Some("Launch checklist")
        );
    }

    #[test]
    fn retention_cutoffs_cover_every_time_setting() {
        use crate::settings::RecordingRetentionPeriod;

        let now = 10_000_000;
        assert_eq!(
            HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::Days3),
            Some(now - 3 * 24 * 60 * 60)
        );
        assert_eq!(
            HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::Weeks2),
            Some(now - 2 * 7 * 24 * 60 * 60)
        );
        assert_eq!(
            HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::Months3),
            Some(now - 3 * 30 * 24 * 60 * 60)
        );
        assert_eq!(
            HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::Never),
            None
        );
        assert_eq!(
            HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::PreserveLimit),
            None
        );
    }

    #[test]
    fn every_retention_policy_selects_only_expired_unsaved_entries() {
        use crate::settings::RecordingRetentionPeriod;

        let conn = setup_conn();
        let now = 10_000_000;
        insert_entry(&conn, now - 60, "new", None);
        insert_entry(&conn, now - 4 * 24 * 60 * 60, "four days", None);
        insert_entry(&conn, now - 15 * 24 * 60 * 60, "fifteen days", None);
        insert_entry(&conn, now - 91 * 24 * 60 * 60, "ninety one days", None);
        insert_entry(&conn, now - 365 * 24 * 60 * 60, "saved", None);
        conn.execute(
            "UPDATE transcription_history SET saved = 1 WHERE transcription_text = 'saved'",
            [],
        )
        .expect("save protected entry");

        let by_count = HistoryManager::entries_past_count(&conn, 2).expect("count retention");
        assert_eq!(by_count.len(), 2);
        assert!(by_count.iter().all(|(id, _)| *id == 3 || *id == 4));

        for (period, expected) in [
            (RecordingRetentionPeriod::Days3, 3),
            (RecordingRetentionPeriod::Weeks2, 2),
            (RecordingRetentionPeriod::Months3, 1),
        ] {
            let cutoff = HistoryManager::retention_cutoff(now, period).expect("time cutoff");
            let expired = HistoryManager::entries_before(&conn, cutoff).expect("time retention");
            assert_eq!(expired.len(), expected);
            assert!(expired.iter().all(|(id, _)| *id != 5));
        }

        assert!(HistoryManager::retention_cutoff(now, RecordingRetentionPeriod::Never).is_none());
    }

    #[test]
    fn app_layer_delete_removes_database_row_and_audio_irreversibly() {
        let directory = tempfile::tempdir().expect("temporary history root");
        let recordings = directory.path().join("recordings");
        fs::create_dir(&recordings).expect("create recordings directory");
        let database = directory.path().join("history.db");
        let conn = Connection::open(&database).expect("open history database");
        conn.execute_batch(
            "CREATE TABLE transcription_history (
                id INTEGER PRIMARY KEY, file_name TEXT NOT NULL
            );
            INSERT INTO transcription_history (id, file_name)
            VALUES (7, 'freeflow-7.wav');",
        )
        .expect("seed history database");
        fs::write(recordings.join("freeflow-7.wav"), b"audio").expect("seed audio");
        drop(conn);

        assert_eq!(
            HistoryManager::delete_entries_and_files_at(
                &recordings,
                &database,
                &[(7, "freeflow-7.wav".into())],
            )
            .expect("delete entry and audio"),
            1
        );
        assert!(!recordings.join("freeflow-7.wav").exists());
        let conn = Connection::open(database).expect("reopen history database");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transcription_history", [], |row| {
                row.get(0)
            })
            .expect("count history rows");
        assert_eq!(count, 0);
    }

    #[test]
    fn deletion_rejects_recording_path_traversal_before_touching_database() {
        let directory = tempfile::tempdir().expect("temporary history root");
        let recordings = directory.path().join("recordings");
        fs::create_dir(&recordings).expect("create recordings directory");
        let database = directory.path().join("history.db");
        let conn = Connection::open(&database).expect("open history database");
        conn.execute_batch(
            "CREATE TABLE transcription_history (
                id INTEGER PRIMARY KEY, file_name TEXT NOT NULL
            );
            INSERT INTO transcription_history (id, file_name)
            VALUES (9, '../settings_store.json');",
        )
        .expect("seed malicious row");
        drop(conn);

        assert!(HistoryManager::delete_entries_and_files_at(
            &recordings,
            &database,
            &[(9, "../settings_store.json".into())],
        )
        .is_err());
        let conn = Connection::open(database).expect("reopen history database");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM transcription_history", [], |row| {
                row.get(0)
            })
            .expect("count history rows");
        assert_eq!(count, 1);
    }
}

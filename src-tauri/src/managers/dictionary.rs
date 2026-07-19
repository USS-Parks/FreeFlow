use crate::storage::migrations::{MigrationRunner, PERSONALIZATION_MIGRATIONS};
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use regex::RegexBuilder;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::path::PathBuf;
use tauri::AppHandle;

pub const MAX_DICTIONARY_ENTRIES: usize = 5_000;
pub const MAX_SPOKEN_FORM_CHARS: usize = 200;
pub const MAX_REPLACEMENT_CHARS: usize = 4_000;
const CSV_HEADER: &str = "spoken_form,replacement,starred";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct DictionaryEntry {
    pub id: i64,
    pub spoken_form: String,
    pub replacement: String,
    pub starred: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum DictionarySort {
    Starred,
    Updated,
    SpokenForm,
}

#[derive(Clone, Debug, Serialize, Type)]
pub struct DictionaryEngineSupport {
    pub whisper_initial_prompt_only: bool,
    pub deterministic_replacement: bool,
}

pub struct DictionaryManager {
    db_path: PathBuf,
}

impl DictionaryManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let db_path = crate::portable::app_data_dir(app_handle)?.join("personalization.db");
        let manager = Self { db_path };
        manager.init_database()?;
        let mut settings = crate::settings::get_settings(app_handle);
        manager.migrate_legacy_words(&settings.custom_words)?;
        if !settings.custom_words.is_empty() {
            settings.custom_words.clear();
            crate::settings::write_settings(app_handle, settings);
        }
        Ok(manager)
    }

    #[cfg(test)]
    fn from_path(db_path: PathBuf) -> Result<Self> {
        let manager = Self { db_path };
        manager.init_database()?;
        Ok(manager)
    }

    fn init_database(&self) -> Result<()> {
        let mut connection = self.connection()?;
        MigrationRunner::new(PERSONALIZATION_MIGRATIONS)?.migrate_to_latest(&mut connection)
    }

    fn connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path).context("open personalization database")
    }

    fn migrate_legacy_words(&self, words: &[String]) -> Result<()> {
        if words.is_empty() {
            return Ok(());
        }
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let now = Utc::now().timestamp_millis();
        for word in words {
            let value = word.trim();
            if validate_entry(value, value).is_ok() {
                if ensure_unique_spoken(&transaction, value, None).is_err() {
                    continue;
                }
                transaction.execute(
                    "INSERT OR IGNORE INTO dictionary_entries
                     (spoken_form, replacement, starred, created_at, updated_at)
                     VALUES (?1, ?1, 0, ?2, ?2)",
                    params![value, now],
                )?;
            }
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn list(&self, query: Option<&str>, sort: DictionarySort) -> Result<Vec<DictionaryEntry>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, spoken_form, replacement, starred, created_at, updated_at
             FROM dictionary_entries",
        )?;
        let mut entries: Vec<DictionaryEntry> = statement
            .query_map([], map_entry)?
            .collect::<rusqlite::Result<_>>()
            .map_err(anyhow::Error::from)?;
        let query = query.unwrap_or_default().trim().to_lowercase();
        if !query.is_empty() {
            entries.retain(|entry| {
                entry.spoken_form.to_lowercase().contains(&query)
                    || entry.replacement.to_lowercase().contains(&query)
            });
        }
        entries.sort_by(|left, right| match sort {
            DictionarySort::Starred => right
                .starred
                .cmp(&left.starred)
                .then_with(|| right.updated_at.cmp(&left.updated_at))
                .then_with(|| left.id.cmp(&right.id)),
            DictionarySort::Updated => right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.id.cmp(&right.id)),
            DictionarySort::SpokenForm => left
                .spoken_form
                .to_lowercase()
                .cmp(&right.spoken_form.to_lowercase())
                .then_with(|| left.id.cmp(&right.id)),
        });
        Ok(entries)
    }

    pub fn create(
        &self,
        spoken_form: &str,
        replacement: &str,
        starred: bool,
    ) -> Result<DictionaryEntry> {
        let spoken_form = spoken_form.trim();
        let replacement = replacement.trim();
        validate_entry(spoken_form, replacement)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        enforce_capacity(&transaction, 1)?;
        ensure_unique_spoken(&transaction, spoken_form, None)?;
        let now = Utc::now().timestamp_millis();
        transaction
            .execute(
                "INSERT INTO dictionary_entries
                 (spoken_form, replacement, starred, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![spoken_form, replacement, starred, now],
            )
            .map_err(map_duplicate)?;
        let id = transaction.last_insert_rowid();
        let entry = get_entry(&transaction, id)?
            .ok_or_else(|| anyhow!("created dictionary entry missing"))?;
        transaction.commit()?;
        Ok(entry)
    }

    pub fn update(
        &self,
        id: i64,
        spoken_form: &str,
        replacement: &str,
        starred: bool,
    ) -> Result<DictionaryEntry> {
        let spoken_form = spoken_form.trim();
        let replacement = replacement.trim();
        validate_entry(spoken_form, replacement)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        ensure_unique_spoken(&transaction, spoken_form, Some(id))?;
        let changed = transaction
            .execute(
                "UPDATE dictionary_entries
                 SET spoken_form=?1, replacement=?2, starred=?3, updated_at=?4
                 WHERE id=?5",
                params![
                    spoken_form,
                    replacement,
                    starred,
                    Utc::now().timestamp_millis(),
                    id
                ],
            )
            .map_err(map_duplicate)?;
        if changed == 0 {
            bail!("dictionary entry not found");
        }
        let entry = get_entry(&transaction, id)?
            .ok_or_else(|| anyhow!("updated dictionary entry missing"))?;
        transaction.commit()?;
        Ok(entry)
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        if self
            .connection()?
            .execute("DELETE FROM dictionary_entries WHERE id=?1", [id])?
            == 0
        {
            bail!("dictionary entry not found");
        }
        Ok(())
    }

    pub fn export_csv(&self) -> Result<String> {
        let entries = self.list(None, DictionarySort::SpokenForm)?;
        let mut csv = format!("{CSV_HEADER}\r\n");
        for entry in entries {
            csv.push_str(&csv_escape(&entry.spoken_form));
            csv.push(',');
            csv.push_str(&csv_escape(&entry.replacement));
            csv.push(',');
            csv.push_str(if entry.starred { "true" } else { "false" });
            csv.push_str("\r\n");
        }
        Ok(csv)
    }

    pub fn import_csv(&self, csv: &str) -> Result<usize> {
        let rows = parse_csv(csv)?;
        if rows.first().map(|row| row.as_slice())
            != Some(&[
                "spoken_form".to_string(),
                "replacement".to_string(),
                "starred".to_string(),
            ])
        {
            bail!("CSV header must be spoken_form,replacement,starred");
        }
        let parsed = rows
            .into_iter()
            .skip(1)
            .filter(|row| !row.iter().all(String::is_empty))
            .map(|row| {
                if row.len() != 3 {
                    bail!("each CSV row must contain three fields");
                }
                let spoken = row[0].trim().to_string();
                let replacement = row[1].trim().to_string();
                validate_entry(&spoken, &replacement)?;
                let starred = match row[2].trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" => true,
                    "false" | "0" | "no" | "" => false,
                    _ => bail!("starred must be true or false"),
                };
                Ok((spoken, replacement, starred))
            })
            .collect::<Result<Vec<_>>>()?;

        let mut normalized = std::collections::HashSet::new();
        for (spoken, _, _) in &parsed {
            if !normalized.insert(spoken.to_lowercase()) {
                bail!("CSV contains duplicate spoken forms");
            }
        }

        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        enforce_capacity(&transaction, parsed.len())?;
        let now = Utc::now().timestamp_millis();
        for (spoken, replacement, starred) in &parsed {
            ensure_unique_spoken(&transaction, spoken, None)?;
            transaction
                .execute(
                    "INSERT INTO dictionary_entries
                 (spoken_form, replacement, starred, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![spoken, replacement, starred, now],
                )
                .map_err(map_duplicate)?;
        }
        transaction.commit()?;
        Ok(parsed.len())
    }

    pub fn apply(&self, text: &str) -> Result<String> {
        apply_entries(text, &self.list(None, DictionarySort::Starred)?)
    }

    pub fn prompt_terms(&self) -> Result<Vec<String>> {
        Ok(self
            .list(None, DictionarySort::Starred)?
            .into_iter()
            .map(|entry| entry.replacement)
            .collect())
    }
}

fn validate_entry(spoken: &str, replacement: &str) -> Result<()> {
    if spoken.is_empty() || replacement.is_empty() {
        bail!("spoken form and replacement are required");
    }
    if spoken.chars().count() > MAX_SPOKEN_FORM_CHARS {
        bail!("spoken form exceeds {MAX_SPOKEN_FORM_CHARS} characters");
    }
    if replacement.chars().count() > MAX_REPLACEMENT_CHARS {
        bail!("replacement exceeds {MAX_REPLACEMENT_CHARS} characters");
    }
    if spoken.contains(['\r', '\n']) {
        bail!("spoken form cannot contain a line break");
    }
    Ok(())
}

fn enforce_capacity(transaction: &Transaction<'_>, additional: usize) -> Result<()> {
    let current: usize =
        transaction.query_row("SELECT COUNT(*) FROM dictionary_entries", [], |row| {
            row.get(0)
        })?;
    if current.saturating_add(additional) > MAX_DICTIONARY_ENTRIES {
        bail!("dictionary limit of {MAX_DICTIONARY_ENTRIES} entries exceeded");
    }
    Ok(())
}

fn ensure_unique_spoken(
    transaction: &Transaction<'_>,
    spoken: &str,
    except_id: Option<i64>,
) -> Result<()> {
    let normalized = spoken.to_lowercase();
    let mut statement = transaction.prepare("SELECT id, spoken_form FROM dictionary_entries")?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (id, existing) = row?;
        if Some(id) != except_id && existing.to_lowercase() == normalized {
            bail!("a dictionary entry with that spoken form already exists");
        }
    }
    Ok(())
}

fn map_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<DictionaryEntry> {
    Ok(DictionaryEntry {
        id: row.get(0)?,
        spoken_form: row.get(1)?,
        replacement: row.get(2)?,
        starred: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn get_entry(connection: &Connection, id: i64) -> Result<Option<DictionaryEntry>> {
    connection.query_row("SELECT id, spoken_form, replacement, starred, created_at, updated_at FROM dictionary_entries WHERE id=?1", [id], map_entry).optional().map_err(Into::into)
}

fn map_duplicate(error: rusqlite::Error) -> anyhow::Error {
    if matches!(&error, rusqlite::Error::SqliteFailure(code, _) if code.extended_code == 2067) {
        anyhow!("a dictionary entry with that spoken form already exists")
    } else {
        error.into()
    }
}

fn apply_entries(text: &str, entries: &[DictionaryEntry]) -> Result<String> {
    #[derive(Debug)]
    struct Match<'a> {
        start: usize,
        end: usize,
        entry: &'a DictionaryEntry,
        spoken_len: usize,
    }
    let mut matches = Vec::new();
    for entry in entries {
        let regex = RegexBuilder::new(&regex::escape(&entry.spoken_form))
            .case_insensitive(true)
            .unicode(true)
            .build()?;
        for found in regex.find_iter(text) {
            let before_ok = text[..found.start()]
                .chars()
                .next_back()
                .is_none_or(|c| !is_word(c));
            let after_ok = text[found.end()..]
                .chars()
                .next()
                .is_none_or(|c| !is_word(c));
            if before_ok && after_ok {
                matches.push(Match {
                    start: found.start(),
                    end: found.end(),
                    entry,
                    spoken_len: entry.spoken_form.chars().count(),
                });
            }
        }
    }
    matches.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.spoken_len.cmp(&left.spoken_len))
            .then_with(|| right.entry.starred.cmp(&left.entry.starred))
            .then_with(|| left.entry.id.cmp(&right.entry.id))
    });
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    for candidate in matches {
        if candidate.start < cursor {
            continue;
        }
        output.push_str(&text[cursor..candidate.start]);
        output.push_str(&candidate.entry.replacement);
        cursor = candidate.end;
    }
    output.push_str(&text[cursor..]);
    Ok(output)
}

fn is_word(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn csv_escape(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn parse_csv(input: &str) -> Result<Vec<Vec<String>>> {
    let mut rows = vec![vec![String::new()]];
    let mut quoted = false;
    let mut chars = input.chars().peekable();
    while let Some(character) = chars.next() {
        match character {
            '"' if quoted && chars.peek() == Some(&'"') => {
                rows.last_mut().unwrap().last_mut().unwrap().push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => rows.last_mut().unwrap().push(String::new()),
            '\n' if !quoted => rows.push(vec![String::new()]),
            '\r' if !quoted && chars.peek() == Some(&'\n') => {}
            other => rows.last_mut().unwrap().last_mut().unwrap().push(other),
        }
    }
    if quoted {
        bail!("CSV contains an unterminated quoted field");
    }
    if rows
        .last()
        .is_some_and(|row| row.len() == 1 && row[0].is_empty())
    {
        rows.pop();
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> (tempfile::TempDir, DictionaryManager) {
        let directory = tempfile::tempdir().unwrap();
        let manager =
            DictionaryManager::from_path(directory.path().join("personalization.db")).unwrap();
        (directory, manager)
    }

    #[test]
    fn unicode_case_boundaries_and_precedence_are_deterministic() {
        let (_directory, manager) = manager();
        manager.create("café", "Café™", false).unwrap();
        manager.create("new york", "NYC", false).unwrap();
        manager.create("new", "NEW", true).unwrap();
        assert_eq!(
            manager.apply("CAFÉ, new york; decafé.").unwrap(),
            "Café™, NYC; decafé."
        );
    }

    #[test]
    fn duplicates_are_case_insensitive_and_limits_are_enforced() {
        let (_directory, manager) = manager();
        manager.create("Résumé", "Résumé", false).unwrap();
        assert!(manager
            .create("rÉSUMÉ", "CV", false)
            .unwrap_err()
            .to_string()
            .contains("already exists"));
        assert!(manager
            .create(&"x".repeat(MAX_SPOKEN_FORM_CHARS + 1), "x", false)
            .is_err());
        let mut connection = manager.connection().unwrap();
        let transaction = connection.transaction().unwrap();
        assert!(enforce_capacity(&transaction, MAX_DICTIONARY_ENTRIES).is_err());
        assert!(manager
            .create("x", &"y".repeat(MAX_REPLACEMENT_CHARS + 1), false)
            .is_err());
    }

    #[test]
    fn csv_round_trip_supports_unicode_quotes_and_newlines() {
        let (_directory, source) = manager();
        source
            .create("résumé", "CV, \"résumé\"\nready", true)
            .unwrap();
        let csv = source.export_csv().unwrap();
        let (_other_directory, other) = manager();
        assert_eq!(other.import_csv(&csv).unwrap(), 1);
        assert_eq!(
            other.list(None, DictionarySort::Starred).unwrap()[0].replacement,
            "CV, \"résumé\"\nready"
        );
    }

    #[test]
    fn invalid_import_rolls_back_every_row() {
        let (_directory, manager) = manager();
        let csv = "spoken_form,replacement,starred\nvalid,Valid,true\nbad,Bad,maybe\n";
        assert!(manager.import_csv(csv).is_err());
        assert!(manager
            .list(None, DictionarySort::Updated)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn duplicate_import_rolls_back_every_row() {
        let (_directory, manager) = manager();
        manager.create("existing", "Existing", false).unwrap();
        let csv = "spoken_form,replacement,starred\nnew,New,false\nEXISTING,Duplicate,false\n";
        assert!(manager.import_csv(csv).is_err());
        assert_eq!(
            manager.list(None, DictionarySort::Updated).unwrap().len(),
            1
        );
    }

    #[test]
    fn search_and_sort_are_unicode_aware() {
        let (_directory, manager) = manager();
        manager.create("Éclair", "dessert", false).unwrap();
        manager.create("alpha", "first", true).unwrap();
        let found = manager
            .list(Some("éCL"), DictionarySort::SpokenForm)
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].spoken_form, "Éclair");
        assert_eq!(
            manager.list(None, DictionarySort::Starred).unwrap()[0].spoken_form,
            "alpha"
        );
    }

    #[test]
    fn legacy_words_migrate_idempotently() {
        let (_directory, manager) = manager();
        let words = vec!["FreeFlow".to_string(), "freeflow".to_string()];
        manager.migrate_legacy_words(&words).unwrap();
        manager.migrate_legacy_words(&words).unwrap();
        assert_eq!(
            manager.list(None, DictionarySort::Updated).unwrap().len(),
            1
        );
    }
}

use crate::storage::migrations::{MigrationRunner, PERSONALIZATION_MIGRATIONS};
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use regex::{Regex, RegexBuilder};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashSet;
use std::path::PathBuf;
use tauri::AppHandle;

pub const MAX_SNIPPETS: usize = 1_000;
pub const MAX_SNIPPET_NAME_CHARS: usize = 100;
pub const MAX_TRIGGER_CHARS: usize = 200;
pub const MAX_EXPANSION_CHARS: usize = 4_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct Snippet {
    pub id: i64,
    pub name: String,
    pub trigger_phrase: String,
    pub expansion: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SnippetSort {
    Updated,
    Name,
    TriggerPhrase,
}

#[derive(Serialize, Deserialize)]
struct SnippetTransfer {
    schema_version: u32,
    snippets: Vec<SnippetTransferEntry>,
}

#[derive(Serialize, Deserialize)]
struct SnippetTransferEntry {
    name: String,
    trigger_phrase: String,
    expansion: String,
}

pub struct SnippetManager {
    db_path: PathBuf,
}

impl SnippetManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self> {
        let manager = Self {
            db_path: crate::portable::app_data_dir(app_handle)?.join("personalization.db"),
        };
        manager.init_database()?;
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

    pub fn list(&self, query: Option<&str>, sort: SnippetSort) -> Result<Vec<Snippet>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, name, trigger_phrase, expansion, created_at, updated_at FROM snippets",
        )?;
        let mut snippets: Vec<Snippet> = statement
            .query_map([], map_snippet)?
            .collect::<rusqlite::Result<_>>()?;
        let query = query.unwrap_or_default().trim().to_lowercase();
        if !query.is_empty() {
            snippets.retain(|snippet| {
                snippet.name.to_lowercase().contains(&query)
                    || snippet.trigger_phrase.to_lowercase().contains(&query)
                    || snippet.expansion.to_lowercase().contains(&query)
            });
        }
        snippets.sort_by(|left, right| match sort {
            SnippetSort::Updated => right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| left.id.cmp(&right.id)),
            SnippetSort::Name => left
                .name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id)),
            SnippetSort::TriggerPhrase => left
                .trigger_phrase
                .to_lowercase()
                .cmp(&right.trigger_phrase.to_lowercase())
                .then_with(|| left.id.cmp(&right.id)),
        });
        Ok(snippets)
    }

    pub fn create(&self, name: &str, trigger: &str, expansion: &str) -> Result<Snippet> {
        let (name, trigger, expansion) = validated(name, trigger, expansion)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        enforce_capacity(&transaction, 1)?;
        ensure_unique_trigger(&transaction, trigger, None)?;
        let now = Utc::now().timestamp_millis();
        transaction
            .execute(
                "INSERT INTO snippets (name, trigger_phrase, expansion, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)",
                params![name, trigger, expansion, now],
            )
            .map_err(map_duplicate)?;
        let snippet = get_snippet(&transaction, transaction.last_insert_rowid())?
            .ok_or_else(|| anyhow!("created snippet missing"))?;
        transaction.commit()?;
        Ok(snippet)
    }

    pub fn update(&self, id: i64, name: &str, trigger: &str, expansion: &str) -> Result<Snippet> {
        let (name, trigger, expansion) = validated(name, trigger, expansion)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        ensure_unique_trigger(&transaction, trigger, Some(id))?;
        let changed = transaction
            .execute(
                "UPDATE snippets SET name=?1, trigger_phrase=?2, expansion=?3, updated_at=?4
                 WHERE id=?5",
                params![name, trigger, expansion, Utc::now().timestamp_millis(), id],
            )
            .map_err(map_duplicate)?;
        if changed == 0 {
            bail!("snippet not found");
        }
        let snippet =
            get_snippet(&transaction, id)?.ok_or_else(|| anyhow!("updated snippet missing"))?;
        transaction.commit()?;
        Ok(snippet)
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        if self
            .connection()?
            .execute("DELETE FROM snippets WHERE id=?1", [id])?
            == 0
        {
            bail!("snippet not found");
        }
        Ok(())
    }

    pub fn export_json(&self) -> Result<String> {
        let snippets = self
            .list(None, SnippetSort::Name)?
            .into_iter()
            .map(|snippet| SnippetTransferEntry {
                name: snippet.name,
                trigger_phrase: snippet.trigger_phrase,
                expansion: snippet.expansion,
            })
            .collect();
        serde_json::to_string_pretty(&SnippetTransfer {
            schema_version: 1,
            snippets,
        })
        .context("serialize snippets")
    }

    pub fn import_json(&self, json: &str) -> Result<usize> {
        let transfer: SnippetTransfer = serde_json::from_str(json).context("parse snippet JSON")?;
        if transfer.schema_version != 1 {
            bail!("unsupported snippet schema version");
        }
        let parsed = transfer
            .snippets
            .iter()
            .map(|entry| validated(&entry.name, &entry.trigger_phrase, &entry.expansion))
            .collect::<Result<Vec<_>>>()?;
        let mut seen = HashSet::new();
        for (_, trigger, _) in &parsed {
            if !seen.insert(trigger.to_lowercase()) {
                bail!("JSON contains duplicate trigger phrases");
            }
        }

        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        enforce_capacity(&transaction, parsed.len())?;
        for (_, trigger, _) in &parsed {
            ensure_unique_trigger(&transaction, trigger, None)?;
        }
        let now = Utc::now().timestamp_millis();
        for (name, trigger, expansion) in &parsed {
            transaction
                .execute(
                    "INSERT INTO snippets
                     (name, trigger_phrase, expansion, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![name, trigger, expansion, now],
                )
                .map_err(map_duplicate)?;
        }
        transaction.commit()?;
        Ok(parsed.len())
    }

    pub fn expand(&self, text: &str) -> Result<String> {
        let snippets = self.list(None, SnippetSort::Updated)?;
        expand_with_snippets(text, &snippets)
    }
}

#[derive(Debug)]
struct Candidate<'a> {
    start: usize,
    end: usize,
    trigger_chars: usize,
    id: i64,
    expansion: &'a str,
}

fn expand_with_snippets(text: &str, snippets: &[Snippet]) -> Result<String> {
    let mut candidates = Vec::new();
    for snippet in snippets {
        let regex = RegexBuilder::new(&regex::escape(&snippet.trigger_phrase))
            .case_insensitive(true)
            .unicode(true)
            .build()
            .context("compile snippet matcher")?;
        collect_matches(text, snippet, &regex, &mut candidates);
    }
    candidates.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.trigger_chars.cmp(&left.trigger_chars))
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    for candidate in candidates {
        if candidate.start < cursor {
            continue;
        }
        output.push_str(&text[cursor..candidate.start]);
        output.push_str(candidate.expansion);
        cursor = candidate.end;
    }
    output.push_str(&text[cursor..]);
    Ok(output)
}

fn collect_matches<'a>(
    text: &str,
    snippet: &'a Snippet,
    regex: &Regex,
    candidates: &mut Vec<Candidate<'a>>,
) {
    for trigger in regex.find_iter(text) {
        if is_phrase_boundary(text[..trigger.start()].chars().next_back())
            && is_phrase_boundary(text[trigger.end()..].chars().next())
        {
            candidates.push(Candidate {
                start: trigger.start(),
                end: trigger.end(),
                trigger_chars: snippet.trigger_phrase.chars().count(),
                id: snippet.id,
                expansion: &snippet.expansion,
            });
        }
    }
}

fn is_phrase_boundary(character: Option<char>) -> bool {
    character.is_none_or(|character| !character.is_alphanumeric() && character != '_')
}

fn validated<'a>(
    name: &'a str,
    trigger: &'a str,
    expansion: &'a str,
) -> Result<(&'a str, &'a str, &'a str)> {
    let name = name.trim();
    let trigger = trigger.trim();
    if name.is_empty() || trigger.is_empty() || expansion.is_empty() {
        bail!("snippet name, trigger phrase, and expansion are required");
    }
    if name.chars().count() > MAX_SNIPPET_NAME_CHARS {
        bail!("snippet name exceeds {MAX_SNIPPET_NAME_CHARS} characters");
    }
    if trigger.chars().count() > MAX_TRIGGER_CHARS {
        bail!("trigger phrase exceeds {MAX_TRIGGER_CHARS} characters");
    }
    if expansion.chars().count() > MAX_EXPANSION_CHARS {
        bail!("snippet expansion exceeds {MAX_EXPANSION_CHARS} characters");
    }
    Ok((name, trigger, expansion))
}

fn enforce_capacity(transaction: &Transaction<'_>, additional: usize) -> Result<()> {
    let count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM snippets", [], |row| row.get(0))?;
    let count = usize::try_from(count).context("invalid snippet count")?;
    if count.saturating_add(additional) > MAX_SNIPPETS {
        bail!("snippet limit of {MAX_SNIPPETS} entries exceeded");
    }
    Ok(())
}

fn ensure_unique_trigger(
    transaction: &Transaction<'_>,
    trigger: &str,
    except_id: Option<i64>,
) -> Result<()> {
    let normalized = trigger.to_lowercase();
    let mut statement = transaction.prepare("SELECT id, trigger_phrase FROM snippets")?;
    let existing = statement
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .any(|(id, stored)| Some(id) != except_id && stored.to_lowercase() == normalized);
    if existing {
        bail!("a snippet with this trigger phrase already exists");
    }
    Ok(())
}

fn map_duplicate(error: rusqlite::Error) -> anyhow::Error {
    match &error {
        rusqlite::Error::SqliteFailure(inner, _)
            if inner.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            anyhow!("a snippet with this trigger phrase already exists")
        }
        _ => anyhow!(error),
    }
}

fn get_snippet(transaction: &Transaction<'_>, id: i64) -> Result<Option<Snippet>> {
    transaction
        .query_row(
            "SELECT id, name, trigger_phrase, expansion, created_at, updated_at
             FROM snippets WHERE id=?1",
            [id],
            map_snippet,
        )
        .optional()
        .map_err(anyhow::Error::from)
}

fn map_snippet(row: &rusqlite::Row<'_>) -> rusqlite::Result<Snippet> {
    Ok(Snippet {
        id: row.get(0)?,
        name: row.get(1)?,
        trigger_phrase: row.get(2)?,
        expansion: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manager() -> (tempfile::TempDir, SnippetManager) {
        let directory = tempfile::tempdir().unwrap();
        let manager =
            SnippetManager::from_path(directory.path().join("personalization.db")).unwrap();
        (directory, manager)
    }

    #[test]
    fn expands_case_punctuation_whole_phrases_and_multiple_triggers() {
        let (_directory, manager) = manager();
        manager
            .create("Greeting", "hello team", "Hello, everyone!")
            .unwrap();
        manager
            .create("Signoff", "sign off", "Regards,\nAvery")
            .unwrap();
        assert_eq!(
            manager.expand("HELLO TEAM; then sign off.").unwrap(),
            "Hello, everyone!; then Regards,\nAvery."
        );
        assert_eq!(
            manager.expand("shell hello teamwork").unwrap(),
            "shell hello teamwork"
        );
        assert_eq!(
            manager.expand("hello team,hello team").unwrap(),
            "Hello, everyone!,Hello, everyone!"
        );
    }

    #[test]
    fn longest_overlapping_trigger_wins_deterministically() {
        let (_directory, manager) = manager();
        manager.create("Short", "insert", "SHORT").unwrap();
        manager.create("Long", "insert address", "LONG").unwrap();
        assert_eq!(manager.expand("insert address now").unwrap(), "LONG now");
    }

    #[test]
    fn unicode_and_maximum_expansion_are_preserved_exactly() {
        let (_directory, manager) = manager();
        let expansion = "é".repeat(MAX_EXPANSION_CHARS);
        manager.create("Unicode", "CAFÉ", &expansion).unwrap();
        assert_eq!(manager.expand("café!").unwrap(), format!("{expansion}!"));
    }

    #[test]
    fn duplicate_and_invalid_imports_roll_back_atomically() {
        let (_directory, manager) = manager();
        manager.create("Existing", "my address", "one").unwrap();
        manager.create("Unicode", "CAFÉ", "coffee").unwrap();
        assert!(manager
            .create("Unicode duplicate", "café", "other")
            .is_err());
        let duplicate = r#"{"schema_version":1,"snippets":[{"name":"A","trigger_phrase":"new trigger","expansion":"ok"},{"name":"B","trigger_phrase":"MY ADDRESS","expansion":"conflict"}]}"#;
        assert!(manager.import_json(duplicate).is_err());
        assert_eq!(manager.list(None, SnippetSort::Updated).unwrap().len(), 2);
        let within_file = r#"{"schema_version":1,"snippets":[{"name":"A","trigger_phrase":"same","expansion":"one"},{"name":"B","trigger_phrase":"SAME","expansion":"two"}]}"#;
        assert!(manager.import_json(within_file).is_err());
        assert_eq!(manager.list(None, SnippetSort::Updated).unwrap().len(), 2);
    }

    #[test]
    fn json_round_trip_preserves_multiline_expansion() {
        let (_directory, manager) = manager();
        manager
            .create("Template", "insert note", "line one\nline two")
            .unwrap();
        let json = manager.export_json().unwrap();
        let other_directory = tempfile::tempdir().unwrap();
        let other =
            SnippetManager::from_path(other_directory.path().join("personalization.db")).unwrap();
        assert_eq!(other.import_json(&json).unwrap(), 1);
        assert_eq!(other.expand("insert note").unwrap(), "line one\nline two");
    }
}

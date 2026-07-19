use anyhow::{anyhow, bail, Context, Result};
use rusqlite::{Connection, TransactionBehavior};

#[derive(Clone, Copy, Debug)]
pub struct ReversibleMigration {
    pub version: u32,
    pub up_sql: &'static str,
    pub down_sql: &'static str,
}

pub const HISTORY_MIGRATIONS: &[ReversibleMigration] = &[
    ReversibleMigration {
        version: 1,
        up_sql: "CREATE TABLE IF NOT EXISTS transcription_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_name TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            saved BOOLEAN NOT NULL DEFAULT 0,
            title TEXT NOT NULL,
            transcription_text TEXT NOT NULL
        );",
        down_sql: "DROP TABLE IF EXISTS transcription_history;",
    },
    ReversibleMigration {
        version: 2,
        up_sql: "ALTER TABLE transcription_history ADD COLUMN post_processed_text TEXT;",
        down_sql: "ALTER TABLE transcription_history DROP COLUMN post_processed_text;",
    },
    ReversibleMigration {
        version: 3,
        up_sql: "ALTER TABLE transcription_history ADD COLUMN post_process_prompt TEXT;",
        down_sql: "ALTER TABLE transcription_history DROP COLUMN post_process_prompt;",
    },
    ReversibleMigration {
        version: 4,
        up_sql: "ALTER TABLE transcription_history ADD COLUMN post_process_requested BOOLEAN NOT NULL DEFAULT 0;",
        down_sql: "ALTER TABLE transcription_history DROP COLUMN post_process_requested;",
    },
    ReversibleMigration {
        version: 5,
        up_sql: "ALTER TABLE transcription_history ADD COLUMN raw_transcript TEXT NOT NULL DEFAULT '';
            ALTER TABLE transcription_history ADD COLUMN model_id TEXT;
            ALTER TABLE transcription_history ADD COLUMN requested_language TEXT;
            ALTER TABLE transcription_history ADD COLUMN effective_language TEXT;
            ALTER TABLE transcription_history ADD COLUMN detected_language TEXT;
            ALTER TABLE transcription_history ADD COLUMN audio_duration_ms INTEGER;
            ALTER TABLE transcription_history ADD COLUMN transcription_ms INTEGER;
            ALTER TABLE transcription_history ADD COLUMN transcription_status TEXT NOT NULL DEFAULT 'pending';
            ALTER TABLE transcription_history ADD COLUMN transcription_error TEXT;
            UPDATE transcription_history
            SET raw_transcript = transcription_text,
                transcription_status = CASE
                    WHEN transcription_text = '' THEN 'pending'
                    ELSE 'completed'
                END;",
        down_sql: "ALTER TABLE transcription_history DROP COLUMN transcription_error;
            ALTER TABLE transcription_history DROP COLUMN transcription_status;
            ALTER TABLE transcription_history DROP COLUMN transcription_ms;
            ALTER TABLE transcription_history DROP COLUMN audio_duration_ms;
            ALTER TABLE transcription_history DROP COLUMN detected_language;
            ALTER TABLE transcription_history DROP COLUMN effective_language;
            ALTER TABLE transcription_history DROP COLUMN requested_language;
            ALTER TABLE transcription_history DROP COLUMN model_id;
            ALTER TABLE transcription_history DROP COLUMN raw_transcript;",
    },
    ReversibleMigration {
        version: 6,
        up_sql: "ALTER TABLE transcription_history ADD COLUMN application_id TEXT;
            ALTER TABLE transcription_history ADD COLUMN window_title TEXT;
            CREATE INDEX transcription_history_timestamp_idx
                ON transcription_history(timestamp DESC);
            CREATE INDEX transcription_history_status_idx
                ON transcription_history(transcription_status);",
        down_sql: "DROP INDEX IF EXISTS transcription_history_status_idx;
            DROP INDEX IF EXISTS transcription_history_timestamp_idx;
            ALTER TABLE transcription_history DROP COLUMN window_title;
            ALTER TABLE transcription_history DROP COLUMN application_id;",
    },
];

pub struct MigrationRunner<'a> {
    migrations: &'a [ReversibleMigration],
}

impl<'a> MigrationRunner<'a> {
    pub fn new(migrations: &'a [ReversibleMigration]) -> Result<Self> {
        for (index, migration) in migrations.iter().enumerate() {
            let expected = u32::try_from(index + 1).context("migration index overflow")?;
            if migration.version != expected {
                bail!(
                    "migration versions must be contiguous: expected {expected}, found {}",
                    migration.version
                );
            }
        }
        Ok(Self { migrations })
    }

    pub fn latest_version(&self) -> u32 {
        self.migrations
            .last()
            .map_or(0, |migration| migration.version)
    }

    pub fn current_version(conn: &Connection) -> Result<u32> {
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .context("read SQLite user_version")
    }

    pub fn migrate_to_latest(&self, conn: &mut Connection) -> Result<()> {
        self.migrate_to(conn, self.latest_version())
    }

    pub fn migrate_to(&self, conn: &mut Connection, target: u32) -> Result<()> {
        let latest = self.latest_version();
        if target > latest {
            bail!("target migration {target} exceeds latest version {latest}");
        }

        let current = Self::current_version(conn)?;
        if current > latest {
            bail!("database version {current} is newer than supported version {latest}");
        }
        if current == target {
            return Ok(());
        }

        let transaction = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .context("start migration transaction")?;

        if current < target {
            for version in (current + 1)..=target {
                let migration = self
                    .migration(version)
                    .ok_or_else(|| anyhow!("missing migration {version}"))?;
                transaction
                    .execute_batch(migration.up_sql)
                    .with_context(|| format!("apply migration {version}"))?;
                transaction.pragma_update(None, "user_version", version)?;
            }
        } else {
            for version in ((target + 1)..=current).rev() {
                let migration = self
                    .migration(version)
                    .ok_or_else(|| anyhow!("missing migration {version}"))?;
                transaction
                    .execute_batch(migration.down_sql)
                    .with_context(|| format!("roll back migration {version}"))?;
                transaction.pragma_update(None, "user_version", version - 1)?;
            }
        }

        transaction.commit().context("commit migration transaction")
    }

    fn migration(&self, version: u32) -> Option<&ReversibleMigration> {
        let index = usize::try_from(version.checked_sub(1)?).ok()?;
        self.migrations.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_migrations_apply_forward_and_roll_back() {
        let runner = MigrationRunner::new(HISTORY_MIGRATIONS).expect("valid history migrations");
        let mut conn = Connection::open_in_memory().expect("open in-memory database");

        runner
            .migrate_to_latest(&mut conn)
            .expect("migrate forward");
        assert_eq!(MigrationRunner::current_version(&conn).expect("version"), 6);
        conn.execute(
            "INSERT INTO transcription_history (
                file_name, timestamp, saved, title, transcription_text,
                post_processed_text, post_process_prompt, post_process_requested
            ) VALUES ('sample.wav', 1, 0, 'sample', 'hello', NULL, NULL, 0)",
            [],
        )
        .expect("insert migrated row");

        runner.migrate_to(&mut conn, 1).expect("roll back to v1");
        assert_eq!(MigrationRunner::current_version(&conn).expect("version"), 1);
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(transcription_history)")
            .expect("prepare table_info")
            .query_map([], |row| row.get(1))
            .expect("query columns")
            .collect::<rusqlite::Result<_>>()
            .expect("collect columns");
        assert!(!columns.iter().any(|column| column == "post_processed_text"));
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM transcription_history", [], |row| row
                .get::<_, i64>(
                0
            ))
            .expect("count rows"),
            1
        );
    }

    #[test]
    fn failed_migration_rolls_back_schema_and_version() {
        const BROKEN: &[ReversibleMigration] = &[
            ReversibleMigration {
                version: 1,
                up_sql: "CREATE TABLE durable (id INTEGER PRIMARY KEY);",
                down_sql: "DROP TABLE durable;",
            },
            ReversibleMigration {
                version: 2,
                up_sql: "THIS IS NOT SQL;",
                down_sql: "SELECT 1;",
            },
        ];
        let runner = MigrationRunner::new(BROKEN).expect("valid version sequence");
        let mut conn = Connection::open_in_memory().expect("open in-memory database");

        assert!(runner.migrate_to_latest(&mut conn).is_err());
        assert_eq!(MigrationRunner::current_version(&conn).expect("version"), 0);
        let table_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='durable'",
                [],
                |row| row.get(0),
            )
            .expect("query table count");
        assert_eq!(table_count, 0);
    }

    #[test]
    fn migration_state_and_data_survive_process_restart() {
        let file = tempfile::NamedTempFile::new().expect("create database path");
        let path = file.path().to_path_buf();
        drop(file);
        let runner = MigrationRunner::new(HISTORY_MIGRATIONS).expect("valid history migrations");

        {
            let mut conn = Connection::open(&path).expect("open first connection");
            runner
                .migrate_to_latest(&mut conn)
                .expect("migrate database");
            conn.execute(
                "INSERT INTO transcription_history (
                    file_name, timestamp, saved, title, transcription_text,
                    post_processed_text, post_process_prompt, post_process_requested
                ) VALUES ('restart.wav', 2, 0, 'restart', 'persisted', NULL, NULL, 0)",
                [],
            )
            .expect("insert persistent row");
        }

        let mut reopened = Connection::open(&path).expect("reopen database");
        runner
            .migrate_to_latest(&mut reopened)
            .expect("idempotent restart migration");
        assert_eq!(
            MigrationRunner::current_version(&reopened).expect("version"),
            6
        );
        let text: String = reopened
            .query_row(
                "SELECT transcription_text FROM transcription_history WHERE file_name='restart.wav'",
                [],
                |row| row.get(0),
            )
            .expect("read persistent row");
        assert_eq!(text, "persisted");
    }

    #[test]
    fn raw_transcript_migration_backfills_existing_rows() {
        let runner = MigrationRunner::new(HISTORY_MIGRATIONS).expect("valid history migrations");
        let mut conn = Connection::open_in_memory().expect("open database");
        runner
            .migrate_to(&mut conn, 4)
            .expect("migrate to legacy schema");
        conn.execute(
            "INSERT INTO transcription_history (
                file_name, timestamp, saved, title, transcription_text,
                post_processed_text, post_process_prompt, post_process_requested
            ) VALUES ('legacy.wav', 1, 0, 'legacy', 'raw words', 'clean words', NULL, 1)",
            [],
        )
        .expect("insert legacy row");

        runner
            .migrate_to_latest(&mut conn)
            .expect("migrate to FF-V3 schema");
        let (raw, status): (String, String) = conn
            .query_row(
                "SELECT raw_transcript, transcription_status
                 FROM transcription_history WHERE file_name = 'legacy.wav'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read migrated row");
        assert_eq!(raw, "raw words");
        assert_eq!(status, "completed");
    }
}

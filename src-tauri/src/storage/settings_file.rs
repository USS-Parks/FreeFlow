use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const SETTINGS_KEY: &str = "settings";

#[derive(Debug)]
pub struct SettingsLoad {
    pub value: Value,
    pub recovered_corrupt_path: Option<PathBuf>,
}

#[derive(Clone)]
pub struct AtomicSettingsFile {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl AtomicSettingsFile {
    pub fn new(path: PathBuf) -> Self {
        Self::with_lock(path, Arc::new(Mutex::new(())))
    }

    pub fn with_lock(path: PathBuf, lock: Arc<Mutex<()>>) -> Self {
        Self { path, lock }
    }

    pub fn load_or_recover(&self, default: &Value) -> Result<SettingsLoad> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if !self.path.exists() {
            self.save_unlocked(default)?;
            return Ok(SettingsLoad {
                value: default.clone(),
                recovered_corrupt_path: None,
            });
        }

        let bytes = fs::read(&self.path)
            .with_context(|| format!("read settings file {}", self.path.display()))?;
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(root) => {
                let Some(value) = root.get(SETTINGS_KEY).cloned() else {
                    self.save_unlocked(default)?;
                    return Ok(SettingsLoad {
                        value: default.clone(),
                        recovered_corrupt_path: None,
                    });
                };
                Ok(SettingsLoad {
                    value,
                    recovered_corrupt_path: None,
                })
            }
            Err(parse_error) => {
                let backup = self.corrupt_backup_path()?;
                fs::rename(&self.path, &backup).with_context(|| {
                    format!(
                        "preserve corrupt settings {} as {} after {parse_error}",
                        self.path.display(),
                        backup.display()
                    )
                })?;
                self.save_unlocked(default)?;
                Ok(SettingsLoad {
                    value: default.clone(),
                    recovered_corrupt_path: Some(backup),
                })
            }
        }
    }

    pub fn save(&self, settings: &Value) -> Result<()> {
        let _guard = self
            .lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.save_unlocked(settings)
    }

    fn save_unlocked(&self, settings: &Value) -> Result<()> {
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent)
            .with_context(|| format!("create settings directory {}", parent.display()))?;

        let mut root = Map::new();
        root.insert(SETTINGS_KEY.to_string(), settings.clone());

        let temp = tempfile::NamedTempFile::new_in(parent)
            .with_context(|| format!("create temporary settings file in {}", parent.display()))?;
        {
            let mut writer = BufWriter::new(temp.as_file());
            serde_json::to_writer_pretty(&mut writer, &Value::Object(root))
                .context("serialize typed settings")?;
            writer.write_all(b"\n").context("terminate settings JSON")?;
            writer.flush().context("flush settings JSON")?;
        }
        temp.as_file()
            .sync_all()
            .context("sync temporary settings file")?;
        temp.persist(&self.path)
            .map_err(|error| error.error)
            .with_context(|| format!("atomically replace settings file {}", self.path.display()))?;

        #[cfg(unix)]
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .with_context(|| format!("sync settings directory {}", parent.display()))?;

        Ok(())
    }

    fn corrupt_backup_path(&self) -> Result<PathBuf> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock precedes Unix epoch")?
            .as_millis();
        let file_name = self
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("settings.json");
        Ok(self.path.with_file_name(format!(
            "{file_name}.corrupt-{timestamp}-{}",
            std::process::id()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn atomic_settings_survive_repository_restart() {
        let directory = tempfile::tempdir().expect("create settings directory");
        let path = directory.path().join("settings_store.json");
        let first = AtomicSettingsFile::new(path.clone());
        first
            .save(&json!({"settings_schema_version": 1, "theme": "dark"}))
            .expect("save settings");
        drop(first);

        let reopened = AtomicSettingsFile::new(path);
        let loaded = reopened
            .load_or_recover(&json!({"theme": "system"}))
            .expect("reload settings");
        assert_eq!(loaded.value["theme"], "dark");
        assert!(loaded.recovered_corrupt_path.is_none());
    }

    #[test]
    fn corrupt_settings_are_preserved_and_replaced_with_defaults() {
        let directory = tempfile::tempdir().expect("create settings directory");
        let path = directory.path().join("settings_store.json");
        fs::write(&path, b"{ definitely not json").expect("write corrupt settings");
        let store = AtomicSettingsFile::new(path.clone());
        let defaults = json!({"settings_schema_version": 1, "theme": "system"});

        let loaded = store.load_or_recover(&defaults).expect("recover settings");
        let backup = loaded
            .recovered_corrupt_path
            .expect("corrupt file should be preserved");
        assert_eq!(loaded.value, defaults);
        assert_eq!(
            fs::read(&backup).expect("read preserved corruption"),
            b"{ definitely not json"
        );

        let restarted = AtomicSettingsFile::new(path)
            .load_or_recover(&json!({}))
            .expect("load recovered settings after restart");
        assert_eq!(restarted.value, defaults);
    }

    #[test]
    fn repeated_atomic_updates_always_leave_valid_json() {
        let directory = tempfile::tempdir().expect("create settings directory");
        let path = directory.path().join("settings_store.json");
        let lock = Arc::new(Mutex::new(()));
        let store = AtomicSettingsFile::with_lock(path.clone(), Arc::clone(&lock));

        for sequence in 0..50 {
            store
                .save(&json!({"sequence": sequence}))
                .expect("save update");
            let bytes = fs::read(&path).expect("read update");
            serde_json::from_slice::<Value>(&bytes).expect("update is valid JSON");
        }
        let loaded = store
            .load_or_recover(&json!({}))
            .expect("load final update");
        assert_eq!(loaded.value["sequence"], 49);
    }

    #[test]
    fn valid_json_with_missing_envelope_converges_to_typed_defaults() {
        let directory = tempfile::tempdir().expect("create settings directory");
        let path = directory.path().join("settings_store.json");
        fs::write(&path, b"{}").expect("write empty envelope");
        let defaults = json!({"settings_schema_version": 1, "theme": "system"});

        let loaded = AtomicSettingsFile::new(path.clone())
            .load_or_recover(&defaults)
            .expect("load missing envelope");
        assert_eq!(loaded.value, defaults);

        let persisted: Value =
            serde_json::from_slice(&fs::read(path).expect("read converged settings"))
                .expect("parse converged settings");
        assert_eq!(persisted[SETTINGS_KEY], defaults);
    }
}

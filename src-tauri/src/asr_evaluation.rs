use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct CorpusManifest {
    pub schema_version: u32,
    pub name: String,
    pub source: String,
    pub license: String,
    pub thresholds: EvaluationThresholds,
    pub items: Vec<CorpusItem>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EvaluationThresholds {
    pub max_wer: f64,
    pub min_task_success: f64,
    pub max_p50_ms: u64,
    pub max_p95_ms: u64,
    pub max_idle_rss_bytes: u64,
    pub max_loaded_rss_bytes: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CorpusItem {
    pub id: String,
    pub audio: PathBuf,
    pub reference: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub required_terms: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ItemScore {
    pub id: String,
    pub audio: String,
    pub reference: String,
    pub hypothesis: String,
    pub raw_hypothesis: String,
    pub requested_language: String,
    pub effective_language: String,
    pub detected_language: Option<String>,
    pub raw_substitutions: usize,
    pub raw_deletions: usize,
    pub raw_insertions: usize,
    pub raw_wer: f64,
    pub corrected_substitutions: usize,
    pub corrected_deletions: usize,
    pub corrected_insertions: usize,
    pub reference_words: usize,
    pub corrected_wer: f64,
    pub task_success: bool,
    pub audio_duration_ms: u64,
    pub transcription_ms: u64,
    pub real_time_factor: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct EvaluationSummary {
    pub schema_version: u32,
    pub corpus_name: String,
    pub corpus_source: String,
    pub corpus_license: String,
    pub model_id: String,
    pub item_count: usize,
    pub raw_wer: f64,
    pub corrected_wer: f64,
    pub task_success: f64,
    pub latency_p50_ms: u64,
    pub latency_p95_ms: u64,
    pub resident_memory_before_load_bytes: Option<u64>,
    pub resident_memory_after_load_bytes: Option<u64>,
    pub resident_memory_after_evaluation_bytes: Option<u64>,
    pub network_denial: Option<NetworkDenialEvidence>,
    pub thresholds: EvaluationThresholdsResult,
    pub passed: bool,
    pub items: Vec<ItemScore>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NetworkDenialEvidence {
    pub target: String,
    pub denied: bool,
    pub error: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct EvaluationThresholdsResult {
    pub max_wer: f64,
    pub min_task_success: f64,
    pub max_p50_ms: u64,
    pub max_p95_ms: u64,
    pub max_idle_rss_bytes: u64,
    pub max_loaded_rss_bytes: u64,
    pub raw_wer_passed: bool,
    pub task_success_passed: bool,
    pub p50_passed: bool,
    pub p95_passed: bool,
    pub idle_rss_passed: bool,
    pub loaded_rss_passed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WordErrors {
    pub substitutions: usize,
    pub deletions: usize,
    pub insertions: usize,
    pub reference_words: usize,
}

impl WordErrors {
    pub fn total(self) -> usize {
        self.substitutions + self.deletions + self.insertions
    }

    pub fn wer(self) -> f64 {
        if self.reference_words == 0 {
            return if self.total() == 0 { 0.0 } else { 1.0 };
        }
        self.total() as f64 / self.reference_words as f64
    }
}

pub fn load_manifest(path: &Path) -> Result<CorpusManifest> {
    let bytes =
        std::fs::read(path).with_context(|| format!("read corpus manifest {}", path.display()))?;
    let manifest: CorpusManifest = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse corpus manifest {}", path.display()))?;
    if manifest.schema_version != 1 {
        bail!(
            "unsupported corpus manifest schema {}; expected 1",
            manifest.schema_version
        );
    }
    if manifest.items.is_empty() {
        bail!("corpus manifest has no items");
    }
    if !(0.0..=1.0).contains(&manifest.thresholds.max_wer)
        || !(0.0..=1.0).contains(&manifest.thresholds.min_task_success)
    {
        bail!("WER and task-success thresholds must be between 0 and 1");
    }
    if manifest.thresholds.max_p50_ms == 0
        || manifest.thresholds.max_p95_ms == 0
        || manifest.thresholds.max_idle_rss_bytes == 0
        || manifest.thresholds.max_loaded_rss_bytes == 0
    {
        bail!("latency and memory thresholds must be greater than zero");
    }
    let mut item_ids = HashSet::with_capacity(manifest.items.len());
    for item in &manifest.items {
        if item.id.trim().is_empty() || !item_ids.insert(item.id.as_str()) {
            bail!("corpus item IDs must be non-empty and unique");
        }
        if item.reference.trim().is_empty() {
            bail!("corpus item '{}' has an empty reference", item.id);
        }
        if item.required_terms.is_empty() {
            bail!(
                "corpus item '{}' must declare at least one required term",
                item.id
            );
        }
    }
    Ok(manifest)
}

pub fn normalize_words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|word| {
            let normalized: String = word
                .chars()
                .flat_map(char::to_lowercase)
                .filter(|character| character.is_alphanumeric() || *character == '\'')
                .collect();
            let normalized = normalized.trim_matches('\'');
            (!normalized.is_empty()).then(|| normalized.to_string())
        })
        .collect()
}

pub fn word_errors(reference: &str, hypothesis: &str) -> WordErrors {
    let reference = normalize_words(reference);
    let hypothesis = normalize_words(hypothesis);
    let width = hypothesis.len() + 1;
    let mut costs = vec![WordErrors::default(); (reference.len() + 1) * width];
    for row in 1..=reference.len() {
        costs[row * width] = WordErrors {
            deletions: row,
            reference_words: row,
            ..WordErrors::default()
        };
    }
    for (column, cost) in costs
        .iter_mut()
        .enumerate()
        .take(hypothesis.len() + 1)
        .skip(1)
    {
        *cost = WordErrors {
            insertions: column,
            ..WordErrors::default()
        };
    }

    for row in 1..=reference.len() {
        for column in 1..=hypothesis.len() {
            let diagonal = costs[(row - 1) * width + column - 1];
            let mut candidates = Vec::with_capacity(3);
            if reference[row - 1] == hypothesis[column - 1] {
                candidates.push(diagonal);
            } else {
                candidates.push(WordErrors {
                    substitutions: diagonal.substitutions + 1,
                    reference_words: row,
                    ..diagonal
                });
            }
            let above = costs[(row - 1) * width + column];
            candidates.push(WordErrors {
                deletions: above.deletions + 1,
                reference_words: row,
                ..above
            });
            let left = costs[row * width + column - 1];
            candidates.push(WordErrors {
                insertions: left.insertions + 1,
                reference_words: row,
                ..left
            });
            costs[row * width + column] = candidates
                .into_iter()
                .min_by_key(|errors| errors.total())
                .expect("three edit candidates");
        }
    }
    let mut result = costs[reference.len() * width + hypothesis.len()];
    result.reference_words = reference.len();
    result
}

pub fn required_terms_present(hypothesis: &str, required_terms: &[String]) -> bool {
    let hypothesis = normalize_words(hypothesis);
    required_terms.iter().all(|term| {
        let term = normalize_words(term);
        !term.is_empty()
            && hypothesis
                .windows(term.len())
                .any(|window| window == term.as_slice())
    })
}

pub fn percentile(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() as f64 * percentile).ceil() as usize).saturating_sub(1);
    sorted[rank.min(sorted.len() - 1)]
}

pub fn require_network_denied(target: &str) -> Result<NetworkDenialEvidence> {
    use std::net::{SocketAddr, TcpStream};
    use std::time::Duration;

    let address: SocketAddr = target
        .parse()
        .with_context(|| format!("network denial target must be a numeric IP:port: {target}"))?;
    match TcpStream::connect_timeout(&address, Duration::from_secs(3)) {
        Ok(stream) => {
            drop(stream);
            bail!("network denial probe unexpectedly connected to {target}")
        }
        Err(error) => Ok(NetworkDenialEvidence {
            target: target.to_string(),
            denied: true,
            error: error.to_string(),
        }),
    }
}

#[cfg(windows)]
pub fn resident_memory_bytes() -> Option<u64> {
    use std::mem::size_of;
    use windows::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    let mut counters = PROCESS_MEMORY_COUNTERS::default();
    // SAFETY: the pseudo-handle is valid for the current process, and the
    // writable counters buffer is exactly the size passed to the API.
    unsafe {
        if !K32GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
        .as_bool()
        {
            return None;
        }
    }
    Some(counters.WorkingSetSize as u64)
}

#[cfg(target_os = "macos")]
pub fn resident_memory_bytes() -> Option<u64> {
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::task::task_info;
    use mach2::task_info::{task_info_t, MACH_TASK_BASIC_INFO};
    use mach2::time_value::time_value_t;
    use mach2::traps::mach_task_self;
    use mach2::vm_types::{integer_t, mach_vm_size_t, natural_t};
    use std::mem::size_of;

    #[repr(C)]
    #[derive(Default)]
    struct MachTaskBasicInfo {
        virtual_size: mach_vm_size_t,
        resident_size: mach_vm_size_t,
        resident_size_max: mach_vm_size_t,
        user_time: time_value_t,
        system_time: time_value_t,
        policy: natural_t,
        suspend_count: integer_t,
    }

    let mut info = MachTaskBasicInfo::default();
    let mut count = (size_of::<MachTaskBasicInfo>() / size_of::<integer_t>()) as u32;
    // SAFETY: mach_task_self() is a valid send right for this process. `info`
    // has the layout of mach_task_basic_info and `count` reports its capacity
    // in the integer_t units required by task_info.
    let result = unsafe {
        task_info(
            mach_task_self(),
            MACH_TASK_BASIC_INFO,
            (&mut info as *mut MachTaskBasicInfo).cast::<integer_t>() as task_info_t,
            &mut count,
        )
    };
    (result == KERN_SUCCESS).then_some(info.resident_size)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn resident_memory_bytes() -> Option<u64> {
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .ok()?;
    let kib = String::from_utf8(output.stdout)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()?;
    kib.checked_mul(1024)
}

#[cfg(not(any(unix, windows)))]
pub fn resident_memory_bytes() -> Option<u64> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_error_counts_substitution_deletion_and_insertion() {
        let substitution = word_errors("one two", "one too");
        assert_eq!(substitution.substitutions, 1);
        assert_eq!(substitution.total(), 1);
        assert_eq!(substitution.wer(), 0.5);

        let deletion = word_errors("one two", "one");
        assert_eq!(deletion.deletions, 1);
        assert_eq!(deletion.total(), 1);

        let insertion = word_errors("one", "one two");
        assert_eq!(insertion.insertions, 1);
        assert_eq!(insertion.total(), 1);
    }

    #[test]
    fn normalization_is_case_and_punctuation_insensitive() {
        assert_eq!(normalize_words("'Hello,' WORLD!"), vec!["hello", "world"]);
        assert_eq!(word_errors("Hello world", "hello, WORLD!").wer(), 0.0);
    }

    #[test]
    fn required_terms_are_phrase_aware() {
        let terms = vec!["marigold begins".to_string(), "sentence".to_string()];
        assert!(required_terms_present(
            "Marigold begins this sentence.",
            &terms
        ));
        assert!(!required_terms_present(
            "Marigold ends this sentence",
            &terms
        ));
        assert!(!required_terms_present(
            "A partial match must not pass",
            &["art".to_string()]
        ));
    }

    #[test]
    fn percentile_uses_nearest_rank() {
        assert_eq!(percentile(&[10, 40, 20, 30], 0.5), 20);
        assert_eq!(percentile(&[10, 40, 20, 30], 0.95), 40);
    }

    #[test]
    fn network_denial_probe_rejects_non_numeric_targets() {
        let error =
            require_network_denied("example.com:443").expect_err("numeric address required");
        assert!(error.to_string().contains("numeric IP:port"));
    }
}

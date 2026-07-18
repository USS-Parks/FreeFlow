use anyhow::Result;
use hound::{WavReader, WavSpec, WavWriter};
use log::debug;
use std::path::{Path, PathBuf};
use tempfile::Builder;

/// Read a WAV file and return normalised f32 samples.
pub fn read_wav_samples<P: AsRef<Path>>(file_path: P) -> Result<Vec<f32>> {
    let reader = WavReader::open(file_path.as_ref())?;
    let samples = reader
        .into_samples::<i16>()
        .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
        .collect::<Result<Vec<f32>, _>>()?;
    Ok(samples)
}

/// Verify a WAV file by reading it back and checking the sample count.
pub fn verify_wav_file<P: AsRef<Path>>(file_path: P, expected_samples: usize) -> Result<()> {
    let reader = WavReader::open(file_path.as_ref())?;
    let actual_samples = reader.len() as usize;
    if actual_samples != expected_samples {
        anyhow::bail!(
            "WAV sample count mismatch: expected {}, got {}",
            expected_samples,
            actual_samples
        );
    }
    Ok(())
}

/// Save audio samples as a WAV file without ever exposing a partial recording.
///
/// The file is finalized and flushed in the destination directory before an
/// atomic, no-clobber rename makes it visible. A crash during the write leaves
/// at most a hidden `.part` file, never a corrupt retryable WAV at `file_path`.
pub fn save_wav_file<P: AsRef<Path>>(file_path: P, samples: &[f32]) -> Result<()> {
    let file_path = file_path.as_ref();
    let parent = file_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("WAV destination has no parent directory"))?;
    let spec = WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut temporary = Builder::new()
        .prefix(".freeflow-capture-")
        .suffix(".part")
        .tempfile_in(parent)?;

    let mut writer = WavWriter::new(temporary.as_file_mut(), spec)?;

    // Convert f32 samples to i16 for WAV
    for sample in samples {
        let sample_i16 = (sample * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(file_path)?;
    debug!("Saved WAV file atomically: {:?}", file_path);
    Ok(())
}

/// Return finalized FreeFlow WAVs that are not yet referenced by history.
/// Hidden `.part` files are deliberately excluded.
pub fn retryable_wav_candidates(
    recordings_dir: &Path,
    referenced_names: &std::collections::HashSet<String>,
) -> Result<Vec<PathBuf>> {
    let mut candidates = Vec::new();
    for entry in std::fs::read_dir(recordings_dir)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with("freeflow-")
            && name.ends_with(".wav")
            && !referenced_names.contains(name)
            && verify_nonempty_wav(&path).is_ok()
        {
            candidates.push(path);
        }
    }
    candidates.sort();
    Ok(candidates)
}

fn verify_nonempty_wav(path: &Path) -> Result<()> {
    let reader = WavReader::open(path)?;
    if reader.len() == 0 {
        anyhow::bail!("WAV contains no samples");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{retryable_wav_candidates, save_wav_file, verify_wav_file};
    use std::collections::HashSet;

    #[test]
    fn atomic_save_publishes_only_the_complete_wav() {
        let directory = tempfile::tempdir().expect("tempdir");
        let destination = directory.path().join("freeflow-1.wav");
        let samples = vec![0.25; 1_600];

        save_wav_file(&destination, &samples).expect("atomic save");
        verify_wav_file(&destination, samples.len()).expect("complete wav");

        let names: Vec<String> = std::fs::read_dir(directory.path())
            .expect("read dir")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        assert_eq!(names, vec!["freeflow-1.wav"]);
    }

    #[test]
    fn atomic_save_refuses_to_replace_an_existing_capture() {
        let directory = tempfile::tempdir().expect("tempdir");
        let destination = directory.path().join("freeflow-1.wav");
        save_wav_file(&destination, &[0.1; 800]).expect("first save");

        assert!(save_wav_file(&destination, &[0.9; 1_600]).is_err());
        verify_wav_file(&destination, 800).expect("original capture remains");
    }

    #[test]
    fn retry_candidates_exclude_referenced_partial_and_invalid_files() {
        let directory = tempfile::tempdir().expect("tempdir");
        save_wav_file(directory.path().join("freeflow-1.wav"), &[0.1; 800]).expect("candidate");
        save_wav_file(directory.path().join("freeflow-2.wav"), &[0.1; 800]).expect("referenced");
        std::fs::write(
            directory.path().join(".freeflow-capture-stale.part"),
            b"partial",
        )
        .expect("partial");
        std::fs::write(directory.path().join("freeflow-invalid.wav"), b"invalid").expect("invalid");

        let referenced = HashSet::from(["freeflow-2.wav".to_string()]);
        let candidates =
            retryable_wav_candidates(directory.path(), &referenced).expect("candidate scan");

        assert_eq!(candidates, vec![directory.path().join("freeflow-1.wav")]);
    }
}

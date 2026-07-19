use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone, Default)]
#[command(name = "freeflow", about = "FreeFlow - local speech to text")]
pub struct CliArgs {
    /// Start with the main window hidden
    #[arg(long)]
    pub start_hidden: bool,

    /// Disable the system tray icon
    #[arg(long)]
    pub no_tray: bool,

    /// Toggle transcription on/off (sent to running instance)
    #[arg(long)]
    pub toggle_transcription: bool,

    /// Toggle transcription with post-processing on/off (sent to running instance)
    #[arg(long)]
    pub toggle_post_process: bool,

    /// Cancel the current operation (sent to running instance)
    #[arg(long)]
    pub cancel: bool,

    /// Enable debug mode with verbose logging
    #[arg(long)]
    pub debug: bool,

    /// Transcribe this WAV (16 kHz mono) headlessly and exit. Runs the same
    /// batch transcription path as the app — no mic, no VAD, no download
    /// (the model must already be installed).
    #[arg(short = 'f', long, value_name = "WAV")]
    pub transcribe_file: Option<PathBuf>,

    /// Evaluate a versioned public/owned corpus manifest with one model load,
    /// emitting WER, task-success, latency, language, and memory evidence.
    #[arg(long, value_name = "JSON")]
    pub evaluate_corpus: Option<PathBuf>,

    /// Before corpus evaluation, require a TCP connection attempt to this
    /// numeric IP:port to fail. Used with an OS-enforced outbound deny rule to
    /// retain same-process zero-network evidence.
    #[arg(long, value_name = "IP:PORT", requires = "evaluate_corpus")]
    pub require_network_denied: Option<String>,

    /// Verify and install a local model artifact through the approved manifest.
    /// Requires --model and --accept-model-manifest-digest. No network request
    /// is made by this command.
    #[arg(
        long,
        value_name = "FILE",
        requires = "model",
        requires = "accept_model_manifest_digest"
    )]
    pub install_model_file: Option<PathBuf>,

    /// Exact digest of the approved model manifest whose source, hash, size,
    /// licenses, and destination were reviewed before --install-model-file.
    #[arg(long, value_name = "SHA256")]
    pub accept_model_manifest_digest: Option<String>,

    /// Model id to install or load for a headless operation (default for
    /// transcription/evaluation: the selected model).
    #[arg(long)]
    pub model: Option<String>,

    /// Hard-select the compute device for --transcribe-file or
    /// --evaluate-corpus by its registry index (see --list-devices). Omit to
    /// use the persisted accelerator setting. transcribe-cpp models only.
    #[arg(long, value_name = "N")]
    pub device_index: Option<usize>,

    /// List the transcribe-cpp compute devices (with indices) and exit.
    #[arg(long)]
    pub list_devices: bool,

    /// List the available models (with ids) and exit. Pass an id to --model.
    /// Honors --json for machine-readable output.
    #[arg(long)]
    pub list_models: bool,

    /// Record from the configured microphone for N seconds, verify non-empty
    /// 16 kHz capture, exercise cancellation, and exit. No ASR model is loaded.
    #[arg(long, value_name = "SECONDS")]
    pub verify_audio: Option<u64>,

    /// Repeat the transcription or audio-capture verification N times.
    #[arg(long, value_name = "N")]
    pub repeat: Option<usize>,

    /// Emit headless results as JSON. Corpus evaluation is always JSON.
    #[arg(long)]
    pub json: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_model_install_requires_explicit_manifest_acceptance() {
        assert!(CliArgs::try_parse_from([
            "freeflow",
            "--install-model-file",
            "model.gguf",
            "--model",
            "approved-model"
        ])
        .is_err());
        assert!(CliArgs::try_parse_from([
            "freeflow",
            "--install-model-file",
            "model.gguf",
            "--model",
            "approved-model",
            "--accept-model-manifest-digest",
            "digest"
        ])
        .is_ok());
    }

    #[test]
    fn network_denial_probe_is_scoped_to_corpus_evaluation() {
        assert!(
            CliArgs::try_parse_from(["freeflow", "--require-network-denied", "1.1.1.1:443"])
                .is_err()
        );
        assert!(CliArgs::try_parse_from([
            "freeflow",
            "--evaluate-corpus",
            "corpus.json",
            "--require-network-denied",
            "1.1.1.1:443"
        ])
        .is_ok());
    }
}

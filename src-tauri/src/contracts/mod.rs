//! Versioned service contracts for FreeFlow engine and platform adapters.
//!
//! Product orchestration depends on these interfaces rather than concrete
//! model, audio, insertion, or operating-system implementations. Contract
//! major versions are breaking; minor versions add backward-compatible
//! capabilities.

use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const CONTRACT_VERSION_V1: ContractVersion = ContractVersion::new(1, 0);
const CONTROL_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContractVersion {
    pub major: u16,
    pub minor: u16,
}

impl ContractVersion {
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }

    pub const fn supports(self, required: Self) -> bool {
        self.major == required.major && self.minor >= required.minor
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContractError {
    Cancelled,
    TimedOut,
    InvalidInput(String),
    Unavailable(String),
    Failed(String),
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cancelled => formatter.write_str("operation cancelled"),
            Self::TimedOut => formatter.write_str("operation timed out"),
            Self::InvalidInput(message) => write!(formatter, "invalid input: {message}"),
            Self::Unavailable(message) => write!(formatter, "service unavailable: {message}"),
            Self::Failed(message) => write!(formatter, "operation failed: {message}"),
        }
    }
}

impl Error for ContractError {}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractFuture<'a, T> = Pin<Box<dyn Future<Output = ContractResult<T>> + Send + 'a>>;

#[derive(Clone, Debug)]
pub struct OperationControl {
    cancelled: Arc<AtomicBool>,
    deadline: Instant,
}

impl OperationControl {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            deadline: Instant::now() + timeout,
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    pub fn checkpoint(&self) -> ContractResult<()> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(ContractError::Cancelled);
        }
        if Instant::now() >= self.deadline {
            return Err(ContractError::TimedOut);
        }
        Ok(())
    }

    fn remaining(&self) -> ContractResult<Duration> {
        self.checkpoint()?;
        Ok(self.deadline.saturating_duration_since(Instant::now()))
    }
}

/// Enforce cancellation and deadline semantics around any adapter future,
/// including adapters that have not yet reached an internal checkpoint.
pub async fn enforce_operation<T, F>(control: &OperationControl, operation: F) -> ContractResult<T>
where
    F: Future<Output = ContractResult<T>> + Send,
{
    let mut operation = Box::pin(operation);
    loop {
        let remaining = control.remaining()?;
        let poll_for = remaining.min(CONTROL_POLL_INTERVAL);
        match tokio::time::timeout(poll_for, operation.as_mut()).await {
            Ok(result) => return result,
            Err(_) => control.checkpoint()?,
        }
    }
}

pub trait VersionedContract: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    fn contract_version(&self) -> ContractVersion {
        CONTRACT_VERSION_V1
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioCaptureRequest {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub max_duration: Duration,
    pub device_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate_hz: u32,
    pub channels: u16,
}

pub trait AudioCapture: VersionedContract {
    fn capture<'a>(
        &'a self,
        request: AudioCaptureRequest,
        control: OperationControl,
    ) -> ContractFuture<'a, AudioBuffer>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct AsrRequest {
    pub audio: AudioBuffer,
    pub language: Option<String>,
    pub initial_prompt: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transcript {
    pub text: String,
    pub detected_language: Option<String>,
    pub confidence: Option<f32>,
}

pub trait AsrEngine: VersionedContract {
    fn transcribe<'a>(
        &'a self,
        request: AsrRequest,
        control: OperationControl,
    ) -> ContractFuture<'a, Transcript>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostProcessRequest {
    pub raw_text: String,
    pub instruction: String,
    pub locale: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostProcessOutput {
    pub text: String,
    pub changed: bool,
}

pub trait PostProcessor: VersionedContract {
    fn process<'a>(
        &'a self,
        request: PostProcessRequest,
        control: OperationControl,
    ) -> ContractFuture<'a, PostProcessOutput>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsertionRequest {
    pub text: String,
    pub preserve_clipboard: bool,
    pub submit_after_insert: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InsertionMethod {
    Direct,
    Clipboard,
    ManualCopy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InsertionOutcome {
    pub method: InsertionMethod,
    pub inserted: bool,
}

pub trait TextInserter: VersionedContract {
    fn insert<'a>(
        &'a self,
        request: InsertionRequest,
        control: OperationControl,
    ) -> ContractFuture<'a, InsertionOutcome>;
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlatformContext {
    pub application_id: Option<String>,
    pub window_title: Option<String>,
    pub selected_text: Option<String>,
    pub secure_field: bool,
}

pub trait PlatformContextProvider: VersionedContract {
    fn snapshot<'a>(&'a self, control: OperationControl) -> ContractFuture<'a, PlatformContext>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransformRequest {
    pub text: String,
    pub locale: Option<String>,
}

pub trait TextTransform: VersionedContract {
    fn transform<'a>(
        &'a self,
        request: TransformRequest,
        control: OperationControl,
    ) -> ContractFuture<'a, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PendingAsr;

    impl VersionedContract for PendingAsr {
        fn adapter_id(&self) -> &'static str {
            "test.pending-asr"
        }
    }

    impl AsrEngine for PendingAsr {
        fn transcribe<'a>(
            &'a self,
            _request: AsrRequest,
            _control: OperationControl,
        ) -> ContractFuture<'a, Transcript> {
            Box::pin(std::future::pending())
        }
    }

    fn empty_asr_request() -> AsrRequest {
        AsrRequest {
            audio: AudioBuffer {
                samples: Vec::new(),
                sample_rate_hz: 16_000,
                channels: 1,
            },
            language: None,
            initial_prompt: None,
        }
    }

    #[test]
    fn contract_version_requires_matching_major_and_sufficient_minor() {
        assert!(ContractVersion::new(1, 2).supports(ContractVersion::new(1, 0)));
        assert!(!ContractVersion::new(1, 0).supports(ContractVersion::new(1, 1)));
        assert!(!ContractVersion::new(2, 0).supports(ContractVersion::new(1, 0)));
    }

    #[test]
    fn operation_wrapper_observes_cancellation() {
        let control = OperationControl::with_timeout(Duration::from_secs(1));
        let cancel_handle = control.clone();
        let canceller = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(25));
            cancel_handle.cancel();
        });

        let engine: Box<dyn AsrEngine> = Box::new(PendingAsr);
        let result = tauri::async_runtime::block_on(enforce_operation(
            &control,
            engine.transcribe(empty_asr_request(), control.clone()),
        ));
        canceller.join().expect("cancellation thread should finish");
        assert_eq!(result, Err(ContractError::Cancelled));
    }

    #[test]
    fn operation_wrapper_enforces_timeout() {
        let control = OperationControl::with_timeout(Duration::from_millis(25));
        let engine: Box<dyn AsrEngine> = Box::new(PendingAsr);
        let result = tauri::async_runtime::block_on(enforce_operation(
            &control,
            engine.transcribe(empty_asr_request(), control.clone()),
        ));
        assert_eq!(result, Err(ContractError::TimedOut));
    }
}

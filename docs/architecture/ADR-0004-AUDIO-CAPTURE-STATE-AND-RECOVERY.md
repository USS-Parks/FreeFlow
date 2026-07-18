# ADR-0004: Audio capture state and recovery

Status: FF-V2 gate candidate

Date: 2026-07-18

## Context

The imported Handy capture path already provided CPAL input, 16 kHz mono
resampling, Silero VAD smoothing, shortcut handling, live levels, and batch or
streaming transcription. FF-V2 found three boundary failures that prevented a
truthful vertical slice:

- a disconnected named microphone silently fell back to the system default;
- lifecycle state was split between the recorder and coordinator with no typed
  diagnostic snapshot for the UI;
- a WAV was written directly and received its history row only after ASR, so a
  crash could leave a corrupt or undiscoverable recording.

The clean-room foundation also referenced `silero_vad_v4.onnx` without shipping
the weight, which made live capture fail before a microphone could open.

## Decision

### Device truth and diagnostics

An explicit device name resolves only to that device. If it is absent, capture
returns `selected_device_missing`; it never records from another microphone.
The typed diagnostics command reports the requested and resolved device,
enumerated inputs, permission state, open-stream state, recording state, and an
actionable status. Device changes are rejected while recording and rolled back
if reopening fails.

### Dictation lifecycle

The transcription coordinator publishes a monotonically sequenced typed state:
`idle`, `starting`, `recording`, `processing`, or `cancelling`. Push-to-talk,
toggle, duplicate press, processing exclusion, and cancellation remain
serialized on the coordinator thread. The `starting` transition records queue
latency, and the backend separately logs/emits the time at which recording UI
feedback was dispatched.

### Retry-safe capture persistence

Stopped PCM is written to a same-directory temporary file, finalized, flushed,
verified, and atomically renamed without clobbering an existing capture. An
empty retryable history row is committed before ASR begins and updated after a
successful transcript. On startup, valid finalized `freeflow-*.wav` files with
no database row are recovered as retryable entries; incomplete `.part` files
are removed.

Cancellation while recording or before transcription completes discards the
audio and pending row. If final output has already committed, cancellation at
the insertion boundary prevents paste but keeps completed history.

### VAD artifact

FreeFlow bundles the exact Silero V4 ONNX fixture used by the pinned `vad-rs`
revision. Its size, Git blob, SHA-256, source revision, path, retrieval date,
and MIT license are recorded in `NOTICE.md` and
`src-tauri/resources/models/silero-vad-v4.json`. A Rust test rejects artifact
drift.

## Consequences

- Selected-device behavior is truthful through disconnect/reconnect cycles.
- A stopped recording is recoverable before ASR begins, without exposing a
  partial final WAV.
- The UI and live verifier can inspect lifecycle and microphone state without
  bypassing the Rust service boundary.
- Bundling adds 1,807,522 bytes and the Silero MIT notice.
- FF-V2 remains incomplete until the physical Windows/macOS live gate in
  `docs/verification/FF-V2-LIVE-GATE.md` passes.

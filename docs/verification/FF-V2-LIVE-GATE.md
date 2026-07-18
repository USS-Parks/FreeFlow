# FF-V2 live microphone gate

Status: **In progress — Windows default-device capture proven; interactive and
macOS evidence pending**

Date: 2026-07-18

## Automated and headless commands

```text
cargo test --manifest-path src-tauri/Cargo.toml
freeflow --verify-audio 1 --repeat 50 --json
```

The verifier uses the production `AudioRecordingManager`, configured device,
CPAL stream, resampler, recorder command channel, and cancellation path. It
does not load an ASR model, does not persist the probe audio, and does not
claim to measure visible-feedback latency. Its `microphone_ready_*` values
measure the distinct on-demand stream-open path. The 150 ms feedback gate is
measured from shortcut receipt through `recording-feedback-metric` dispatch in
the interactive application.

## Windows evidence captured this session

- Host: interactive Windows x64 host, configured system-default microphone.
- Silero V4: 1,807,522 bytes; SHA-256
  `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`.
- Result: 50/50 open-record-stop cycles returned audio.
- Samples per one-second cycle: minimum 16,320; maximum 20,000 at 16 kHz mono
  after resampling/padding behavior.
- Every cycle contained non-zero input; observed peaks ranged from
  `0.0008545244` to `0.0027551006` on the quiet test microphone.
- Cancellation probe returned the manager to idle.
- On-demand microphone-ready latency was 504.72 ms p95. This is recorded as a
  performance finding, not mislabeled as the UI feedback threshold.

## Windows continuation evidence after verifier correction

- The corrected verifier completed another 50/50 configured-default microphone
  cycles from the current FF-V2 source and exited successfully.
- Every cycle contained non-zero input. Sample counts remained 16,320–20,000
  at 16 kHz mono and cancellation again returned the manager to idle.
- On-demand microphone-ready latency was 164.21 ms p95. The JSON reports this
  as `microphone_ready_p95_ms` and explicitly reports
  `feedback_gate_measured: false`; it no longer fails or passes the distinct
  150 ms visible-feedback gate from this headless measurement.
- The interactive application gate was not rerun during this continuation so
  the user's foreground Windows session remained available. No shortcut,
  first-word, hot-plug, contention, sleep/wake, or visible-feedback pass is
  inferred from the headless result.

## Remaining required live matrix

For each Windows and macOS host, retain the host manifest, application log, and
JSON verifier output.

1. Run 50 hold/release cycles, 20 toggle cycles, Escape during recording and
   processing, focus changes, and sleep/wake. Confirm one transition per input,
   no stuck recording, and processing exclusion.
2. Record the `recording-feedback-metric` event/log for every activation.
   Require visible feedback within 150 ms p95.
3. Speak `Marigold begins this sentence` immediately after activation in 50
   cycles. Require `Marigold` in every valid capture/transcript.
4. Select a connected external microphone and prove diagnostics resolves its
   exact name. Disconnect it while idle and while the stream is warm; require
   `selected_device_missing` with no fallback capture. Reconnect it and require
   the same selection to recover without restarting FreeFlow.
5. Let Zoom, Teams, and a browser acquire/release the microphone. Require the
   next capture to succeed or report blocked, busy, missing, or disconnected
   state truthfully.
6. Repeat on macOS 12+ with microphone permission deny/regrant evidence.

FF-V2 must not be marked complete until all six items pass on both platforms.

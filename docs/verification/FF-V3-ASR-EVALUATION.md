# FF-V3 local ASR evaluation gate

Status: **Gate implementation in progress; no platform pass is recorded yet**

This document freezes the FF-V3 quality and resource thresholds before the
first retained corpus benchmark. It supplements the release-level constraints
in `docs/product/BEHAVIOR-PARITY-MATRIX.md`; it does not relax them.

## Selected default

The FF-V3 default candidate is `parakeet-unified-en-0.6b-q8_0`, revision
`17cf15519695fed7891fe1e81bfc512f3a58cc7b`, SHA-256
`4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`.
The artifact remains an explicit FF-V1 user installation and is never bundled.

## Frozen thresholds

Each platform must pass the same manifest-bound thresholds:

| Measure               |                   Threshold | Scoring rule                                                                                                                                                                                                                  |
| --------------------- | --------------------------: | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Raw ASR WER           |                at most 0.12 | Aggregate raw-engine word edits divided by aggregate reference words; case and punctuation are ignored. Corrected ASR WER is also reported but cannot rescue a raw-WER failure. No LLM or optional post-processing is scored. |
| Semantic task success |               at least 0.95 | Every independently declared required term or phrase is present in the corrected ASR result.                                                                                                                                  |
| ASR latency p50       |            at most 2,500 ms | Nearest-rank inference time after a single cold model load.                                                                                                                                                                   |
| ASR latency p95       |            at most 6,000 ms | Nearest-rank inference time after a single cold model load.                                                                                                                                                                   |
| Idle/headless RSS     |   at most 314,572,800 bytes | Resident memory immediately before model load.                                                                                                                                                                                |
| Loaded/evaluating RSS | at most 3,221,225,472 bytes | Resident memory after model load and after the final evaluation item.                                                                                                                                                         |

The loaded limit is a bounded compatibility ceiling for the declared reference
hosts, not a claim that the model itself consumes 3 GiB. Retained results report
all three RSS snapshots so later model candidates can be compared directly.

## Corpus contract

The gate requires both of these sets on each reference platform:

1. A reproducible public English corpus subset with source revision or archive
   hash, license, exact item IDs, and unmodified references.
2. A consented, project-owned dictation set covering the first word, names,
   numbers, negation, paths/identifiers, pauses, and at least one noisy sample.

Every input must be 16 kHz mono 16-bit PCM WAV. Corpus audio is not committed
unless its license and consent permit redistribution. A retained manifest can
therefore reference a local evidence directory while still recording hashes in
the associated evidence bundle.

The manifest schema is version 1. Relative audio paths resolve from the
manifest directory. Required threshold fields are `max_wer`,
`min_task_success`, `max_p50_ms`, `max_p95_ms`, `max_idle_rss_bytes`, and
`max_loaded_rss_bytes`.

## Headless procedure

Run the production engine path without opening a window:

```text
freeflow --evaluate-corpus <manifest.json> --model parakeet-unified-en-0.6b-q8_0 --json
```

The command loads the selected installed model once, transcribes each item,
records raw and corrected engine text, requested/effective/detected language,
audio duration, inference time, real-time factor, edit counts, semantic result,
percentiles, and memory snapshots. Exit code 0 means every frozen threshold
passed; 1 means a runtime or threshold failure; 2 means invalid corpus input.

## Offline proof

After the FF-V1 model is installed, deny outbound traffic for the process before
launching the evaluator. Retain the deny rule or process-level trace together
with the JSON result. The evaluator has no download path and a missing model is
a hard failure. A mock-only or source-inspection claim is not sufficient for the
zero-network gate.

## Completion rule

FF-V3 remains open until public and owned results, platform manifests, model
identity/hash, memory measurements, and zero-network evidence pass on Windows
and macOS. The user explicitly deferred FF-V2's remaining live work to a
separate Mac; that deferral does not convert FF-V2 into a pass and does not
waive any FF-V3 platform evidence.

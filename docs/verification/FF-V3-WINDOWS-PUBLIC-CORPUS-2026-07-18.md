# FF-V3 Windows public-corpus evidence — 2026-07-18

Status: **Windows public-corpus threshold pass; FF-V3 remains open**

## Revision and host

- Starting revision: `2361db37020e779865d50fc9886cff17e3790677`
- Branch: `codex/ff-v3-local-asr`
- OS: Microsoft Windows NT `10.0.26200.0`
- CPU: AMD Ryzen 7 5800H with Radeon Graphics, 16 logical processors
- Selected backend: `Vulkan0`
- GPU: NVIDIA GeForce RTX 3050 Ti Laptop GPU, 4,096 MiB, driver `546.30`
- Release executable: 43,565,568 bytes, SHA-256
  `7f2d1d1dc7d846d60106e05583b895bc168617e5eba97727aaee5fd65bab629c`

## Model identity

- Model ID: `parakeet-unified-en-0.6b-q8_0`
- Artifact revision: `17cf15519695fed7891fe1e81bfc512f3a58cc7b`
- Artifact size: 731,357,568 bytes
- Artifact SHA-256:
  `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`
- Manifest digest:
  `0c6a40bac30ebe258d963f76e98a49b078d1a1dd426b4d4d530c6e6bf88a7186`
- Cold load reported by the production manager: 1,274 ms
- Language capability: English-only; requested and effective language were
  `en`. The engine truthfully reported no detected-language value because this
  model does not expose language identification.

## Public corpus

The evaluator used a deterministic 20-speaker subset of OpenSLR SLR12
LibriSpeech `test-clean`, totaling 210.5 seconds of 16 kHz mono PCM. The source
archive was 346,663,984 bytes.

- Official archive MD5: `32fa31d27d2e1cad72775fee3f4849a9`
- Verified local archive MD5: `32fa31d27d2e1cad72775fee3f4849a9`
- Verified local archive SHA-256:
  `39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23`
- License: Creative Commons Attribution 4.0 International
- Input manifest:
  `docs/verification/corpora/ff-v3-librispeech-test-clean-20.json`
- Complete machine-readable result:
  `docs/verification/evidence/ff-v3/windows-public-corpus-2026-07-18.json`

The official Xiph FLAC 1.5.0 Windows decoder converted the selected source
files to WAV. Its downloaded ZIP matched the published SHA-256
`53f1500f0d6e7c61379d7fee50d4a9f7f504c650009506d9ba015530d76c0dde`.
Neither the 346 MB source archive, the 731 MB model, nor generated WAV files are
committed.

## Result

| Measure               |            Frozen threshold |          Observed | Result        |
| --------------------- | --------------------------: | ----------------: | ------------- |
| Raw WER               |                at most 0.12 |          0.013011 | Pass          |
| Corrected WER         |                    reported |          0.013011 | Informational |
| Semantic task success |               at least 0.95 |      0.95 (19/20) | Pass          |
| ASR latency p50       |            at most 2,500 ms |            146 ms | Pass          |
| ASR latency p95       |            at most 6,000 ms |            332 ms | Pass          |
| Idle/headless RSS     |   at most 314,572,800 bytes |  52,613,120 bytes | Pass          |
| RSS after model load  | at most 3,221,225,472 bytes | 120,995,840 bytes | Pass          |
| RSS after evaluation  | at most 3,221,225,472 bytes | 178,192,384 bytes | Pass          |

Raw and corrected results were identical because the isolated portable settings
had no custom dictionary. The sole semantic miss was the reference spelling
`endeavour` versus the model output `endeavor`; the WER scorer intentionally
retains lexical spelling differences.

## Network posture and open gate items

The headless evaluator contains no model-install or download path, and the run
forced `HTTP_PROXY`, `HTTPS_PROXY`, and `ALL_PROXY` to the closed local endpoint
`127.0.0.1:9`. That is useful negative-path evidence but is **not** recorded as
the required zero-network pass. A temporary program-scoped Windows Firewall
rule was attempted before evaluation and Windows rejected it with `Access is
denied` because this session lacks administrator rights.

FF-V3 therefore remains open for:

1. a consented project-owned dictation corpus on Windows;
2. Windows process-level blocked-network or packet-trace evidence with suitable
   privileges;
3. the same public/owned corpus, quality, latency, memory, and blocked-network
   evidence on the macOS reference host; and
4. any additional model-matrix candidate after it receives an FF-V1-compliant
   immutable manifest and explicit installation consent.

No FF-V4 work is authorized or started by this evidence.

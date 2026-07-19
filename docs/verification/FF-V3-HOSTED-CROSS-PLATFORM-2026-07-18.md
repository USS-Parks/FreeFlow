# FF-V3 hosted cross-platform evidence — 2026-07-18

Status: **Passed**

GitHub Actions run [29672022926](https://github.com/USS-Parks/FreeFlow/actions/runs/29672022926) passed the revised FF-V3 gate on Windows x86_64, Apple Silicon macOS, and Intel macOS at exact commit `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5`. Normal main CI run [29672015509](https://github.com/USS-Parks/FreeFlow/actions/runs/29672015509) also passed that commit. The frozen thresholds were not changed after observing results.

## Frozen identity

- Model: `parakeet-unified-en-0.6b-q8_0`
- Model SHA-256: `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`
- Manifest digest: `0c6a40bac30ebe258d963f76e98a49b078d1a1dd426b4d4d530c6e6bf88a7186`
- Corpus: deterministic 20-speaker LibriSpeech `test-clean` subset from archive SHA-256 `39fde525e59672dc6d1551919b1478f724438a95aa55f874b576be21967e6c23`
- Thresholds: WER `<= 0.12`; task success `>= 0.95`; p50 `<= 2500 ms`; p95 `<= 6000 ms`; idle RSS `<= 314572800`; loaded/evaluation RSS `<= 3221225472`; numeric TCP connection denied by real OS isolation.

## Results

| Platform                        |      WER | Task success |      p50 / p95 |            RSS before / loaded / evaluated | Network denial                                                     | Result |
| ------------------------------- | -------: | -----------: | -------------: | -----------------------------------------: | ------------------------------------------------------------------ | ------ |
| Windows x86_64 (`windows-2022`) | 0.013011 |         0.95 | 1665 / 4170 ms | 15,843,328 / 1,086,214,144 / 1,284,997,120 | Program-scoped outbound firewall; probe failed with OS error 10013 | Pass   |
| macOS arm64 (`macos-15`)        | 0.013011 |         0.95 |  690 / 1666 ms | 74,448,896 / 1,193,328,640 / 1,403,895,808 | `sandbox-exec` deny-network profile; probe failed with OS error 1  | Pass   |
| macOS x86_64 (`macos-15-intel`) | 0.013011 |         0.95 | 1588 / 3779 ms | 57,778,176 / 1,171,804,160 / 1,351,737,344 | `sandbox-exec` deny-network profile; probe failed with OS error 1  | Pass   |

The evaluator ran after a manifest-consent-bound local install and has no download path. Each job retained its JSON result, install receipt, host identity, executable hash, isolation evidence, and stderr. Artifact names and GitHub artifact digests are recorded in [`hosted-cross-platform-2026-07-18.json`](evidence/ff-v3/hosted-cross-platform-2026-07-18.json).

## Prior-run history

The completion claim does not hide earlier failures:

- Run `29669509926` produced a Windows pass, but macOS RSS collection used an external `ps` process that the network-denying sandbox blocked. That was an evidence-collector defect, not a macOS pass.
- Run `29670237557` passed both macOS architectures after native RSS collection replaced `ps`; Windows missed the frozen p95 threshold at `6334 ms`, then `6188 ms` on the retained rerun. Neither attempt was treated as a pass.
- Run `29671820518` stopped during environment provisioning because `windows-2022` does not provide `winget`. No product result was inferred.
- Commit `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5` replaced the package-manager dependency with the official Vulkan SDK `1.4.350.0` installer, exact SHA-256 `855b27ba05d2d8119c5114c5d4ff870ca38f2c632b11e1bb9923b9b7e6ecfe7b`, and official unattended switches. Run `29672022926` then passed all three platforms.

## Scope boundary

By explicit user rescope, the consented project-owned dictation corpus remains mandatory at the later release gate. It is deferred, not waived or replaced. FF-V2's separate live matrix is also still deferred and is not converted into a pass by this FF-V3 result.

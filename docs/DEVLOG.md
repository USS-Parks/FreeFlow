# FreeFlow Development Log

The log is append-only once STS execution begins. A prompt is not complete until its prescribed gate passes and the resulting commit SHA is recorded.

## Session ledger

| Date       | Prompt   | Status      | Files changed                                                                                                                                                                                   | Verification                                                                                                                                                                                                                                                                            | Commit SHA                                                                                                                                                                                                                 | Notes                                                                                                                                                                                           |
| ---------- | -------- | ----------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-07-17 | PLAN-001 | Complete    | `README.md`, `PLANNING/FREEFLOW-CLEAN-ROOM-PSPR.md`, `docs/legal/CLEAN-ROOM-POLICY.md`, `docs/product/BEHAVIOR-PARITY-MATRIX.md`, `docs/DEVLOG.md`                                              | Documentation review pending                                                                                                                                                                                                                                                            | Uncommitted                                                                                                                                                                                                                | Initial canonical plan drafted; no product execution authorized                                                                                                                                 |
| 2026-07-17 | PLAN-002 | Complete    | `README.md`, `PLANNING/FREEFLOW-CLEAN-ROOM-PSPR.md`, `PLANNING/FREEFLOW-UPSTREAM-FOUNDATION-BRIEF.md`, `docs/DEVLOG.md`                                                                         | GitHub API evidence and documentation checks pending                                                                                                                                                                                                                                    | Uncommitted                                                                                                                                                                                                                | Planning-only upstream snapshot; Handy preliminary foundation candidate; FF-G2 not executed                                                                                                     |
| 2026-07-17 | FF-G1    | Complete    | Public-source ledger, P0/P1 fixtures, parity matrix, clean-room policy, PSPR status and upstream complexity brief                                                                               | 19/19 P0/P1 fixture mapping; 9 forum sources; relative-link, STS marker, encoding and staged whitespace checks passed                                                                                                                                                                   | `2624d411f4589974513854c16b4cfc0511d4d178`                                                                                                                                                                                 | Public forums are unverified risk hypotheses, not proprietary implementation evidence                                                                                                           |
| 2026-07-17 | FF-G2    | Complete    | `docs/architecture/ADR-0001-UPSTREAM-FOUNDATION.md`, model-cache provenance, PSPR/brief status                                                                                                  | Handy: lint, format, frontend build, Rust check, no-bundle release build, and 119 tests passed on Windows; OpenWhispr: quality and unsigned build passed, 4/547 tests failed; Cargo/npm advisories recorded; Parakeet size/hash verified                                                | `2ca3ac3aaa7158d2f1b29021254170bb26e59940`                                                                                                                                                                                 | Handy `d861e24bc825c699ccf7215a430684c6e322131c` accepted; OpenWhispr reference-only; macOS host unavailable and remains a release gate                                                         |
| 2026-07-18 | FF-G3    | Complete    | Audited Handy merge, FreeFlow identity/assets, updater/model network removal, CI, provenance gate, dependency ledger, notices, build documentation                                              | Frozen Bun install, lint, format, typecheck/Vite build, 116 Rust tests, RustSec policy audit, provenance scan, staged diff checks, no-slop hook, and Windows no-bundle release build passed                                                                                             | `0dd27b26c6d3b54407bad89789e62ae08978ef60`                                                                                                                                                                                 | Windows artifact: 45,116,416 bytes, SHA-256 `d1f3fbbea0b72a768c5d863c3ffd3e8b7a1659e371b52d351a6a07b09a4f019f`; macOS evidence remains a release blocker                                        |
| 2026-07-18 | FF-G4    | Complete    | Versioned engine/service contracts, reversible SQLite migrations, atomic typed settings, backend file/clipboard commands, frontend boundary gate, ADR-0002                                      | Frozen Bun install, lint/service-boundary scan, format, typecheck/Vite build, 127 Rust tests, RustSec policy audit, provenance scan, staged diff checks, no-slop hook, and Windows no-bundle release build passed                                                                       | `ce9c7df66a607feceb6a63c970aa336b17c5bc9c`                                                                                                                                                                                 | Windows artifact: 43,334,144 bytes, SHA-256 `c93c6319e6c74f91df41b797b5c871a7cd4cb182bd71efb77603b168f2104ba6`; macOS evidence remains a release blocker                                        |
| 2026-07-18 | FF-V1    | Complete    | Immutable Parakeet manifest, consent UI, downloader/manual installer, receipt/tamper integrity, model lifecycle integration, ADR-0003, notices, 23 locale catalogs, macOS live workflow         | Frozen Bun install; lint; format; frontend build; 137/137 ordinary Rust tests; Windows manual/direct live tests; Apple Silicon and Intel direct-download/install/load/session tests; full hosted Windows/macOS/provenance CI; translation, RustSec, actionlint, Windows no-bundle build | `ccac099de6b3cb86fdbfdea1fc57d05a5b8078b2`; `0baeb5320d0128fa9bb7146c69905d498f605d46`; `a1d13f8befbf8d672e59d406142231a735ce71e7`; `5d269096ae9c976719f1e7aaebc3c7a8fc8bd15b`                                             | Closeout Windows artifact: 43,653,120 bytes, SHA-256 `ab6b8750088055dfd098e62451dd6e8c384dfc14d33a9276dddfcf537020dce2`; macOS live run `29649911938` and full CI run `29650991927` passed      |
| 2026-07-18 | FF-V2    | In progress | Truthful microphone selection/diagnostics, typed dictation lifecycle, atomic retryable capture/recovery, bundled pinned Silero V4, localized errors, live verifier, ADR-0004 and live checklist | 145/145 ordinary Rust tests; 22/22 translation catalogs; Windows production-path default-mic verifier captured 50/50 cycles and cancellation returned idle                                                                                                                              | `af7a3b2746737eec869344cbb5f2387b0d55ef75`                                                                                                                                                                                 | On-demand mic-ready p95 was 504.72 ms and is not misrepresented as the 150 ms feedback gate; interactive feedback/first-word/hot-plug/contention/sleep-wake and macOS live evidence remain open |
| 2026-07-18 | FF-V3    | Complete    | Local-ASR raw/recovery path and evaluator; immutable hosted gate; retained Windows, Apple Silicon, and Intel public-corpus/resource/zero-network evidence                                       | 153/153 ordinary Rust tests with 2 live-model tests ignored; normal main CI `29672015509`; hosted live gate `29672022926`; all frozen WER/task/latency/RSS/network thresholds passed on all three runners                                                                               | `df5c23c7eb5d8c6d058ae8f695fb3584123b923a`; `a6c8adb2c40f6d594cbc6e5f13f53d6941b1c7a4`; `2158e5d7a8eeb16c95cbcadccc8407685a04a6bb`; `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5`; `f4d2a0cbaec389a987be1d03f3a8d8084f7799c9` | Revised FF-V3 gate passed. FF-V2 remains user-deferred; project-owned dictation corpus remains mandatory at the later release gate                                                              |
| 2026-07-18 | FF-V4    | In progress | Target capture/security guard, direct-first insertion, clipboard/manual fallbacks, boundary formatting, paste-last, undo metadata, localized tray UI, frozen live matrix                        | 159/159 runnable Rust library tests passed with 2 ignored; Windows Cargo check; ESLint; service-boundary; Prettier; TypeScript; Vite build; 22/22 translation catalogs; hosted Windows/macOS/provenance/security run `29675125703` passed                                               | `5877aacea9aa2e9cb2be832cf5df2f77c981a2fb`                                                                                                                                                                                 | Live 100-attempt application matrices, clipboard preservation, secure-field refusal, and interactive Windows/macOS evidence remain open; FF-V5 has not begun                                    |
| 2026-07-19 | FF-V5    | Complete    | Original status bar states/docking/accessibility, state-bearing tray controls, Hub keyboard navigation, 23 catalogs, retained live matrix                                                       | Frozen install; frontend/translation/build gates; 162 Rust tests; strict Clippy delta; no-bundle build; RustSec/provenance; hosted run `29680441253`                                                                                                                                    | `b02751dc4e0edb06ec0827dfc3fbcdb24e2d7d6d`; closeout `26fc961e23e97e5d358b21f619d4464c37cf02f7`                                                                                                                            | Retained live multi-monitor/scaling/full-screen/focus/drag/screen-reader matrix assigned to FF-R2                                                                                               |
| 2026-07-19 | FF-V6    | Complete    | Resumable onboarding, privacy promise, permission repair, model/completion separation, shortcut/autostart preferences, first-dictation proof, diagnostics, uninstall-data guidance, 23 catalogs | Frozen install; frontend/446-key translation/build gates; 168 Rust tests; strict Clippy delta; no-bundle release build; RustSec/provenance; hosted run `29690161596` passed                                                                                                             | `bcea051e8f3435eb927502d83fb2c14f7c6b091c`                                                                                                                                                                                 | Retained signed-build live install/permissions/restart/upgrade/uninstall matrix assigned to FF-R2                                                                                               |

### 2026-07-18 — FF-G3

- Objective: Create the FreeFlow foundation from only the FF-G2-approved Handy material, with original identity, preserved notices, controlled network surfaces, CI, and clean developer builds.
- Starting commit: `1a9b2195ed0ec215b5ffaf0074c1a5c7092c810f`; imported parent `d861e24bc825c699ccf7215a430684c6e322131c`.
- Reuse classification: audited upstream reuse at the Tauri/Rust/React foundation; original FreeFlow branding, policy, CI, and provenance implementation; OpenWhispr remains reference-only.
- Files changed: imported foundation workspace; `NOTICE.md`; `.cargo/audit.toml`; `.github/workflows/ci.yml`; `assets/freeflow-icon.svg`; `scripts/check-foundation-provenance.sh`; `docs/legal/UPSTREAM-DEPENDENCY-LEDGER.md`; original FreeFlow React marks, generated application icons, replacement sound/status media, configuration, documentation, and dependency locks.
- Verification commands: `bun install --frozen-lockfile`; `bun run lint`; `bun run format:check`; `bun run build`; `cargo test --manifest-path src-tauri/Cargo.toml`; `bun run tauri build --no-bundle`; `cargo audit -f src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; `git diff --cached --check`.
- Live-system evidence: Vulkan SDK 1.4.350.0 plus Visual Studio Build Tools produced `src-tauri/target/release/freeflow.exe`; ProductName and FileDescription are `FreeFlow`; size 45,116,416 bytes; SHA-256 `d1f3fbbea0b72a768c5d863c3ffd3e8b7a1659e371b52d351a6a07b09a4f019f`.
- Result: pass on Windows. All 116 Rust tests passed; the provenance scan found no prohibited upstream runtime identity; the updater plugin/capabilities/UI/IPC and upstream model endpoints are absent; the foundation model catalog is intentionally empty until FF-V1.
- Commit SHA: `0dd27b26c6d3b54407bad89789e62ae08978ef60`.
- Deviations or remaining work: macOS host evidence is unavailable in this session. The CI matrix is defined but unexecuted until publication; macOS build/live evidence remains an explicit release blocker. Two `quick-xml 0.38.4` RustSec advisories remain narrowly ignored only through the parked Linux Wayland graph; active Windows/macOS resolution is `0.41.0`. Model weights remain untracked and network installation remains disabled until FF-V1.

### 2026-07-18 — FF-G4

- Objective: Establish versioned service contracts plus durable configuration and history storage without allowing feature UI to bypass the Rust application boundary.
- Starting commit: `7d00d1767b6b02c1474a6563c73b990c65872171`.
- Reuse classification: extension at the imported manager seams; new contract, migration, atomic-settings, and boundary-enforcement implementation.
- Files changed: `src-tauri/src/contracts/mod.rs`; `src-tauri/src/storage/`; history/settings managers; Rust commands and generated TypeScript bindings; frontend history/debug/clipboard consumers; Tauri capabilities; dependency locks; `scripts/check-service-boundary.ts`; `docs/architecture/ADR-0002-SERVICE-CONTRACTS-AND-STORAGE.md`.
- Verification commands: `bun install --frozen-lockfile`; `bun run lint`; `bun run format:check`; `bun run build`; `cargo test --manifest-path src-tauri/Cargo.toml`; `bun run tauri build --no-bundle`; `cargo audit -f src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; `git diff --cached --check`.
- Live-system evidence: Vulkan SDK 1.4.350.0 plus Visual Studio Build Tools produced `src-tauri/target/release/freeflow.exe`; size 43,334,144 bytes; SHA-256 `c93c6319e6c74f91df41b797b5c871a7cd4cb182bd71efb77603b168f2104ba6`.
- Result: pass on Windows. All 127 Rust tests passed, including cancellation, timeout, corrupt-settings recovery, forward/rollback migration, transactional-failure rollback, and restart persistence. The frontend service-boundary and provenance gates passed. RustSec reported no denied vulnerability and 28 allowed upstream warnings.
- Commit SHA: `ce9c7df66a607feceb6a63c970aa336b17c5bc9c`.
- Deviations or remaining work: concrete imported managers are not falsely claimed as contract adapters; their attachment remains owned by the vertical-slice prompts that change those behaviors. macOS build/live evidence remains unavailable and an explicit release blocker. Model network installation remains disabled until FF-V1 confirms immutable model revisions, hashes, sizes, licenses, and user confirmation.

### 2026-07-18 — FF-V1 (in progress)

- Objective: Implement a traceable, explicit-consent model supply chain with direct and manual installation, verified lifecycle operations, and no caller-controlled network source.
- Starting commit: `0bb8d0f9f7233d0529931984e31c746f6ab2d5f9` on `codex/ff-v1-model-install`.
- Reuse classification: extension at the imported model-manager and transcribe.cpp seams; new manifest, consent, transfer, receipt, integrity, test, and provenance implementation.
- Files changed: `models/manifests/parakeet-unified-en-0.6b-q8_0.json`; model catalog/manager/commands/install service; generated TypeScript bindings; onboarding/settings/store install UI; all 23 locale catalogs; `NOTICE.md`; `models/README.md`; `docs/architecture/ADR-0003-MODEL-SUPPLY-CHAIN.md`; dependency locks; live model-load verifier; `.github/workflows/macos-model-live-gate.yml`.
- Verification commands: `bun install --frozen-lockfile`; `bun run lint`; `bun run format:check`; `bun run build`; `bun run check:translations`; `cargo test --manifest-path src-tauri/Cargo.toml`; focused manual-import live test with `FREEFLOW_LIVE_PARAKEET_PATH`; consent-gated direct-download live test with `FREEFLOW_MODEL_LICENSE_ACCEPTANCE`; `bun run tauri build --no-bundle`; `cargo audit -f src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; `actionlint .github/workflows/ci.yml .github/workflows/macos-model-live-gate.yml`.
- Automated evidence: 137/137 ordinary Rust tests passed with both 731 MB live tests intentionally ignored in the ordinary suite. Corrupt/manual hash mismatch, low disk, offline, cancellation, Range resume, stale consent digest, approved HTTPS redirect policy, invalid receipt repair, receipt tampering, and same-size installed-weight tampering passed. All 22 non-English catalogs contain the legally accurate English fallback strings for the new consent surface; translation-key consistency passed. Actionlint 1.7.12 accepted both workflows.
- Live-system evidence: local artifact size `731357568`, SHA-256 `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`. Production manual-install logic copied, exact-size checked, SHA-verified, receipted, and re-discovered the pinned artifact. The separate production direct-transfer test downloaded the artifact through the restricted HTTP client, verified manifest digest `0c6a40bac30ebe258d963f76e98a49b078d1a1dd426b4d4d530c6e6bf88a7186`, installed and re-discovered it, loaded it on `Vulkan0`, reported English streaming capability, and created a session in 123.94 seconds. The closeout Windows no-bundle gate rebuilt `src-tauri/target/release/freeflow.exe` at `43653120` bytes with SHA-256 `ab6b8750088055dfd098e62451dd6e8c384dfc14d33a9276dddfcf537020dce2`.
- Result: full FF-V1 gate pass on Windows, Apple Silicon macOS, and Intel macOS. Hosted foundation/provenance run `29650991927` passed on commit `5d269096ae9c976719f1e7aaebc3c7a8fc8bd15b`. RustSec reported no denied vulnerability and 28 allowed upstream warnings. The provenance and frontend service-boundary gates passed.
- Commit SHA: implementation `ccac099de6b3cb86fdbfdea1fc57d05a5b8078b2`; Intel gate correction `0baeb5320d0128fa9bb7146c69905d498f605d46`; CI path repair `a1d13f8befbf8d672e59d406142231a735ce71e7`; check-publication repair `5d269096ae9c976719f1e7aaebc3c7a8fc8bd15b`.
- Hosted macOS evidence: initial run `29648550447` accepted both repository consent variables and passed Apple Silicon, while Intel exposed the imported `ort-sys 2.0.0-rc.12` prebuilt gap before FreeFlow ran. Corrected run `29649911938` passed consent and both architectures against manifest digest `0c6a40bac30ebe258d963f76e98a49b078d1a1dd426b4d4d530c6e6bf88a7186`. Apple Silicon installed the pinned artifact, loaded it on CPU, reported English streaming capability, created a session, and passed in 89.25 seconds. Intel did the same in 143.47 seconds. Each architecture reported one passed test and zero failures.
- Deviations or remaining work: per explicit user authorization, `origin/main` was initialized at completed FF-G4 closeout `0bb8d0f9f7233d0529931984e31c746f6ab2d5f9`; FF-V1 remains on `codex/ff-v1-model-install`. The hosted model-load proof used CPU on both macOS architectures; performance benchmarking remains FF-V3 scope. The Intel developer gate conditionally links Homebrew's bottled ONNX Runtime solely because the imported ONNX/VAD dependency graph is compiled with the application; shipping or removing that runtime remains later packaging work. FF-V2 has not started. The legal/consent strings intentionally use accurate English fallbacks in the 22 non-English catalogs pending qualified localization.

### 2026-07-18 — FF-V2 (in progress)

- Objective: Implement truthful device selection, resampling/VAD capture, push-to-talk/toggle/cancel lifecycle, live levels and feedback metrics, retry-safe audio persistence, and microphone diagnostics.
- Starting commit: `32244ac124e28f0f3a4868f7895ea59557a1772d` on `codex/ff-v2-audio-state`.
- Reuse classification: extension at the imported CPAL recorder, resampler, Silero VAD, shortcut coordinator, history, and Tauri command seams; new atomic capture/recovery, typed diagnostics/state, live verifier, and clean-room artifact provenance.
- Files changed: audio manager/recorder persistence and history recovery; coordinator state events; audio commands and generated bindings surface; headless CLI verifier; frontend error handling and 23 locale catalogs; bundled Silero V4 artifact/manifest; notices, dependency ledger, ADR-0004, PSPR and live checklist.
- Verification commands: frozen dependency state retained; `bun run lint`; `bun run format:check`; `bun run build`; `bun run check:translations`; `cargo test --manifest-path src-tauri/Cargo.toml`; `cargo build --manifest-path src-tauri/Cargo.toml`; `freeflow --verify-audio 1 --repeat 50 --json`; `bun run tauri build --no-bundle`; `cargo audit --file src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; `git diff --check`.
- Live-system evidence: Windows configured-default microphone completed 50/50 real CPAL open/record/stop cycles, returned 16,320–20,000 16 kHz samples per one-second probe with non-zero levels, and returned idle after cancellation. On-demand microphone-ready latency was 504.72 ms p95; this is retained as a finding rather than called a pass for the separately instrumented 150 ms visual-feedback gate. The final optimized no-bundle build produced `src-tauri/target/release/freeflow.exe` at 43,785,728 bytes with SHA-256 `ed0bfcdb31f8f13e142a7bcf1f458e6ce1520e3a5d8a5719e964040894ec479f`; its bundled Silero model is 1,807,522 bytes and matches the pinned SHA-256 `a35ebf52fd3ce5f1469b2a36158dba761bc47b973ea3382b3186ca15b1f5af28`.
- Result: implementation and Windows default-device gate candidate pass. All 145 ordinary Rust tests passed with two live tests intentionally ignored; all 22 non-English translation catalogs passed; RustSec found no denied vulnerability and 28 allowed upstream warnings; release build, provenance, service-boundary, format, frontend build, and diff gates passed. Full FF-V2 gate remains open.
- Commit SHA: gate candidate `af7a3b2746737eec869344cbb5f2387b0d55ef75`; FF-V2 remains in progress.
- Deviations or remaining work: interactive 50-cycle feedback/first-word evidence, 20 toggle cycles, physical selected-device hot-plug/reconnect, Zoom/Teams/browser contention, sleep/wake, and macOS permission/device evidence remain mandatory. No FF-V3 ASR quality claim is implied.

### 2026-07-18 — FF-V2 verifier correction (in progress)

- Objective: Separate the headless microphone-ready diagnostic from the 150 ms interactive visible-feedback gate and measure application feedback from shortcut receipt rather than after coordinator queueing.
- Starting commit: `5c948da53e9ff41026b558ccf8390a2fb3417226` on `codex/ff-v2-audio-state`.
- Reuse classification: focused correction at the existing FF-V2 action/coordinator and headless verifier seams; no new dependency or product surface.
- Files changed: shortcut action activation timestamp propagation; feedback metric origin; headless verifier JSON/exit semantics; FF-V2 live-gate interpretation and continuation evidence.
- Verification commands: `bun install --frozen-lockfile`; `bun run lint`; `bun run format:check`; `bun run build`; `bun run check:translations`; `cargo test --manifest-path src-tauri/Cargo.toml`; `cargo build --manifest-path src-tauri/Cargo.toml`; `freeflow --verify-audio 1 --repeat 50 --json`; `bun run tauri build --no-bundle`; `cargo audit -f src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; `git diff --check`.
- Live-system evidence: the corrected Windows configured-default probe completed 50/50 non-zero captures, returned 16,320–20,000 16 kHz samples per cycle, reported `microphone_ready_p95_ms: 164.2139`, explicitly reported `feedback_gate_measured: false`, and returned idle after cancellation. The optimized artifact is 43,785,728 bytes with SHA-256 `79a91513a8d445fd1887813d212a9b3990e4ff8d7767df5973525ef81b9f157f`.
- Result: correction pass. Frozen installation reported 366 installs across 442 packages with no changes; lint, format, frontend build, translations, 145 ordinary Rust tests with two large model tests ignored, debug/release builds, RustSec policy, provenance, live capture, and diff checks passed. RustSec found no denied vulnerability and the existing 28 allowed upstream warnings.
- Commit SHA: `e6e44dcd969d5da0ee0ceb8c2f38a471d3f9db8a`.
- Deviations or remaining work: the warnings-denied clippy attempt did not reach linting because a concurrent native transcribe/Vulkan cache build failed with `Access is denied` during parallel shader generation; it is an environment non-result, not a pass or source failure. No foreground GUI or input automation was continued while the user used the Windows host. Interactive Windows and macOS live evidence remains mandatory, so FF-V2 is not complete.

### 2026-07-18 — FF-V3 (in progress)

- Objective: Connect persisted capture audio to the selected local engine, preserve recoverable engine-raw text separately from deterministic correction and optional post-processing, retain model/language/latency/error metadata, surface typed progress, and establish the frozen public/owned corpus quality/resource gate.
- Starting commit: `2361db37020e779865d50fc9886cff17e3790677` on `codex/ff-v3-local-asr`.
- Roster deviation: the user explicitly deferred FF-V2's remaining live matrix to a separate macOS machine and directed STS to begin FF-V3 immediately. This authorization does not mark FF-V2 complete, waive FF-V3 evidence, or authorize FF-V4.
- Reuse classification: extension at the existing transcribe-cpp/transcribe-rs manager seam, retry-safe history pipeline, Tauri event boundary, headless CLI, and history UI; new reversible FF-V3 metadata migration and independent evaluator/scorer.
- Files changed: transcription/actions/history managers and retry command; headless CLI and evaluator; migration v5; Windows RSS feature bindings; generated TypeScript bindings; history raw/error recovery UI; 23 locale catalogs; FF-V3 threshold, input-manifest, result, notice, PSPR, and DEVLOG evidence.
- Verification commands: ESLint over `src`; `scripts/check-service-boundary.ts`; `scripts/check-translations.ts`; TypeScript compiler and Vite production build; `cargo fmt --check`; `cargo test --manifest-path src-tauri/Cargo.toml` (150 passed, 2 ignored); warnings-denied `cargo clippy --all-targets` after allowing only the seven inherited lint classes enumerated below; `cargo build --release --manifest-path src-tauri/Cargo.toml`; `cargo audit --file src-tauri/Cargo.lock`; `scripts/check-foundation-provenance.sh`; Prettier and `git diff --check`.
- Live-system evidence: the pinned 731,357,568-byte model matched SHA-256 `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`, loaded on NVIDIA RTX 3050 Ti `Vulkan0` in 1,274 ms, and evaluated a deterministic 20-speaker/210.5-second LibriSpeech `test-clean` subset. Raw WER was `0.013011`; semantic task success was `0.95`; inference p50/p95 were `146/332 ms`; RSS before load/after load/after evaluation was `52,613,120/120,995,840/178,192,384` bytes. Every frozen Windows public-corpus threshold passed. The 43,565,568-byte release executable SHA-256 is `7f2d1d1dc7d846d60106e05583b895bc168617e5eba97727aaee5fd65bab629c`.
- Network evidence: the evaluator has no download path and ran with all conventional outbound proxy variables pointed to closed local endpoint `127.0.0.1:9`. A temporary program-scoped Windows Firewall rule was attempted and rejected with `Access is denied`; therefore this is not recorded as the required blocked-network pass.
- Result: implementation and Windows public-corpus gate candidate pass. FF-V3 remains in progress.
- Commit SHA: `df5c23c7eb5d8c6d058ae8f695fb3584123b923a`.
- Deviations or remaining work: Windows project-owned corpus evidence, privileged blocked-network or process packet-trace evidence, additional manifest-approved model candidates, and macOS public/owned quality/latency/memory/offline evidence remain mandatory. The unmodified starting tree has existing `unused-imports`, `dead-code`, `needless-lifetimes`, `needless-return`, `items-after-test-module`, `manual-repeat-n`, and `write-with-newline` findings; strict clippy passed after allowing exactly those inherited lint classes, while the sole FF-V3 evaluator finding was corrected. Bun was unavailable on PATH, so the unchanged installed dependency tree was used directly for frontend gates; no frozen-install result is claimed. No FF-V4 work began.

### 2026-07-18 — FF-V3 hosted gate continuation (complete)

- Objective: Complete the revised FF-V3 gate without foreground interaction by retaining public-corpus, performance, memory, model-identity, and real zero-network evidence on hosted Windows, Apple Silicon macOS, and Intel macOS runners.
- Starting commit: `9ba4a08c893ee7bb336b772e57361b3a2d536f25` on canonical `main`.
- Authorized rescope: the user explicitly moved consented project-owned corpus evidence to the later release gate. It remains mandatory before release and is not replaced with synthetic evidence.
- Implementation: add a manifest-consent-bound headless local-model installer; add a same-process numeric TCP denial probe to corpus results; reproduce the exact hashed LibriSpeech subset; run the production evaluator under a Windows program-scoped outbound firewall rule or macOS network-denying sandbox; retain host, executable, install, isolation, stderr, and JSON evidence as workflow artifacts.
- Local verification: 153/153 ordinary Rust tests passed with 2 live-model tests ignored; warnings-denied FF-V3 clippy delta passed with the seven documented inherited lint classes allowed; ESLint, service-boundary, 22-catalog translation, TypeScript, Vite production build, Prettier, Rustfmt, PowerShell/Bash syntax, Actionlint/ShellCheck with the previously proven hosted Intel runner label allowlisted, provenance, and diff checks passed. The corpus preparation script reproduced all 20 WAV inputs from the retained hash-verified archive.
- Hosted evidence: run `29672022926` passed at exact commit `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5`. Windows x86_64 recorded WER/task `0.013011/0.95`, p50/p95 `1665/4170 ms`, RSS `15,843,328/1,086,214,144/1,284,997,120`, and firewall-denied TCP error `10013`. Apple Silicon recorded `0.013011/0.95`, `690/1666 ms`, `74,448,896/1,193,328,640/1,403,895,808`, and sandbox-denied error `1`. Intel macOS recorded `0.013011/0.95`, `1588/3779 ms`, `57,778,176/1,171,804,160/1,351,737,344`, and sandbox-denied error `1`. All frozen thresholds passed.
- Failure history: run `29669509926` exposed sandbox-incompatible external RSS collection on macOS; run `29670237557` passed macOS after native RSS collection but Windows p95 missed at `6334 ms` and `6188 ms`; run `29671820518` exposed missing `winget` on `windows-2022`. These were retained as failures. The final pass used the official Vulkan SDK installer with exact SHA-256 rather than changing acceptance thresholds.
- Normal CI: run `29672015509` passed provenance, security audit, Windows foundation, and macOS foundation jobs at the same exact commit.
- Result: revised FF-V3 gate complete. Full evidence: `docs/verification/FF-V3-HOSTED-CROSS-PLATFORM-2026-07-18.md` and `docs/verification/evidence/ff-v3/hosted-cross-platform-2026-07-18.json`.
- Commit SHAs: workflow `a6c8adb2c40f6d594cbc6e5f13f53d6941b1c7a4`; native macOS RSS `2158e5d7a8eeb16c95cbcadccc8407685a04a6bb`; Windows runner pin `2f68c4d089288bb62cf153587dc50aadf057acab`; deterministic Vulkan provisioning and passing gate head `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5`; retained evidence closeout `f4d2a0cbaec389a987be1d03f3a8d8084f7799c9`.
- Remaining work: consented project-owned dictation corpus evidence remains mandatory at the later release gate. FF-V2 remains explicitly user-deferred rather than passed. FF-V4 is now next in the roster.

### 2026-07-18 — FF-V4 (in progress)

- Objective: Implement reliable, target-bound insertion with direct delivery,
  lossless clipboard fallback, explicit manual recovery, boundary formatting,
  paste-last, and safe undo metadata.
- Starting commit: `9bf7583266eecbd8fa053c58db6d186bf78152f9` on canonical `main`.
- Reuse classification: extension at the imported Enigo, clipboard, shortcut,
  tray, settings, history, Tauri event, and translation seams; new Windows UI
  Automation and macOS Accessibility target capture plus deterministic insertion
  policy and evidence specification.
- Files changed: insertion/platform contracts; platform target capture;
  clipboard/insertion service; transcription action target lifecycle; settings
  and generated binding; tray paste-last action; frontend manual-copy recovery;
  23 locale catalogs; FF-V4 live gate; PSPR and DEVLOG.
- Verification commands: `cargo fmt`; `cargo test --manifest-path
src-tauri/Cargo.toml --lib`; `cargo check --manifest-path
src-tauri/Cargo.toml`; ESLint over `src`; service-boundary checker; Prettier;
  TypeScript compiler; Vite production build; translation consistency checker;
  `git diff --check`.
- Automated evidence: 159/159 runnable Rust library tests passed with two
  existing live-model tests ignored. Security-unknown, secure-field, changed
  target, boundary, clipboard-restoration error, macOS plain-text format
  allowlist, and content-free undo metadata tests pass. Windows Cargo check,
  ESLint, service boundary, formatting, TypeScript, production frontend build,
  all 22 non-English translation catalogs, provenance, and the Windows
  no-bundle release build and final optimized rebuild pass. The final release
  executable is 43,650,560 bytes with SHA-256
  `ce0f9f931084e5b7e25c0222cc57f8ff2eb926670b9a1276a6d3659f3902b899`.
- Hosted structural evidence: normal CI run
  [`29675125703`](https://github.com/USS-Parks/FreeFlow/actions/runs/29675125703)
  passed at ledger commit `7d0d3cb3c64587d874e5e50050a2534573749cef`.
  Windows foundation, macOS foundation, provenance, and RustSec were all green.
  This is native cross-platform compile/test evidence, not interactive
  application-matrix evidence.
- Live-system evidence: pending. Local Windows input automation was deliberately
  not run because it would take over the user's foreground session. Local macOS
  cross-compilation stops in the dependency graph before FreeFlow code because
  this Windows host has no Apple C compiler; hosted/native macOS evidence is
  required rather than inferred.
- Result: implementation candidate structurally passes; FF-V4 remains in
  progress.
- Commit SHA: candidate `5877aacea9aa2e9cb2be832cf5df2f77c981a2fb`.
- Deviations or remaining work: execute the frozen 100-attempt Windows and
  macOS application matrices, prove all clipboard/security/recovery invariants,
  retain exact platform evidence, and only then close FF-V4. FF-V5 has not
  started.

### 2026-07-19 — FF-V4 implementation closeout and retained live gate (complete)

- Objective: Close the verified FF-V4 implementation slice without weakening
  its live acceptance criteria, then continue STS at FF-V5.
- Starting commit: `6159e5a6d0f02bc99b7f6d1b828b6e2fe1f4bc55` on canonical `main`.
- Authorized rescope: after directing completion to proceed stem-to-stern, the
  user explicitly required FF-V4 to stop blocking the remaining roster. The
  frozen 100-attempt Windows and macOS matrices therefore move intact to FF-R2;
  the 98% thresholds and security/clipboard invariants are not waived.
- Verification evidence: candidate `5877aacea9aa2e9cb2be832cf5df2f77c981a2fb`
  passed 159/159 runnable Rust library tests with two live-model tests ignored,
  the frontend/service-boundary/translation gates, Windows no-bundle release
  build, and hosted native Windows/macOS/provenance/security run `29675125703`.
- Result: FF-V4 implementation slice complete. The retained live insertion
  matrix is an explicit FF-R2 release blocker. FF-V5 is next.
- Commit SHA: recorded by the closeout commit containing this entry.

### 2026-07-19 — FF-V5 (complete)

- Objective: Complete the original FreeFlow status bar and native tray/menu
  recovery surface with persistent work-area docking, accessible state
  semantics, and single-pipeline controls.
- Starting commit: `b4bd830` on canonical `main`.
- Reuse classification: extension at the existing Tauri overlay, tray,
  transcription coordinator, typed settings, i18next, and Hub navigation seams;
  no new runtime dependency.
- Files changed: overlay state/docking service and UI; tray state/menu controls;
  settings/bindings; Hub keyboard navigation; 23 locale catalogs; capability,
  PSPR, and frozen FF-V5 evidence.
- Verification commands: verified Bun 1.3.14 frozen install; `bun run lint`;
  `bun run format:check`; `bun run check:translations`; `bun run build`;
  `cargo test --manifest-path src-tauri/Cargo.toml --lib`; warnings-denied
  `cargo clippy --all-targets` with only the seven inherited lint classes
  allowed; `bun run tauri build --no-bundle`; `cargo audit -f
src-tauri/Cargo.lock`; provenance and diff gates.
- Automated evidence: 162/162 runnable Rust library tests passed with two
  live-model tests ignored; all 22 non-English catalogs contain the 413-key
  reference surface; frontend/service-boundary/type/build gates and strict
  Clippy delta passed. RustSec reported no denied vulnerability and 28 allowed
  upstream warnings. The final Windows release executable is 44,092,416 bytes
  with SHA-256
  `5241d495a565b346db33d756b8a8f085588a90f5d8f175f8a9b8f16fa27ea85c`.
- Hosted evidence: run `29680441253` passed native Windows and macOS foundation
  jobs, provenance, and RustSec at the exact candidate commit.
- Result: FF-V5 implementation slice complete. Retained interactive evidence is
  assigned to FF-R2 by the 2026-07-19 PSPR consolidation. FF-V6 is next.
- Commit SHA: `b02751dc4e0edb06ec0827dfc3fbcdb24e2d7d6d`.

### 2026-07-19 — FF-V6 (complete)

- Objective: deliver original, resumable onboarding for the local/privacy
  promise, permissions, model choice, shortcut, launch at login, first
  dictation, diagnostics, and uninstall-data choices.
- Starting commit: `26fc961e23e97e5d358b21f619d4464c37cf02f7`.
- Reuse classification: extension at the typed settings, model-selection,
  shortcut, autostart, permission, and transcription-event seams; new original
  welcome, preferences, first-dictation, diagnostics, and uninstall guidance.
- Files changed: durable `OnboardingStage` and migrations; typed onboarding and
  diagnostics commands/bindings; model/autostart correctness; full React wizard;
  keyboard-accessible shortcut control; 23 locale catalogs; uninstall and gate
  documentation.
- Verification commands: verified Bun 1.3.14 frozen install; `bun run lint`;
  `bun run format:check`; `bun run check:translations`; `bun run build`; full
  `cargo test`; warnings-denied Clippy with only the seven inherited lint classes
  allowed; `bun run tauri build --no-bundle`; RustSec; provenance; diff checks.
- Automated evidence: 168/168 runnable Rust tests passed with the two explicit
  731 MB live-model tests ignored; 446 keys are consistent across 23 catalogs;
  frontend, typed boundary, strict Clippy delta, RustSec, provenance, and
  optimized Windows build gates passed. One initial temp-directory error 5 was
  eliminated by rerunning the entire suite under a project-scoped temp root.
- Artifact: 44,121,088-byte `freeflow.exe`, SHA-256
  `69cac522fa3c4013285bfc37ecc2e27e68efbe2c71829335e5e5e5ede68edc54`.
- Hosted evidence: run `29690161596` passed native Windows and macOS foundation
  jobs, provenance, and RustSec at exact candidate commit
  `bcea051e8f3435eb927502d83fb2c14f7c6b091c`.
- Result: FF-V6 implementation slice complete. Milestone V implementation is
  complete and FF-P1 is next.
- Commit SHA: `bcea051e8f3435eb927502d83fb2c14f7c6b091c`.
- Deviations or remaining work: the signed-build clean-install, deny/regrant,
  revoke/repair, restart, upgrade, launch-at-login, first-dictation, and
  uninstall-data matrix remains mandatory at FF-R2 under the approved
  consolidation; no public-release claim is made by this candidate.

### 2026-07-19 — FF-P1 (complete)

- Objective: deliver searchable local transcription history, complete metadata
  and statistics, durable recovery, explicit retention, fail-closed deletion,
  clear-all, and never-store behavior.
- Starting commit: `92caaef445fc56269a1f9f18dbaf60c54b7a6673` on canonical
  `main`.
- Reuse classification: extension at the existing SQLite history manager,
  atomic WAV recovery, typed Tauri command/binding, settings, tray paste-last,
  and i18next seams; no new runtime dependency.
- Files changed: history schema migration and manager; transcription completion
  and retry lifecycle; settings and typed commands/bindings; history UI; 23
  locale catalogs; behavior matrix; frozen FF-P1 evidence.
- Verification commands: verified Bun 1.3.14 frozen install; ESLint and service
  boundary; Prettier and Rustfmt; 460-key translation consistency; TypeScript
  and production Vite build; full `cargo test`; warnings-denied Clippy with only
  the seven inherited lint classes allowed; optimized Tauri no-bundle build;
  RustSec; provenance; diff checks.
- Automated evidence: 173/173 runnable Rust tests pass with the two explicit
  731 MB live-model tests ignored. Focused retention, irreversible deletion,
  path validation, metadata search, migrations/restart, and atomic WAV recovery
  tests pass. RustSec reports no denied vulnerability and 28 allowed upstream
  warnings. All catalogs are structurally complete.
- Artifact: 44,207,104-byte `freeflow.exe`, SHA-256
  `7806dc5077326d18deb9a502e143e8cace2f4f153764520ae233c97582195c3e`.
- Hosted evidence: run `29692703288` passed native Windows and macOS
  foundation jobs, provenance, and security at the exact candidate commit.
- Result: FF-P1 is complete and FF-P2 is next.
- Commit SHA: `20b03120c1cbe6b9633cdd47c4f2b11b588b73e5`.
- Deviations or remaining work: none in FF-P1 implementation scope. The
  retained cross-platform interactive release matrices remain owned by FF-R2.

### 2026-07-19 — FF-P2 (complete)

- Objective: replace the legacy custom-word array with a durable local
  dictionary supporting vocabulary/replacement CRUD, starring, search/sort,
  deterministic precedence, atomic CSV transfer, and truthful ASR integration.
- Starting commit: `bc9dd283cd7c878f8bc15b02b309e0e94b8b6203` on canonical
  `main`.
- Reuse classification: extension at the existing SQLite migration, Tauri
  manager/command/binding, transcription capability, settings migration, and
  i18next seams; no new runtime dependency.
- Files changed: personalization migration and dictionary manager; typed
  commands and generated bindings; batch/streaming transcription integration;
  dictionary settings UI; 23 locale catalogs; behavior matrix; frozen FF-P2
  evidence.
- Verification commands: verified Bun 1.3.14 frozen install; ESLint and service
  boundary; Prettier and Rustfmt; 481-key translation consistency; TypeScript
  and production Vite build; full Rust tests; warnings-denied Clippy with only
  the seven inherited lint classes allowed; optimized Tauri no-bundle build;
  RustSec; provenance; diff checks.
- Automated evidence: 182/182 runnable Rust library tests pass with the two
  explicit 731 MB live-model tests ignored. Focused Unicode, case, boundary,
  precedence, duplicate, size, CSV rollback/round-trip, legacy migration,
  reversible schema, and engine-support tests pass. RustSec reports no denied
  vulnerability and 28 allowed upstream warnings. All catalogs are complete.
- Artifact: 44,329,984-byte `freeflow.exe`, SHA-256
  `dbce3160391bf8ed1ea0c06ce8b7bf20b29a3167da4f29edd0834f61adb829b0`.
- Hosted evidence: run `29694672120` passed native Windows and macOS
  foundation jobs, provenance, and security at the exact candidate commit.
- Result: FF-P2 is complete and FF-P3 is next.
- Commit SHA: `fc66b424fbe33826f77f3587d751714cad04ed6e`.
- Deviations or remaining work: none in FF-P2 implementation scope. The
  retained cross-platform interactive release matrices remain owned by FF-R2.

## Entry template

### YYYY-MM-DD — FF-XX

- Objective:
- Starting commit:
- Reuse classification: reuse / extraction / extension at seam / new implementation
- Files changed:
- Verification commands:
- Live-system evidence:
- Result:
- Commit SHA:
- Deviations or remaining work:

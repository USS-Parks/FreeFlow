# FreeFlow Local Dictation Clean-Room PSPR

Status: **Approved — STS active; FF-G1 through FF-G4, FF-V1 through FF-V6, and FF-P1 through FF-P4 complete; retained live matrices assigned to FF-R2; FF-P5 in progress**
Version: 2.0
Date: 2026-07-19
Canonical repository: `C:\Users\17076\Documents\FreeFlow`
Canonical remote: `https://github.com/USS-Parks/FreeFlow` (public; empty at plan-draft time)
Authoritative behavior ledger: `docs/product/BEHAVIOR-PARITY-MATRIX.md`
Authoritative public-source ledger: `docs/research/WISPR-OBSERVABLE-BEHAVIOR-SOURCE-LEDGER.md`
Authoritative P0/P1 fixtures: `docs/verification/P0-P1-ACCEPTANCE-FIXTURES.md`
Authoritative execution ledger: `docs/DEVLOG.md`

## 1. Initiative and outcome

Build and publish an independent, free, open-source Windows and macOS desktop application that provides workflow-level functional equivalence to the publicly documented desktop dictation experience of Wispr Flow, while running speech recognition and optional language-model cleanup locally.

The working product name is **FreeFlow**. The final public name is not settled until a naming-clearance gate passes.

“Functions identically” is operationalized as parity of user tasks, interaction cost, recovery behavior, and measured quality. Exact model outputs, source code, pixels, branded copy, proprietary services, and undisclosed algorithms are neither required nor permitted.

## 2. Authority and execution rule

This PSPR is the source of truth for implementation order and gates. Drafting or approving edits to this document is not authorization to implement it.

Execution begins only when the user says **`run it STS`**, approves named prompts, or approves a named milestone. Execution follows roster dependency order. Deviations require an explicit recorded addendum.

Each prompt normally produces one focused commit after its gate passes. No prompt is marked complete on mock-only evidence when its claim concerns global shortcuts, microphone capture, accessibility insertion, OS permissions, offline operation, installers, or other live integration behavior.

### 2026-07-19 retained interactive-gate consolidation

At the user's explicit direction to continue STS stem-to-stern without allowing
unavailable foreground or macOS sessions to block implementation, interactive
Windows/macOS matrices are retained intact at FF-R2. An implementation prompt
may close after its frozen deterministic invariants, native platform
compile/tests, and non-interactive integration gates pass. This changes timing,
not acceptance: no public release or live-platform claim may close until FF-R2
executes every retained matrix or records a user-approved support re-scope.

## 3. Scope

### In scope

- Windows 10/11 x64 desktop application.
- macOS 12+ on Apple Silicon and Intel, with documented performance degradation where acceleration is unavailable.
- No-account local use after models are installed.
- Local audio capture, VAD, ASR, formatting, history, personalization, and optional local text transformation.
- Original FreeFlow Hub, floating status control, tray/menu-bar experience, settings, onboarding, and accessibility.
- Signed/notarized release artifacts where project credentials are available; reproducible unsigned developer builds otherwise.
- MIT licensing for original FreeFlow code, subject to preserving upstream notices and recording model-specific licenses.

### Explicit exclusions

- Proprietary binary analysis, decompilation, protocol discovery, security bypass, copied assets, and service automation.
- Bit-for-bit transcription equivalence or use of Wispr outputs as training/evaluation data.
- Wispr accounts, pricing, billing, word caps, referrals, team administration, or proprietary cloud sync.
- Mobile apps, meeting recording, screen capture, and enterprise features in the initial release.
- Public release under the name FreeFlow until name/trademark clearance is recorded.

## 4. Clean-room governance

`docs/legal/CLEAN-ROOM-POLICY.md` is binding on all prompts.

The public reference product’s current Terms prohibit taking apart, decompiling, or reverse engineering the service to access source code, algorithms, or other IP. Therefore the default evidence boundary is public documentation only. No prompt in this roster installs or inspects the proprietary application.

Any proposed comparative testing against the proprietary service requires a separate, pre-approved legal and experimental addendum. It must not use service output to train or improve a model.

## 5. Settled architecture defaults

### Application foundation

- Preferred upstream: a pinned, audited fork of [Handy](https://github.com/cjpais/handy), because it is MIT-licensed and already implements cross-platform local audio capture, Whisper/Parakeet transcription, global shortcuts, tray behavior, and insertion with Tauri, React/TypeScript, and Rust.
- Mandatory decision gate: compare fork cost and maintainability against selective extraction from Handy and [OpenWhispr](https://github.com/OpenWhispr/openwhispr). Do not fork either until license, provenance, dependency, update, security, and gap audits are recorded.
- Planning evidence: `PLANNING/FREEFLOW-UPSTREAM-FOUNDATION-BRIEF.md` records the 2026-07-17 candidate snapshot and preliminary Handy-first decision. It does not replace FF-G2’s live builds or final exact-SHA audit.
- Desktop shell: Tauri 2.
- Hub UI: React, TypeScript, Vite, and an original accessible component/theme layer. Reuse upstream frontend infrastructure only where license and rebranding rules permit.
- Native/core layer: Rust.

### Local inference and data

- Audio: upstream Handy `cpal`/resampling/VAD seams unless the audit rejects them.
- ASR: upstream Whisper-family and Parakeet adapters, with model choice based on measured quality, latency, memory, license, and platform stability.
- Text transforms: optional `llama.cpp`-compatible local runtime behind a provider interface. P0 dictation must never require an LLM.
- Storage: SQLite with migrations, explicit retention controls, and separation of audio, raw transcript, final transcript, settings, dictionary, snippets, notes, and metrics.
- Network posture: no authentication or telemetry. Network access is allowed only for an explicit user-initiated model/update download, with URL, size, hash, license, and destination shown before transfer. Manual model installation remains supported.
- Secrets: none required for local mode. Any future BYOK provider is a separately approved extension and must use the OS credential store.

### Platform adapters

- Global shortcut, active application, selected/surrounding text, and insertion behavior are interfaces with Windows and macOS implementations.
- Accessibility/UI Automation is preferred for direct insertion and selection when safe. Clipboard paste is a documented fallback that restores prior clipboard content.
- Context collection is off by default, minimum-necessary, visible to the user, locally processed, and denylistable by app.

## 6. Reuse ledger

| Area                             | Default classification                            | Candidate                                        | Decision rule                                                                           |
| -------------------------------- | ------------------------------------------------- | ------------------------------------------------ | --------------------------------------------------------------------------------------- |
| Cross-platform shell/build       | Reuse                                             | Handy/Tauri 2                                    | Fork if current builds pass both platforms and updater/branding can be replaced cleanly |
| Audio capture/resampling/VAD     | Reuse                                             | Handy Rust core                                  | Preserve if latency and device-switch gates pass                                        |
| Local ASR adapters/model manager | Reuse then extend                                 | Handy `transcribe-cpp` and `transcribe-rs` seams | Extend only through a stable engine interface                                           |
| Global shortcuts/tray/autostart  | Reuse then repair                                 | Handy                                            | Live-test modifier, press/release, sleep/wake, and stuck-key behavior                   |
| Text insertion                   | Reuse at existing seam                            | Handy platform code                              | Harden with app matrix and clipboard preservation                                       |
| History, dictionary, snippets    | Extract concepts; implement at FreeFlow data seam | Handy features where present                     | Use original schema/API; do not inherit brittle settings storage                        |
| UI and floating status control   | New implementation over reusable primitives       | FreeFlow                                         | Original visual design and copy                                                         |
| Formatting/backtrack/styles      | New implementation                                | Deterministic pipeline plus optional local LLM   | Freeze behavioral tests before implementation                                           |
| Transforms/command mode          | New implementation at provider seam               | `llama.cpp`-compatible runtime                   | Optional download; raw text fallback on failure                                         |
| Scratchpad/insights              | New implementation over local database            | FreeFlow                                         | No cloud/account dependency                                                             |
| Meeting and sync features        | Parked                                            | None                                             | Require separate PSPR                                                                   |

## 7. Global verification gates

Every implementation prompt runs the narrowest relevant subset; milestone gates run all applicable checks.

1. **Provenance gate:** clean-room checklist, dependency license record, notices, and reuse classification are complete.
2. **Static gate:** Rust format/check/clippy with warnings denied; TypeScript typecheck, lint, and formatting pass.
3. **Unit gate:** deterministic Rust and UI unit tests pass, including migrations and error paths.
4. **Integration gate:** audio-to-transcript state machine, model lifecycle, cancellation, retries, and retention tests pass.
5. **Windows live gate:** signed-in interactive Windows 10/11 environment proves shortcut, mic, active-field insertion, tray, sleep/wake, and installer behavior.
6. **macOS live gate:** interactive Intel or Apple Silicon macOS 12+ environment proves shortcut, mic, Accessibility permission, active-field insertion, menu bar, sleep/wake, and app bundle behavior. Release requires evidence for both architectures or an explicit support re-scope.
7. **Offline/privacy gate:** all required workflows pass with outbound traffic denied after model installation; process-level traffic and filesystem evidence are retained.
8. **Performance/quality gate:** latency, memory, WER/task-success, insertion reliability, and stress thresholds in the behavior matrix pass on declared reference hardware.
9. **Release gate:** clean checkout builds; SBOM, notices, model manifests/hashes, checksums, installer smoke tests, upgrade/rollback notes, and user documentation are complete.

## 8. Sequential prompt roster

All prompts are `Pending` until STS execution records otherwise.

### Phase G — Governance and foundation decision

#### FF-G1 — Freeze clean-room requirements and source ledger

Objective: Review the clean-room policy and behavior matrix, convert every P0/P1 claim into an independently authored acceptance fixture, and record public-source URLs with retrieval dates.

Gate: No fixture contains proprietary output, assets, screenshots, or non-public observations; provenance review passes; unresolved legal/naming questions are explicit blockers rather than assumptions.

#### FF-G2 — Audit and pin the upstream foundation

Objective: Audit current Handy and OpenWhispr releases/commits for build health, licenses, dependency risk, updater/telemetry behavior, architecture, feature gaps, and maintenance cost; choose fork, extraction, or greenfield at each seam.

Gate: A written decision record names exact upstream SHAs, retained copyright notices, dependency/model licenses, known vulnerabilities, patch burden, and a reversible import procedure. Both upstreams receive clean build attempts on Windows and macOS where available.

#### FF-G3 — Create the FreeFlow foundation

Objective: Import only the approved upstream material, replace package identifiers/branding/update endpoints, preserve notices, and establish the Rust/TypeScript workspace, CI, issue templates, and architecture boundaries.

Gate: Clean-room/provenance review passes; no upstream brand asset or network endpoint remains; formatting, lint, typecheck, Rust checks, unit smoke tests, and clean developer builds pass.

#### FF-G4 — Establish storage, configuration, and engine contracts

Objective: Define versioned Rust interfaces for audio, ASR, post-processing, insertion, platform context, and transforms; add SQLite migrations and typed settings with atomic updates.

Gate: Contract tests cover cancellation, timeouts, corrupt settings, migration forward/rollback behavior, and process restart; no feature UI bypasses the service layer.

Milestone G acceptance: the project builds cleanly from source, has traceable provenance, uses original branding, and can evolve without coupling product logic to a single model or OS adapter.

### Phase V — Core local dictation vertical slice

#### FF-V1 — Model manifest and explicit local model installation

Objective: Implement model catalog/manifests, user-initiated download, hash verification, disk-space checks, license disclosure, cancellation/resume, manual install, selection, and deletion.

Gate: Corrupt/hash-mismatch/low-disk/offline/cancel paths pass; no unapproved URL is contacted; at least one approved ASR model installs and loads on Windows and macOS.

#### FF-V2 — Audio capture and dictation state machine

Objective: Implement device selection, resampling, VAD, push-to-talk/toggle/cancel, live levels, processing state, retry-safe audio persistence, and microphone diagnostics.

Gate: Unit/integration state tests pass; live microphone selection and hot-plug pass on both platforms; recording feedback meets the 150 ms p95 gate; cancelled audio is handled by policy.

#### FF-V3 — Local ASR and raw transcript path

Objective: Connect captured audio to the selected local engine, support language selection/detection, surface progress/errors, and preserve recoverable raw transcripts.

Gate: Public-corpus results and latency/memory measurements are recorded for the approved model matrix; the selected default meets frozen thresholds on both reference platforms; zero network is required. By explicit 2026-07-18 rescope, consented project-owned corpus evidence remains mandatory at the later release gate rather than FF-V3.

#### FF-V4 — Reliable cross-application insertion

Objective: Implement active-target capture, direct insertion, clipboard-preserving fallback, manual-copy fallback, whitespace/capitalization boundary handling, paste-last, and safe undo metadata.

Gate: Automated target-binding, clipboard-preservation, secure-field, boundary,
recovery, and content-free undo invariants pass; native Windows and macOS builds
and tests pass. By explicit 2026-07-19 user rescope, the representative live
application matrices and their 98% threshold remain mandatory at FF-R2 rather
than blocking FF-V5.

#### FF-V5 — Tray, original FreeFlow status bar, and recovery UX

Objective: Implement tray/menu-bar controls and an original floating status bar for idle, recording, processing, success, warning, and error states with persisted docking and accessibility labels.

Gate: deterministic work-area/docking/state tests, non-activating native window
configuration, keyboard-accessible tray/Hub controls, screen-reader semantics,
restart persistence, and native Windows/macOS compile/tests pass. The live
multi-monitor, scaling, full-screen, focus-stealing, drag-cancel, and screen-reader
matrices remain mandatory at FF-R2 under the 2026-07-19 consolidation.

#### FF-V6 — Onboarding, permissions, and autostart

Objective: Build original onboarding for local/privacy promise, model choice, mic, macOS Accessibility, shortcuts, first dictation, launch-at-login, and repair diagnostics.

Gate: clean-install, permission-denied/revoked, restart, upgrade, and uninstall-data choices pass on Windows and macOS.

Milestone V acceptance: a non-technical user can install, choose a model, hold a shortcut, speak, and reliably insert or recover local text in common apps with networking blocked.

### Phase P — Personalization and smart formatting

#### FF-P1 — Local history, retention, and deletion

Objective: Implement searchable history with audio/raw/final text, app metadata, latency, retry/copy/delete, paste-last, retention options, and never-store mode.

Gate: storage lifecycle tests prove each retention setting, deletion is irreversible at the app layer, recovery survives crashes, and never-store leaves no transcript/audio after completion.

#### FF-P2 — Dictionary and correction rules

Objective: Implement vocabulary/replacement CRUD, starring, search/sort, deterministic precedence, CSV import/export, and ASR prompt/boost integration only where the engine supports it reliably.

Gate: Unicode, case, boundary, duplicate, size-limit, import rollback, and engine-support tests pass; unsupported boosting is disclosed rather than simulated.

#### FF-P3 — Voice snippets

Objective: Implement phrase-triggered static expansions, CRUD, search/sort, JSON import/export, conflict prevention, and deterministic whole-phrase matching.

Gate: casing, punctuation, whole-word, multiple-trigger, overlap, Unicode, 4,000-character expansion, duplicate, and import-rollback fixtures pass.

#### FF-P4 — Deterministic formatting and voice controls

Objective: Implement spoken punctuation, paragraph/line breaks, list detection, number normalization, filler policy, spacing/capitalization boundaries, backtrack fixtures, and opt-in press-enter.

Gate: frozen multilingual fixtures pass without an LLM where deterministic; ambiguous cleanup preserves raw text; press-enter confirmation and literal-text behavior pass live.

#### FF-P5 — Local app context and style profiles

Objective: Classify active applications locally, add per-category style/settings profiles, optional minimum-necessary surrounding-text context, per-app denylist, and visible context diagnostics.

Gate: context is off by default; secure/denied apps expose no text; process/app categorization and mid-sentence formatting fixtures pass; data never leaves the device.

Milestone P acceptance: P0 behavior rows BF-001 through BF-014 and BF-021/BF-022 pass, and the app remains useful without downloading a text-generation model.

### Phase A — Optional on-device language intelligence

#### FF-A1 — Local transform runtime and model selection

Objective: Add an optional `llama.cpp`-compatible provider, licensed model manifest, resource-aware recommendations, cancellation, timeout, bounded prompt construction, and raw-text fallback.

Gate: no model is silently downloaded; CPU/Metal/available Windows acceleration paths are benchmarked; memory pressure, cancellation, corrupt model, and timeout never lose the raw transcript.

#### FF-A2 — Auto cleanup, backtrack, and styles

Objective: Implement None/Light/Medium/High cleanup and original per-app FreeFlow styles using deterministic preprocessing plus optional local transforms.

Gate: frozen semantic preservation, names/numbers/code, hallucination, brevity, backtrack, and style fixtures pass; raw/final diff and undo are always available.

#### FF-A3 — Selected-text transforms and diff

Objective: Implement configurable transform slots, original prompts, shortcut binding, selected-text replacement, change diff, accept/undo/retry/copy, and writing samples stored locally.

Gate: selection under representative apps, 1,000-word limit, shortcut conflicts, unchanged output, timeout, undo, and diff correctness pass on both platforms.

#### FF-A4 — Local command mode

Objective: Add press-and-hold spoken commands over a selection or cursor for rewrite, translation, and generation; preference changes require a separate explicit confirmation.

Gate: command/regular-dictation state isolation, cancel, processing exclusion, selection replacement, fallback copy, and preference confirmation pass; no calendar or remote account access is implied.

Milestone A acceptance: BF-015 through BF-017 pass on the declared hardware tiers, while core dictation remains independent of the LLM.

### Phase X — Complete local desktop product

#### FF-X1 — Hub navigation and settings completeness

Objective: Finish original Home, History, Dictionary, Snippets, Styles, Transforms, Models, Privacy, Shortcuts, System, About, and diagnostics experiences.

Gate: keyboard navigation, screen-reader names, focus order, high contrast, scaling, empty/error/loading states, settings persistence, and no-network behavior pass.

#### FF-X2 — Scratchpad and local insights

Objective: Implement local notes with search/pin/edit/export/delete and local-only WPM, streak, word-count, app-category, cleanup, and replacement metrics.

Gate: metrics reconcile against history fixtures; notes survive migrations/crashes; export/delete/never-store semantics pass; no leaderboard or account is present.

#### FF-X3 — Developer dictation enhancements

Objective: Add conservative identifier formatting, configurable developer lexicon, and optional filename tagging from explicit user-selected workspace roots.

Gate: code/terminal fixtures preserve exact whitespace and identifiers; indexing honors roots/ignore rules, can be disabled/deleted, stays local, and never reads file contents unless separately approved.

#### FF-X4 — Long-session segmentation

Objective: Support recoverable sessions up to 20 minutes through bounded audio segments, warning/finalize behavior, and immediate next-session readiness.

Gate: ten consecutive 20-minute synthetic/live-approved sessions complete without unbounded memory, crash, lost segment, or unrecoverable transcript.

Milestone X acceptance: every non-parked behavior-matrix row has evidence or an honest documented variance approved as a re-scope.

### Phase R — Hardening and release

#### FF-R1 — Privacy and local-only proof

Objective: Threat-model audio/text/context/model/update flows; run firewall-denied and process-traffic tests; verify storage, logs, crash reports, clipboard restoration, secure fields, and deletion.

Gate: no undisclosed traffic or sensitive logging; high-severity findings are fixed; the privacy statement matches measured behavior.

#### FF-R2 — Reliability, performance, and accessibility matrix

Objective: Run the full hardware/OS/app/model corpus, stress, sleep/wake, device-change, multi-monitor, scaling, upgrade, and accessibility suite, including the retained FF-V2 microphone matrix and FF-V4 cross-application insertion matrices.

Gate: all quantitative release gates pass or a user-approved, documented support re-scope updates the behavior matrix before release.

#### FF-R3 — Supply chain, licenses, and reproducible packaging

Objective: Lock dependencies, generate SBOM/notices/model manifests, audit vulnerabilities, verify hashes, and produce Windows and macOS installers from clean checkouts.

Gate: no incompatible/unknown license or unresolved critical vulnerability; installer checksums and build provenance exist; signing/notarization pass when credentials are available.

#### FF-R4 — Naming, documentation, and release candidate

Objective: Clear the public product name, complete original user/developer/privacy/accessibility/troubleshooting docs, publish source and release artifacts, and record known limitations.

Gate: name clearance is recorded; onboarding and docs match the binaries; a new user completes the core workflow on each platform; final parity audit and DEVLOG reconcile to exact release SHAs.

Milestone R acceptance: a free, open-source, locally operated Windows/macOS release is reproducibly buildable, installable, private by measured behavior, and functionally complete against the approved non-parked matrix.

## 9. Independently approvable milestones

| Milestone                   | Prompts             | Usable cut                                                               |
| --------------------------- | ------------------- | ------------------------------------------------------------------------ |
| G — Audited foundation      | FF-G1 through FF-G4 | Clean, buildable, traceable project foundation                           |
| V — Local dictation alpha   | FF-V1 through FF-V6 | Install → speak → local transcript → reliable insertion/recovery         |
| P — Personalized beta       | FF-P1 through FF-P5 | History, privacy controls, dictionary, snippets, formatting, app context |
| A — Local intelligence beta | FF-A1 through FF-A4 | Optional on-device cleanup, styles, transforms, and command mode         |
| X — Feature-complete RC     | FF-X1 through FF-X4 | Full local desktop product against non-parked matrix                     |
| R — Public release          | FF-R1 through FF-R4 | Hardened, licensed, packaged Windows/macOS release                       |

## 10. Prerequisites and blockers

- Windows live-test host with microphone and interactive desktop.
- macOS Apple Silicon and Intel live-test access before claiming both architectures. Lack of Intel hardware blocks an Intel release claim but does not block earlier platform-independent prompts.
- Sufficient disk for multiple 0.5–3 GB ASR/LLM candidates during benchmarking.
- Code-signing and Apple notarization credentials for trusted public installers; without them, only unsigned developer artifacts can be completed.
- Qualified review before any activity outside the clean-room policy or before relying on a contested public product name.

## 11. Parked scope and override points

The following require a PSPR addendum rather than silent expansion: mobile, meeting/system-audio recording, speaker diarization, calendar/reminders, self-hosted sync, team sharing, plugin SDK, cloud/BYOK providers, screen capture, Linux, Windows ARM, and direct comparative testing against Wispr services.

The user may override supported OS versions, release license, UI framework, upstream choice, model defaults, hardware tiers, or parked scope. Every override must update affected gates and preserve plan history.

## 12. Completion criteria

The initiative is complete only when:

1. all approved roster prompts and milestone gates pass;
2. the behavior matrix contains evidence for every non-parked row and honest variances for approved re-scopes;
3. Windows and macOS release artifacts build from clean source, install, run, and pass live-system gates;
4. core dictation and all claimed local features work with networking blocked after model installation;
5. source, notices, SBOM, model licenses/hashes, privacy documentation, user documentation, and known limitations are published;
6. the development log records exact commits and verification evidence; and
7. no prohibited proprietary material, branding, or restricted observation is present.

## 13. Plan history

| Date       | Version           | Change                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 | Execution impact                                                                                                                                                                                                                            |
| ---------- | ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 2026-07-17 | 0.1               | Initial clean-room PSPR, public behavior matrix, upstream reuse strategy, gates, milestones, and exclusions                                                                                                                                                                                                                                                                                                                                                                                                                            | None; execution remains unauthorized                                                                                                                                                                                                        |
| 2026-07-17 | 0.1 planning note | Added an upstream foundation evidence brief with current candidate SHAs and a preliminary Handy-first decision                                                                                                                                                                                                                                                                                                                                                                                                                         | None; FF-G2 remains pending and execution unauthorized                                                                                                                                                                                      |
| 2026-07-17 | 0.2               | User authorized STS; FF-G1 began with public forum/case-study evidence and frozen P0/P1 fixtures                                                                                                                                                                                                                                                                                                                                                                                                                                       | Execution authorized in roster order; clean-room policy remains binding                                                                                                                                                                     |
| 2026-07-17 | 0.2 execution     | FF-G1 gate passed at `2624d411f4589974513854c16b4cfc0511d4d178`; 19 P0/P1 behaviors map to 19 original fixtures and nine forum sources are classified as unverified hypotheses                                                                                                                                                                                                                                                                                                                                                         | FF-G2 is now in progress                                                                                                                                                                                                                    |
| 2026-07-17 | 0.3 execution     | FF-G2 audited exact Handy/OpenWhispr SHAs on Windows, accepted Handy as the foundation, recorded build/license/vulnerability evidence and a reversible import in ADR-0001, installed Vulkan SDK 1.4.350.0, and verified a local Parakeet evaluation artifact; decision committed at `2ca3ac3aaa7158d2f1b29021254170bb26e59940`                                                                                                                                                                                                         | FF-G3 is next; the eight Handy RustSec findings and all upstream branding/network identities are mandatory first-patch scope                                                                                                                |
| 2026-07-18 | 0.4 execution     | FF-G3 imported audited Handy commit `d861e24bc825c699ccf7215a430684c6e322131c` as a two-parent merge, replaced package identity and all distributed branding/media, removed updater and model-download network surfaces, preserved notices, added Windows/macOS CI and provenance gates, and passed the Windows clean developer build at `0dd27b26c6d3b54407bad89789e62ae08978ef60`                                                                                                                                                    | FF-G4 is next; macOS CI/live build evidence remains unavailable locally and is retained as a release blocker rather than claimed complete evidence                                                                                          |
| 2026-07-18 | 0.5 execution     | FF-G4 added versioned Rust contracts for audio, ASR, post-processing, insertion, platform context, and transforms; reversible transactional SQLite migrations; typed crash-safe atomic settings; and an enforced frontend service boundary at `ce9c7df66a607feceb6a63c970aa336b17c5bc9c`                                                                                                                                                                                                                                               | Milestone G is complete on Windows and FF-V1 is next; concrete manager adapters remain owned by their vertical-slice prompts, while macOS build/live evidence remains a release blocker                                                     |
| 2026-07-18 | 0.6 execution WIP | FF-V1 added a consent-bound immutable Parakeet manifest, exact-size/SHA verification, disk checks, resumable and cancellable direct transfer, verified manual installation, durable receipts, selection/deletion integration, license disclosure, tamper detection, Windows manual-import and direct-download live evidence, and an actionlint-clean dual-architecture macOS live workflow with a consent-gated feature-branch push route                                                                                              | Apple Silicon passed hosted run `29648550447`; Intel exposed a missing imported ORT prebuilt before FreeFlow ran, so an authorized workflow-only link-dependency correction is pending; FF-V2 must not begin before Intel passes            |
| 2026-07-18 | 0.7 execution     | FF-V1 closed at implementation commit `ccac099de6b3cb86fdbfdea1fc57d05a5b8078b2` with Intel gate correction `0baeb5320d0128fa9bb7146c69905d498f605d46`; hosted run `29649911938` passed the consented Parakeet install/load/session path on both macOS architectures; `origin/main` was initialized at completed FF-G4 closeout `0bb8d0f9f7233d0529931984e31c746f6ab2d5f9`; CI repair commits `a1d13f8befbf8d672e59d406142231a735ce71e7` and `5d269096ae9c976719f1e7aaebc3c7a8fc8bd15b` produced full hosted run `29650991927` success | FF-V1 is complete and FF-V2 is next; imported Intel ORT packaging/removal remains later release work and is not misrepresented as complete                                                                                                  |
| 2026-07-18 | 0.8 execution WIP | FF-V2 gate candidate `af7a3b2746737eec869344cbb5f2387b0d55ef75` adds truthful selected-device failure/rollback, typed microphone diagnostics and dictation lifecycle events, atomic pre-ASR WAV persistence with restart recovery, explicit cancellation policy, a commit-pinned bundled Silero V4 artifact with hash/license provenance, and a production-path headless microphone verifier                                                                                                                                           | Windows default-microphone capture passed 50/50 cycles and cancellation returned idle; interactive 150 ms feedback, first-word, physical hot-plug/contention, sleep/wake, and macOS live evidence remain required, so FF-V2 is not complete |
| 2026-07-18 | 0.9 execution WIP | FF-V2 correction `e6e44dcd969d5da0ee0ceb8c2f38a471d3f9db8a` separates headless microphone-ready timing from the interactive 150 ms feedback gate and measures application feedback from shortcut receipt; a second Windows headless run passed 50/50 captures with 164.21 ms microphone-ready p95 and explicit `feedback_gate_measured: false`                                                                                                                                                                                         | FF-V2 remains in progress; no interactive Windows or macOS evidence is inferred, and the foreground-dependent matrix is deferred so the user can continue using the host                                                                    |
| 2026-07-18 | 1.0 execution WIP | The user explicitly deferred the remaining FF-V2 live matrix to a separate macOS machine and directed STS to begin FF-V3 immediately from clean commit `2361db37020e779865d50fc9886cff17e3790677` on `codex/ff-v3-local-asr`; FF-V3 freezes its quality/resource thresholds before retained benchmarking and adds a headless public/owned corpus evaluator                                                                                                                                                                             | This is an authorized roster-order deviation, not an FF-V2 pass. FF-V2 stays incomplete; FF-V3 is in progress and FF-V4 must not begin until FF-V3 passes                                                                                   |
| 2026-07-18 | 1.1 execution WIP | The user explicitly moved the consented project-owned dictation corpus from the FF-V3 completion gate to the later release gate. FF-V3 continues to require public-corpus quality, latency, memory, exact model identity, and real zero-network evidence on Windows and macOS; owned-corpus capture/evaluation remains mandatory before release rather than being waived.                                                                                                                                                              | This rescope permits unattended hosted FF-V3 completion without manufacturing synthetic "owned" evidence. FF-V4 remains blocked until the revised FF-V3 gate passes                                                                         |
| 2026-07-18 | 1.2 execution     | The revised FF-V3 gate passed unchanged on Windows x86_64, Apple Silicon macOS, and Intel macOS in hosted run `29672022926` at exact commit `10b6744cef2ac41bfc4f7f8e6106b7f2f354abf5`; evidence closeout `f4d2a0cbaec389a987be1d03f3a8d8084f7799c9` records public-corpus quality, frozen latency/memory thresholds, immutable model identity, executable identity, and real OS-enforced zero-network probes.                                                                                                                         | FF-V3 is complete and FF-V4 is next. FF-V2 remains user-deferred rather than passed, and consented project-owned corpus evidence remains a mandatory later release gate.                                                                    |
| 2026-07-18 | 1.3 execution WIP | FF-V4 candidate `5877aacea9aa2e9cb2be832cf5df2f77c981a2fb` implements target-guarded direct-first insertion, lossless-text-only clipboard fallback, explicit manual recovery, boundary formatting, paste-last, and content-free undo metadata. Its live gate is frozen in `docs/verification/FF-V4-LIVE-INSERTION-GATE.md` before interactive results are observed.                                                                                                                                                                    | Structural gates pass on Windows, but the 100-attempt representative application matrices and secure-field/clipboard live proofs remain mandatory on Windows and macOS. FF-V4 is not complete and FF-V5 must not begin.                     |
| 2026-07-19 | 1.4 execution     | Under the user's stem-to-stern direction, FF-V4's deterministic/native slice closed while its unmodified interactive Windows/macOS insertion matrix moved to FF-R2.                                                                                                                                                                                                                                                                                                                                                                    | FF-V5 may begin; the move changes timing only and does not waive the release matrix.                                                                                                                                                        |
| 2026-07-19 | 1.5 execution     | FF-V5 candidate `b02751dc4e0edb06ec0827dfc3fbcdb24e2d7d6d` added the original status bar, persistent docking, accessible lifecycle states, state-bearing tray controls, and keyboard Hub navigation. Hosted run `29680441253` passed native Windows/macOS, provenance, and security; closeout `26fc961e23e97e5d358b21f619d4464c37cf02f7` retained the interactive matrix at FF-R2.                                                                                                                                                     | FF-V5 implementation is complete and FF-V6 is next; live multi-monitor/scaling/full-screen/focus/drag/screen-reader evidence remains required at FF-R2.                                                                                     |
| 2026-07-19 | 1.6 execution     | FF-V6 candidate `bcea051e8f3435eb927502d83fb2c14f7c6b091c` added durable staged onboarding, privacy disclosure, permission repair, model/completion separation, shortcut and verified launch-at-login preferences, production-event first-dictation proof, repair diagnostics, and explicit uninstall-data choices. Hosted run `29690161596` passed native Windows/macOS, provenance, and security.                                                                                                                                    | Milestone V implementation is complete and FF-P1 is next. The signed-build clean-install/permission/restart/upgrade/autostart/first-dictation/uninstall matrix remains mandatory at FF-R2 before public release.                            |
| 2026-07-19 | 1.7 execution     | FF-P1 candidate `20b03120c1cbe6b9633cdd47c4f2b11b588b73e5` added searchable raw/final history with app and performance metadata, strict audio-plus-row deletion, every retention policy, clear-all, crash recovery, and never-store cleanup. Hosted run `29692703288` passed native Windows/macOS, provenance, and security.                                                                                                                                                                                                           | FF-P1 is complete and FF-P2 is next. Retained cross-platform interactive release matrices remain mandatory at FF-R2.                                                                                                                        |
| 2026-07-19 | 1.8 execution     | FF-P2 candidate `fc66b424fbe33826f77f3587d751714cad04ed6e` replaced the legacy custom-word array with a reversibly migrated SQLite dictionary, Unicode-aware deterministic rules, starring/search/sort, atomic CSV transfer, one-time legacy migration, and capability-gated Whisper prompting. Hosted run `29694672120` passed native Windows/macOS, provenance, and security.                                                                                                                                                        | FF-P2 is complete and FF-P3 is next. Unsupported engine boosting remains explicitly disclosed rather than simulated; retained cross-platform interactive release matrices remain mandatory at FF-R2.                                        |
| 2026-07-19 | 1.9 execution     | FF-P3 candidate `e753b1527485756ae7424b337e4bacbf03d47955` added reversibly migrated voice snippets, typed CRUD/search/sort, atomic versioned JSON transfer, Unicode-aware deterministic phrase matching, exact post-process expansion, and a complete Advanced settings UI. Hosted run `29696619451` passed native Windows/macOS, provenance, and security.                                                                                                                                                                           | FF-P3 is complete and FF-P4 is next. Retained cross-platform interactive release matrices remain mandatory at FF-R2.                                                                                                                        |
| 2026-07-19 | 2.0 execution     | FF-P4 candidate `f1d66200f1b345a8a82157bd78965d84044dac2c` added deterministic multilingual spoken formatting, corrections, effective-language filler policy, raw/final separation, and confirmed utterance-final submit. Hosted run `29698366361` passed native Windows/macOS, provenance, and security.                                                                                                                                                                                                                              | FF-P4 is complete and FF-P5 is next. The retained interactive press-enter application matrix remains mandatory at FF-R2.                                                                                                                    |

## 14. Public research basis

- [Wispr Flow Terms of Service](https://wisprflow.ai/terms-of-service)
- [Wispr Flow features](https://wisprflow.ai/features)
- [Wispr Flow desktop navigation](https://docs.wisprflow.ai/articles/5096240724-navigating-the-wispr-flow-app-desktop-ios-and-android)
- [Wispr Flow data controls](https://wisprflow.ai/data-controls)
- [Wispr Flow system requirements](https://docs.wisprflow.ai/articles/1036674442-supported-devices-and-system-requirements)
- [Handy repository](https://github.com/cjpais/handy)
- [OpenWhispr repository](https://github.com/OpenWhispr/openwhispr)
- [whisper.cpp repository](https://github.com/ggml-org/whisper.cpp)
- [llama.cpp repository](https://github.com/ggml-org/llama.cpp)

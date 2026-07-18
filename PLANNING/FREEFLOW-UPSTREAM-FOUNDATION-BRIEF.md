# FreeFlow Upstream Foundation Brief

Status: Superseded by `docs/architecture/ADR-0001-UPSTREAM-FOUNDATION.md`
Snapshot date: 2026-07-17 (America/Los_Angeles)
Decision owner: FreeFlow PSPR FF-G2

## Question

Should FreeFlow begin from Handy, OpenWhispr, selective extraction, or a greenfield desktop shell?

## Snapshot evidence

The values below are volatile GitHub metadata captured through the public GitHub API. FF-G2 must refresh them and build the exact candidate commits before making the final pin.

| Attribute | Handy | OpenWhispr |
|---|---|---|
| Repository | [cjpais/Handy](https://github.com/cjpais/handy) | [OpenWhispr/openwhispr](https://github.com/OpenWhispr/openwhispr) |
| Candidate commit | `d861e24bc825c699ccf7215a430684c6e322131c` | `08ae3a8d2d59fb770fd6efbffe5ae22db25021bc` |
| Latest release at snapshot | `v0.9.3` (2026-07-15) | `v1.7.5` (2026-07-11) |
| Declared license | MIT | MIT |
| Primary shell | Tauri 2 | Electron 41 |
| Primary implementation | Rust plus React/TypeScript | Node/Electron plus React/TypeScript and native C/Swift helpers |
| Git tree paths | 444 | 763 |
| GitHub repository size | 11,386 KB | 36,754 KB |
| Stars / forks at snapshot | 26,780 / 2,309 | 4,616 / 648 |
| Open issues at snapshot | 179 | 230 |

Repository size, stars, and issue counts are orientation signals, not quality gates.

## Quantified OpenWhispr import surface

At candidate `08ae3a8d2d59fb770fd6efbffe5ae22db25021bc`, public manifests and the recursive Git tree show:

| Measure | Count |
|---|---:|
| Runtime npm dependencies | 60 |
| Development npm dependencies | 21 |
| Named `download:*` package scripts | 20 |
| Named `compile:*` package scripts | 14 |
| Native/platform source paths in the narrow C/Swift/helper scan | 25 |
| Inference-provider implementation files | 12 |
| Standalone `*.test.js` files | 59 |
| Main-process entry file size | 60,797 bytes |

A path-name classification provides a second view of removal/isolation work. These categories overlap and must not be added together:

| Subsystem signal | Matching source/build files |
|---|---:|
| Authentication, billing, enterprise, team, workspace, referral and invitation | 34 |
| Meeting, system audio, diarization and speaker identity | 45 |
| Agent, chat and tool execution | 44 |
| Notes, synchronization, embeddings, Qdrant and vector search | 43 |
| Cloud/provider/network integrations | 64 |
| Native helpers and platform build/download tooling | 75 |

For comparison, Handy's captured tree has 444 paths, 27 frontend runtime dependencies, 14 frontend development dependencies and 14 narrowly classified platform-native/build paths. Handy also has Rust dependencies that require a separate transitive audit; these counts do not claim its total supply chain is only 41 packages.

### Why this is substantial rather than cosmetic

Importing OpenWhispr wholesale would require one of two costly choices:

1. **Adopt Electron/Node as FreeFlow's shell.** This discards the settled Tauri/Rust direction and imports Electron lifecycle, IPC, native-module ABI, Node 24, Electron Builder, sidecar supervision and a larger idle/resource/security surface.
2. **Port OpenWhispr into Tauri/Rust.** React views and pure TypeScript utilities may transfer, but main-process services, Node native modules, better-sqlite3 access, WebAudio behavior and native helper orchestration must be redesigned behind Rust commands and events. That is a port, not a drop-in import.

Even if unused UI is hidden, cloud/auth/meeting services can still be referenced by preload IPC, database migrations, settings schemas, startup jobs, update/build scripts and background process managers. The removal gate must prove they are unreachable and absent from built artifacts, not merely inaccessible from navigation.

The 20 download scripts also make a clean build a supply-chain event: whisper.cpp, llama-server, sherpa-onnx, Qdrant, embedding/VAD/diarization models, yt-dlp, AEC and Windows helpers each need source, version, hash, license, platform and update-policy records. FreeFlow's default build must not silently fetch this full set.

The meeting stack adds consent, system-audio permission, acoustic echo cancellation, speaker embeddings/identity, retention and calendar concerns that the initial PSPR intentionally parks. The agent/tool stack adds prompt-injection and external-action boundaries. The workspace/auth/billing stack adds remote data models and account state that directly conflict with the no-account P0 path.

Therefore OpenWhispr's complexity is not that it has "many files." It is that several mature products share one process graph and release pipeline. Extracting only dictionary/snippet testable logic or local llama-server management is plausible; importing the whole application and subtracting features is higher risk than extending Handy.

## Capability comparison

| Area | Handy evidence | OpenWhispr evidence | Planning assessment |
|---|---|---|---|
| Local-first posture | README describes entirely local transcription; local model catalog and hash/download code; no account required | Local Whisper/Parakeet and local reasoning exist, but cloud/auth/team paths are first-class | Handy aligns more closely with FreeFlow’s default trust boundary |
| Windows and macOS | Tauri configurations, Windows and macOS target dependencies, installers, autostart, permissions | Electron Builder targets, platform-native shortcut/paste/audio helpers | Both are viable; each still requires live builds |
| Audio and VAD | Rust `cpal`, resampler, Silero VAD, audio manager/coordinator | Web/native audio managers, platform taps, VAD, meeting AEC helpers | Handy has the smaller core-dictation surface |
| ASR | `transcribe-cpp` Whisper-family plus `transcribe-rs` ONNX engines; Metal on macOS and Vulkan/dynamic backends on Windows x64 | whisper.cpp, sherpa-onnx/Parakeet, CUDA/Vulkan managers and sidecars | Both cover local ASR; Handy integrates it in the Rust core |
| Shortcut and insertion | Tauri/global shortcut, `rdev`, `handy-keys`, `enigo`, clipboard and input modules | Dedicated C/Swift key listeners, fast-paste helpers and clipboard restoration tests | Handy is the simpler starting seam; OpenWhispr is useful as a robustness reference |
| Local history | `rusqlite`, migrations, history manager/commands/settings | `better-sqlite3`, history view and transcription services | Both have reusable concepts; schemas require audit |
| Dictionary | Custom-word settings and correction threshold | Dictionary view/service, correction learner and echo filter | OpenWhispr has broader observable coverage |
| Snippets | No dedicated snippet files in the captured tree | Snippets view/service/database tests and matching utilities | FreeFlow must implement this at its own service seam; OpenWhispr may inform tests |
| Post-processing | Prompt settings and an LLM client oriented around provider/API paths; optional Apple Intelligence code | Bundled/downloadable llama-server path plus many cloud providers | FreeFlow needs a local-only provider contract; neither path should be accepted unchanged |
| Notes, agents, meetings | Not a core focus | Extensive notes, local reasoning, agent, meeting, diarization and calendar code | Valuable later reference, but importing it would violate initial scope discipline |
| Test/build surface | Rust checks, Playwright, build and release workflows | Node helper tests, CodeQL, platform native builds and release workflows | Handy is smaller; neither test suite proves FreeFlow’s live gates |

## Supply-chain and adaptation risks

### Handy

- The MIT license does not license the Handy name, logo, icon, or brand assets. Every such asset and identifier must be replaced.
- The Tauri configuration enables `macos-private-api`; FF-G2 must determine exactly why and whether FreeFlow can remove it.
- Several Rust dependencies are Git-sourced or forked (`rdev`, `vad-rs`, `rodio`, `hf-hub`, `tao`, `tao-macros`, `tauri-nspanel`). Exact revisions, licenses, maintenance status, and release reproducibility need verification.
- Updater endpoints, signing keys, sponsor content, release notes, sounds, and package identifiers must not survive the import.
- Existing post-processing can make network requests. FreeFlow’s P0 path must remove or hard-disable network-backed behavior until a separately approved extension exists.
- Model weights have licenses separate from the application. A model cannot enter the catalog until its license, source, hash, size, and redistribution posture are recorded.
- Current upstream supports more platforms than the initial FreeFlow scope, including Linux and Windows ARM paths. FF-G2 must choose whether to preserve dormant support or remove it to reduce release burden.

### OpenWhispr

- The application brings a much larger Node/Electron and native-sidecar surface, including authentication, billing/team, cloud inference, agents, meetings, calendar, vector search, Qdrant, yt-dlp, diarization, system audio and multiple provider SDKs.
- Build and development scripts automatically download numerous binaries and models. Every URL, checksum, license, platform artifact and execution boundary would need review.
- Importing the application wholesale would make local-only verification and dependency minimization harder.
- Useful dictionary, snippet, local-reasoning, clipboard, and native-helper code remains MIT-licensed, but file-level attribution and notices must be preserved if code is extracted.

## Preliminary decision

Use **Handy as the primary foundation candidate**, subject to FF-G2’s clean Windows and macOS builds and file-level audit.

Use **OpenWhispr as an architecture and test-reference candidate**, not as the application foundation. Default to independent FreeFlow implementations at Tauri/Rust service seams. Extract OpenWhispr code only when a narrowly scoped file is materially better than a reimplementation, fits the selected stack, passes dependency/provenance review, and retains its MIT copyright notice.

Do not use a greenfield shell unless FF-G2 proves Handy’s required removal/rebrand patch is riskier than retaining its audio, shortcut, inference, history, model, and packaging foundations.

## Proposed reversible import procedure for FF-G3

If FF-G2 confirms Handy:

1. Record the final commit SHA and tree SHA in the development log and dependency/provenance ledger.
2. Add a read-only `upstream-handy` remote and fetch the exact commit and tags.
3. Base FreeFlow history on that commit rather than copying an untraceable source snapshot.
4. Preserve Handy’s MIT license and copyright notice.
5. In the first FreeFlow-owned commit, replace all package identifiers, updater URLs/keys, analytics or remote-provider defaults, brand assets, sounds, sponsor/referral material, release notes, and user-facing names.
6. Add `NOTICE.md` identifying retained upstream work and a machine-readable dependency/model manifest.
7. Run a forbidden-string/endpoint/asset gate before any feature work.
8. Keep `upstream-handy` fetch-only; never push FreeFlow branches to it.

The exact Git commands belong in FF-G2’s approved decision record after clean builds and must not be executed from this planning brief alone.

## FF-G2 evidence still required

- Clean checkout builds on live Windows and macOS hosts at the refreshed candidate SHA.
- Dependency and transitive license inventory, including model licenses and native binaries.
- Vulnerability, secret, updater, telemetry, network endpoint and build-download audit.
- Measured baseline for launch, shortcut response, ten-second dictation latency, memory and paste reliability.
- Exact list of brand/proprietary assets and endpoints to remove.
- Maintainer/update strategy: periodic upstream merge, cherry-pick, or permanent fork.
- Final fork-versus-extraction ADR with an exact SHA and rollback procedure.

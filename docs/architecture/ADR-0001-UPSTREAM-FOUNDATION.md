# ADR-0001: Upstream foundation

Status: Accepted for FF-G2
Date: 2026-07-17
Decision: Import Handy as FreeFlow's primary foundation; use OpenWhispr only
as a public architecture and test reference unless a later file-level
extraction decision is approved.

## Context

FreeFlow needs a Windows and macOS desktop shell, local audio capture, local
ASR, shortcuts, insertion, history, model management, and packaging. The
settled stack is Tauri 2, Rust, React, TypeScript, and Vite. The initial product
must not require an account, cloud transcription, telemetry, or a language
model for basic dictation.

This record closes PSPR prompt FF-G2. It supersedes the preliminary decision
in `PLANNING/FREEFLOW-UPSTREAM-FOUNDATION-BRIEF.md`.

## Exact audited inputs

| Upstream | Version | Commit | Tree | License notice to retain |
|---|---|---|---|---|
| [cjpais/Handy](https://github.com/cjpais/Handy) | `v0.9.3` | `d861e24bc825c699ccf7215a430684c6e322131c` | `5b91bf5be354d7fa6f267a4f47045db0cfc718bc` | MIT, Copyright (c) 2025 CJ Pais |
| [OpenWhispr/openwhispr](https://github.com/OpenWhispr/openwhispr) | `v1.7.5` | `08ae3a8d2d59fb770fd6efbffe5ae22db25021bc` | `2cfb698e9e076d9e1b3a5a4947983b7c51326691` | MIT, Copyright (c) 2024 OpenWhispr Team, required only if code is later extracted |

Both clean audit clones remained source-clean after dependency installation,
builds, and tests. Generated and downloaded audit artifacts were ignored by
their upstream repositories. No FreeFlow implementation code was copied from
OpenWhispr.

## Decision

Use Handy as a permanent, periodically merged fork. Import the exact audited
commit with its Git lineage and retain its MIT notice. The first FreeFlow-owned
merge commit must simultaneously establish provenance, remove or replace
upstream branding and release credentials, and hard-disable network-backed
runtime behavior outside explicit user-initiated model/update downloads.

Use OpenWhispr as a public reference for failure cases and tests around
clipboard restoration, hotkeys, dictionary behavior, local server recovery,
and platform helpers. Default to a new implementation at FreeFlow's Rust
service seams. A future source extraction requires a separate file-level
record naming the file, commit, license, retained notice, dependency impact,
and reason extraction is preferable to reimplementation.

Do not use a greenfield shell. Handy's retained audio, ASR, shortcut,
insertion, history, settings, model, and packaging seams are more valuable
than the bounded removal/rebrand patch.

## Windows build evidence

Audit host: Windows 11 build `10.0.26200`, Node `24.15.0`, Rust/Cargo `1.96.1`,
Visual Studio Build Tools 2026/MSVC `19.50`, CMake `4.2`, Bun `1.2.20`, and
Vulkan SDK `1.4.350.0`.

### Handy

| Check | Result |
|---|---|
| `bun install --frozen-lockfile` | Pass; 360 packages installed |
| `bun run lint` | Pass |
| `bun run format:check` | Pass |
| `bun run build` | Pass; TypeScript and Vite production build |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Pass after installing the required Vulkan SDK; 9m16s; one upstream unused-assignment warning |
| `bun run tauri build --no-bundle` | Pass; 23m25s; `handy.exe` produced |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Pass; 119 passed, 0 failed; 11m39s; two upstream warnings |

The Windows no-model runtime set contains one executable and fourteen DLLs,
144.6 MiB total. The executable is 46.2 MiB. The native transcription build
requires the Vulkan SDK because `transcribe-cpp-sys` enables Vulkan on Windows;
without the SDK CMake fails on the Vulkan library, headers, and `glslc`.

### OpenWhispr

| Check | Result |
|---|---|
| `npm ci` | Pass; 865 packages installed; Electron rebuilt `better-sqlite3` |
| `npm run quality-check` | Pass; ESLint, Prettier, and TypeScript |
| `npm test` | Fail; 526 passed, 4 failed, 17 skipped out of 547 |
| `npm run build:renderer` | Pass |
| `npm run compile:native` | Pass, but the default shell could not find MSVC and downloaded prebuilt Windows helpers instead |
| `npm run pack` | Fail after resource preparation because repository config invokes OpenWhispr's Azure Trusted Signing profile |
| unsigned `electron-builder --dir` audit override | Pass after all Windows-only helpers were fetched; override was temporary and not retained |

The four Windows test failures cover corrupt-ZIP rejection, executable-mode
expectations for a downloaded CUDA binary, and two child-process termination
timeout cases. Four additional database tests skip because `better-sqlite3`
was rebuilt for Electron rather than the host Node runtime.

The successful unsigned output contains 252 files, 18 executables, and 45
DLLs, totaling 766.3 MiB before user ASR or reasoning weights. The main
executable alone is 213.4 MiB. Its duplicate-dependency report includes large
AI provider, AWS, authentication, editor, and React graphs.

No macOS host is available in this execution environment. The PSPR requires
build attempts on each platform where available, so FF-G2 records the Windows
attempts as complete. A clean macOS build and live permissions check remain a
mandatory release gate and must run in macOS CI and on a real signed test app.

## Why OpenWhispr import is substantially complex

At the audited commit OpenWhispr has 763 repository paths, 60 runtime npm
dependencies, 21 development dependencies, 20 `download:*` scripts, 14
`compile:*` scripts, 25 narrowly classified native/platform source paths, 12
inference-provider implementations, and a 60,797-byte main-process entry file.

Its features share an Electron main process, preload IPC, renderer settings,
database migrations, startup jobs, and release pipeline. Removing screens is
not sufficient. A wholesale import would require either:

1. abandoning the settled Tauri/Rust shell and accepting Electron/Node ABI,
   lifecycle, IPC, sidecar, resource, and security costs; or
2. porting main-process services, native modules, SQLite access, WebAudio, and
   helper supervision into Rust commands and events.

The observed package path downloaded or staged Whisper, llama.cpp plus a DLL
family, sherpa-onnx, yt-dlp, Qdrant, meeting AEC, Silero VAD, diarization
models, NirCmd, and multiple Windows helpers before packaging. It then invoked
the upstream Azure signing identity. That is a mature multi-product release
graph, not a drop-in dictation core.

## Dependency, license, and model findings

### Handy

- The application is MIT licensed, but its name, logos, icons, sounds, sponsor
  material, signing identity, and updater identity are not reusable product
  branding and must be removed or replaced.
- The locked Rust graph contains 841 packages. The dominant declared license
  expressions are MIT, Apache-2.0, and their combinations. Smaller groups
  include MPL-2.0, BSD, ISC, Unicode-3.0, Zlib, BSL-1.0, CDLA-Permissive-2.0,
  and other permissive expressions.
- Seven Git-source packages are pinned: `hf-hub`, `rdev`, `rodio`, `tao`,
  `tao-macros`, `tauri-nspanel`, and `vad-rs`. Their exact revisions are in
  `Cargo.lock`. `tauri-nspanel` omits a manifest license field but ships both
  `LICENSE_MIT` and `LICENSE_APACHE-2.0`; FF-G3 must add the omission to the
  exception ledger rather than silently treating it as metadata-complete.
- The model catalog declares 65 entries: 23 Apache-2.0, 21 MIT, 14 CC-BY-4.0,
  6 `other`, and 1 CC-BY-NC-4.0. Catalog labels are not sufficient proof of
  weight provenance or redistribution rights. FreeFlow will ship no weights
  until each artifact has an independently verified source, revision, size,
  hash, license text, attribution, and redistribution decision.
- A local evaluation copy of `parakeet-unified-en-0.6b-Q8_0.gguf` is recorded
  in `models/README.md` and excluded from Git. Its converted repository
  declares CC-BY-4.0 while the current NVIDIA base-model page declares the
  NVIDIA Open Model License. Redistribution is blocked until reconciled.

### OpenWhispr

- The application is MIT licensed, but its production graph installed 865 npm
  packages and its release output bundles numerous separately licensed native
  binaries and models.
- `npm audit --omit=dev` reports two high-severity findings through
  `onnxruntime-node` -> `adm-zip <0.6.0`: GHSA-xcpc-8h2w-3j85, crafted ZIP
  input causing a 4 GB allocation.
- Native/model URLs, hashes, licenses, and update behavior would require an
  independent ledger before any corresponding artifact could be reused.

## Known Handy vulnerabilities and mandatory first patch

`cargo audit` scanned the exact 841-package lock and found eight
vulnerabilities plus 28 allowed warnings:

| Package | Advisory | Severity/fix |
|---|---|---|
| `quick-xml 0.38.4` | RUSTSEC-2026-0194 | High; upgrade to `>=0.41.0` |
| `quick-xml 0.38.4` | RUSTSEC-2026-0195 | High; upgrade to `>=0.41.0` |
| `rustls-webpki 0.103.9` | RUSTSEC-2026-0049 | Upgrade to `>=0.103.10` |
| `rustls-webpki 0.103.9` | RUSTSEC-2026-0098 | Upgrade to `>=0.103.12` |
| `rustls-webpki 0.103.9` | RUSTSEC-2026-0099 | Upgrade to `>=0.103.12` |
| `rustls-webpki 0.103.9` | RUSTSEC-2026-0104 | Upgrade to `>=0.103.13` |
| `tar 0.4.44` | RUSTSEC-2026-0067 | Medium; upgrade to `>=0.4.45` |
| `tar 0.4.44` | RUSTSEC-2026-0068 | Medium; upgrade to `>=0.4.45` |

The warnings include unmaintained GTK3 bindings, `bincode`, and `fxhash`, plus
yanked WASM packages. FF-G3 must update the resolvable vulnerable packages,
record target-specific/unreachable findings, and fail CI on new unreviewed
RustSec advisories. No FreeFlow release may be cut from the audited lock as-is.

## Required FF-G3 removal and adaptation patch

The import is accepted only with this bounded first patch:

1. Preserve Handy's MIT license and copyright in `LICENSE`/`NOTICE.md`.
2. Replace the application name, package IDs, protocol IDs, filenames, icons,
   sounds, sponsor content, release notes, URLs, and user-facing strings.
3. Remove Handy's GitHub updater URL/key, Azure signing command/account, and
   any remote provider default. Leave updating disabled until FreeFlow owns a
   signed release channel.
4. Make P0 runtime network-deny by default. Permit only explicit, confirmed
   model/update downloads with source, size, hash, license, and destination.
5. Patch or disposition the eight RustSec findings and add repeatable advisory
   and license gates.
6. Pin and ledger every Git dependency; replace forks with released upstream
   crates when behavior and tests permit.
7. Do not ship any model weight. Keep the verified local Parakeet artifact for
   evaluation only until its license discrepancy is closed.
8. Preserve Windows x64 and macOS arm64/x64. Park Linux and Windows ARM release
   packaging until explicitly approved, while avoiding gratuitous source
   deletion that would make upstream merges harder.
9. Retain `macos-private-api`/`tauri-nspanel` only long enough to verify why the
   overlay needs them; remove them if public APIs satisfy the live overlay gate.

## Reversible import procedure

FF-G3 will preserve both histories with a no-commit unrelated-history merge:

```text
git tag ff-g2-pre-import <FF-G2-closeout-SHA>
git remote add upstream-handy https://github.com/cjpais/Handy.git
git fetch upstream-handy d861e24bc825c699ccf7215a430684c6e322131c
git merge --allow-unrelated-histories --no-commit d861e24bc825c699ccf7215a430684c6e322131c
```

Before committing, FF-G3 will keep FreeFlow's canonical README/planning/docs,
retain the upstream MIT license, apply the required removal/adaptation patch,
add `NOTICE.md` and machine-readable provenance/model ledgers, and run the
forbidden-brand/endpoint/asset gate plus all build/test/security gates.

The resulting merge commit has the exact Handy commit as a parent. Rollback is
`git revert -m 1 <FF-G3-merge-SHA>` after verifying the target branch and
working tree. The `upstream-handy` remote is fetch-only; no FreeFlow branch may
be pushed to it.

## Maintenance policy

- Review upstream Handy monthly and before each FreeFlow release.
- Merge or cherry-pick only after replaying the clean-room, forbidden-string,
  network, license, advisory, Windows, and macOS gates.
- Keep FreeFlow-specific contracts and platform adapters narrow so upstream
  audio/ASR fixes remain mergeable.
- Treat OpenWhispr updates as research inputs, never automatic source updates.

## Consequences

FreeFlow starts with a proven local dictation core and smaller native runtime,
while accepting a real rebrand/security/network-hardening patch and ongoing
Git-fork maintenance. OpenWhispr's broader product ideas remain available as
public behavior/test references without importing its Electron process graph.


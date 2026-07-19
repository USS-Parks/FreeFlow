# FF-P1 history, retention, and deletion gate

Date: 2026-07-19  
Scope: deterministic storage lifecycle and production UI/service boundary

## Delivered behavior

- Search paginates over raw text, final text, application id, window title,
  and model id without sending content outside the local SQLite database.
- History records the durable audio reference, raw and final text, application
  metadata, model/language metadata, audio duration, transcription latency, and
  locally derived words per minute.
- Retry, copy, paste-last, save, individual deletion, clear-all, and every
  existing count/time retention policy use the application service boundary.
- `never_store` uses an empty retry row only while transcription is active. On
  success or failure it removes the row and WAV without writing transcript text;
  startup purges interrupted retry rows, partial captures, and unreferenced WAVs.
- Individual, bulk, and retention deletion validate recording names, remove and
  verify audio first, then delete rows in a SQLite transaction. A failed file
  removal cannot be hidden by deleting the database row.

## Frozen lifecycle assertions

The gate is represented by production-path Rust tests:

- all five retention choices select only eligible unsaved rows;
- saved rows survive count and time retention;
- 3-day, 2-week, and 3-month cutoffs are exact and deterministic;
- app-layer deletion leaves neither database row nor audio file;
- path traversal is rejected before the database is touched;
- text and application-metadata search is case-insensitive and paginated;
- migrations advance reversibly to schema version 6 and survive restart;
- atomic captures publish only complete WAVs, reject replacement, and recover
  only valid finalized unreferenced recordings.

## Verification commands

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run check:translations
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings \
  -A unused-imports -A dead-code -A clippy::needless-lifetimes \
  -A clippy::needless-return -A clippy::items-after-test-module \
  -A clippy::manual-repeat-n -A clippy::write-with-newline
bun run tauri build --no-bundle
cargo audit -f src-tauri/Cargo.lock
scripts/check-foundation-provenance.sh
git diff --check
```

The full Rust suite passes 173 runnable tests with the two explicit 731 MB live
model-install tests ignored. The history gate contributes nine focused tests;
the migration gate contributes four and the atomic/recovery utility gate three.
All 460 translation keys are present across 23 catalogs. The frontend production
build, service-boundary check, strict Clippy delta, and formatting gates pass.
RustSec reports no denied vulnerability and 28 allowed upstream warnings. The
optimized Windows executable is 44,207,104 bytes with SHA-256
`7806dc5077326d18deb9a502e143e8cace2f4f153764520ae233c97582195c3e`.

This prompt concerns deterministic local storage behavior and does not require
foreground input automation. The retained cross-platform release matrices remain
owned by FF-R2.

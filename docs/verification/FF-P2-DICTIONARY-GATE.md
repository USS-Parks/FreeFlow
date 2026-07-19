# FF-P2 dictionary and correction-rule gate

Date: 2026-07-19
Scope: local vocabulary, deterministic replacements, and capability-gated ASR prompting

## Delivered behavior

- Dictionary entries live in a dedicated, reversibly migrated SQLite database,
  not the settings document. Existing custom words migrate once and preserve
  their spelling without reappearing after deletion.
- The typed service boundary provides create, read, update, delete, star,
  Unicode-aware search/sort, CSV import/export, and engine-support diagnostics.
- Matching is case-insensitive and Unicode-aware, requires whole word/phrase
  boundaries, preserves surrounding punctuation, and applies the replacement
  exactly as stored. At one position, the longest phrase wins, then starred
  status, then stable entry id; consumed spans cannot be replaced twice.
- CSV import parses quoted commas, quotes, and newlines before opening its write
  transaction. Invalid fields, duplicates, and capacity violations roll back
  the entire import.
- Vocabulary is supplied as an initial prompt only to a loaded Whisper-family
  engine that advertises `InitialPrompt`. Streaming, ONNX, unsupported, and
  non-Whisper engines receive deterministic local rules without simulated
  boosting. The UI discloses this policy.
- Limits are 5,000 entries, 200 Unicode scalar values for a spoken form, and
  4,000 for a replacement.

## Frozen assertions

Production-path tests cover:

- Unicode case matching, exact replacement case, punctuation, and boundaries;
- overlapping phrase precedence and starred-first ordering;
- Unicode-aware search and stable alphabetical/recent/starred sorting;
- ASCII and Unicode case-insensitive duplicate rejection;
- both per-field limits and total dictionary capacity;
- quoted Unicode CSV round trips;
- invalid and duplicate import rollback with no partial rows;
- idempotent legacy settings migration;
- reversible personalization-database migration; and
- prompt attachment only when both Whisper architecture and advertised engine
  support are present.

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

The full Rust library suite passes 182 runnable tests with the two explicit
731 MB live-model installation tests ignored. FF-P2 contributes seven focused
dictionary tests, one engine-support test, and one reversible personalization
migration test. All 481 translation keys are present across 23 catalogs. The
frontend production build, service-boundary check, warnings-denied Clippy
delta, frozen dependency install, formatting, provenance, and diff gates pass.
RustSec reports no denied vulnerability and 28 allowed upstream warnings.

The optimized Windows executable is 44,329,984 bytes with SHA-256
`dbce3160391bf8ed1ea0c06ce8b7bf20b29a3167da4f29edd0834f61adb829b0`.
Hosted native Windows/macOS, provenance, and security evidence will be attached
to this record at the exact candidate commit before FF-P2 is closed.

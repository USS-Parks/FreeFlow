# FF-P3 voice-snippets gate

Date: 2026-07-19
Scope: deterministic phrase-triggered static text expansion

## Delivered behavior

- Voice snippets live beside the dictionary in the reversibly migrated
  personalization database. The typed service boundary provides create, read,
  update, delete, Unicode-aware search/sort, and versioned JSON import/export.
- Trigger matching is case-insensitive and Unicode-aware, requires whole phrase
  boundaries, preserves surrounding punctuation, and inserts expansion text
  exactly as stored. Multiple occurrences expand, including adjacent repeated
  triggers.
- At one position, the longest trigger wins, then stable entry id. Consumed
  spans cannot expand twice, so overlapping triggers resolve deterministically.
- Duplicate triggers are rejected using Unicode lowercase comparison. Import
  validates the complete document, duplicate set, existing conflicts, capacity,
  and every field before committing any row.
- Expansion runs after optional transcription post-processing. The same shared
  output path covers ordinary dictation and history retries, and expanded text
  is persisted as the final output.
- Limits are 1,000 snippets, 100 Unicode scalar values for a name, 200 for a
  trigger phrase, and exactly 4,000 for an expansion.

## Frozen assertions

Production-path tests cover:

- mixed trigger casing, punctuation retention, and whole-phrase boundaries;
- multiple distinct triggers and adjacent repeats of the same trigger;
- deterministic longest-trigger overlap precedence;
- Unicode matching and Unicode case-insensitive duplicate rejection;
- exact 4,000-character expansion and multiline expansion preservation;
- existing-database conflicts and duplicate-within-file rejection; and
- versioned JSON round trip plus all-or-nothing import rollback.

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

The full Rust library suite passes 187 runnable tests with the two explicit
731 MB live-model installation tests ignored. FF-P3 contributes five focused
snippet tests and extends the reversible personalization migration test. All
501 translation keys are present across 23 catalogs. The frontend production
build, service-boundary check, warnings-denied Clippy delta, frozen dependency
install, formatting, provenance, and diff gates pass. RustSec reports no denied
vulnerability and 28 allowed upstream warnings.

The optimized Windows executable is 44,445,696 bytes with SHA-256
`34fa26e8bc387dd9319b2bbf12857678d0fb0df1ba9265e065de0086a0a4bf24`.
Hosted native Windows/macOS, provenance, and security evidence will be attached
at the exact candidate commit before FF-P3 is closed.

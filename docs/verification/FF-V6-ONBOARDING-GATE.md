# FF-V6 onboarding, permissions, and autostart gate

Status: **Local deterministic gate passed; hosted gate pending**

## Implemented proof surface

- A durable `OnboardingStage` checkpoint resumes welcome, permissions, model,
  preferences, or first-dictation setup after a restart.
- Existing completed installations migrate directly to `complete`; upgrades do
  not replay onboarding.
- Permission repair interrupts and then returns to the exact persisted setup
  step. Completed users return to the Hub after repair.
- Selecting or switching a model no longer marks onboarding complete. Only the
  explicit completion command can do so, and it refuses to complete without a
  selected local model.
- The first-dictation step listens to the production typed transcription event
  and remains gated until it receives `completed`.
- Launch-at-login changes apply to the operating system first, are read back,
  and are persisted only when the requested state is confirmed.
- Setup diagnostics report wizard state, model selection, requested and actual
  launch-at-login state, portable mode, and the resolved local data directory.
- The Windows NSIS uninstaller retains its explicit, unchecked-by-default
  **Delete app data** choice and never deletes data during an update. macOS and
  portable keep/delete instructions are documented in `docs/UNINSTALLING.md`
  and linked to the in-app data-folder control.
- All user-facing strings exist in all 23 translation catalogs; untranslated
  catalogs receive the accurate English fallback until qualified localization.

## Deterministic gate

Run from a clean checkout at the candidate commit:

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
```

Hosted Windows and macOS foundation, provenance, and security jobs must pass at
the exact candidate commit.

Local candidate results on Windows x86_64:

- Bun 1.3.14 frozen install checked 366 installs across 442 packages with no
  changes.
- ESLint, frontend service-boundary, Prettier, Rustfmt, 446-key translation
  consistency, TypeScript, and Vite production build passed.
- Rust: 168 passed, 0 failed, 2 explicit 731 MB live-model tests ignored. The
  first attempt exposed Windows temp-directory error 5 in an existing atomic
  settings stress test; the full suite passed with `TEMP` and `TMP` set to the
  project-scoped `C:\tmp\freeflow-ffv6-tests-20260719` directory.
- Strict Clippy passed with only the seven documented inherited lint classes
  allowed.
- RustSec found no denied vulnerability and 28 policy-allowed upstream warnings;
  provenance and whitespace gates passed.
- The optimized no-bundle executable is 44,121,088 bytes with SHA-256
  `69cac522fa3c4013285bfc37ecc2e27e68efbe2c71829335e5e5e5ede68edc54`.

## Retained FF-R2 live matrix

The 2026-07-19 consolidation moves unavailable foreground/platform interaction
to FF-R2 without waiving it. Before public release, retain signed-build evidence
for clean install, microphone denial and regrant, macOS Accessibility denial and
regrant, permission revocation while established and while mid-wizard, restart
at every persisted stage, upgrade from a pre-stage store, launch-at-login across
restart/login, first dictation, Windows keep/delete uninstall choices, and macOS
keep/delete data choices. Verify with networking blocked except for an explicit,
consent-bound model installation.

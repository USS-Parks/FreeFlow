# FF-A3 selected-text transforms and diff gate

Date: 2026-07-19
Scope: local selected-text transforms, exact replacement, diff review, recovery actions, and local writing samples

## Delivered behavior

- Up to eight configurable transform slots bind original local prompts to
  global shortcuts. The three defaults are Polish, Shorten, and Warm tone;
  duplicate shortcut comparisons ignore case and whitespace, and a failed
  shortcut update restores the previous working registration.
- Transform capture is explicit and fail closed. It accepts one ordinary local
  selection only, rejects secure, unknown-security, remote, denylisted,
  identity-less, empty, multi-range, over-1,000-word, and over-12,000-character
  inputs, and does not add selection reading to the ordinary dictation path.
- Selected text and up to five locally stored writing samples are untrusted data
  inside the authored prompt boundary. The local provider uses the already
  consented FF-A1 runtime and never silently downloads a model.
- A non-focus-stealing overlay shows processing, exact changes, unchanged
  output, applied, undone, and failed states. The user can accept, undo, retry,
  copy, or dismiss; pressing the same transform shortcut again accepts a ready
  preview without moving focus.
- Accept revalidates both target identity and the exact current selection before
  replacing it. Retry also requires the original text to remain selected, undo
  restores through the target application's native operation, and failure,
  timeout, unchanged output, or dismissal never replaces text.

## Frozen fixture coverage

Rust fixtures freeze the FF-FIX-016 implementation classes:

- the exact 1,000-word boundary and over-limit rejection;
- shortcut normalization and conflict detection;
- selected-text and writing-sample prompt isolation;
- exact source/output reconstruction from word and whitespace diffs;
- unchanged-output handling;
- secure, remote, denylisted, unknown, empty, and multi-range capture refusal;
- settings migration, slot/sample caps, and bounded local storage; and
- transform timeout/failure preservation through the FF-A1 local runtime.

Platform adapters use Windows UI Automation `GetSelection` and macOS
`AXSelectedText`. The representative Windows/macOS application, native undo,
and selection-replacement matrix remains consolidated in FF-R2 as instructed;
this candidate records automated and structural evidence without claiming that
retained interactive release evidence.

## Verification commands

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run check:translations
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
bun run tauri build --no-bundle
cargo audit -f src-tauri/Cargo.lock
scripts/check-foundation-provenance.sh
git diff --check
```

The local candidate gate passes all 606 translation keys across 23 catalogs,
226 runnable Rust tests with the two explicit 731 MB Parakeet live-install
tests ignored, strict warnings-denied Clippy, native no-bundle build, RustSec
policy, provenance, and diff checks. RustSec reports no denied vulnerability
and 28 policy-allowed upstream warnings. The optimized Windows executable is
45,165,568 bytes with SHA-256
`36aaba53e5a1666a8d41608505392ce4658417bea2b41ede1ab474299d88b387`.
Hosted GitHub Actions run `29708191017` passed Windows and macOS native tests,
provenance, and security at exact candidate
`e22b9022578a9cca02e37e6fb759ff8b37a58dea`.

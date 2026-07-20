# FF-A4 local command mode gate

Date: 2026-07-19
Scope: hold-to-speak local text commands, lifecycle isolation, guarded delivery, cancellation, and preference confirmation

## Delivered behavior

- Command mode has its own configurable global shortcut and is always
  press-and-hold, regardless of the ordinary dictation toggle preference. It
  runs through the existing transcription coordinator, audio manager, and
  cancellation generation so regular dictation and command recording or
  processing cannot overlap.
- Explicit activation captures one safe local selection when present, otherwise
  the current cursor target. Secure, unknown-security, remote, denylisted, and
  identity-less selection targets fail closed. Ordinary dictation still does
  not read selected text.
- The spoken instruction is classified before reaching the verified FF-A1 local
  runtime. Only local rewrite, translation, summarization, and generation work
  is admitted; calendar, messaging, login, account connection, upload, and
  download actions are rejected without implying remote capability.
- Selection output replaces only the exact unchanged selection on the original
  target. Cursor output uses the existing target-guarded insertion path. Every
  target change, policy refusal, unavailable insertion method, or insertion
  error copies the complete output for manual paste instead of losing it.
- Cleanup level, overlay style, push-to-talk, and local application-context
  requests create a pending confirmation session. Settings remain unchanged
  until the user presses Confirm change in the non-focus-stealing overlay;
  dismissal or cancellation applies nothing. Unsupported preference commands
  fail closed.
- Command audio and instructions are not added to dictation history. Escape or
  the overlay cancel action stops capture/processing through the shared
  cancellation path, kills a dropped local-transform process, and changes no
  text or preference.

## Frozen fixture coverage

Rust fixtures freeze the FF-FIX-017 implementation classes:

- selection and cursor intents remain distinct;
- preference requests classify to pending confirmation rather than mutation;
- unsupported preference and remote-account actions are rejected;
- command JSON preserves the classified instruction/data boundary;
- command mode is owned by the regular coordinator while transform shortcuts
  remain outside it; and
- the existing coordinator cancellation and busy-stage fixtures cover release,
  duplicate press, cancel, and processing exclusion.

The representative Windows/macOS selection, cursor, native replacement, manual
copy, and hold/cancel application matrix remains consolidated in FF-R2 as
instructed. This candidate records automated and structural evidence without
claiming that retained interactive release evidence.

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

The local candidate gate passes all 628 translation keys across 23 catalogs,
231 runnable Rust tests with the two explicit 731 MB Parakeet live-install
tests ignored, strict warnings-denied Clippy, native no-bundle build, RustSec
policy, provenance, and diff checks. RustSec reports no denied vulnerability
and 28 policy-allowed upstream warnings. The optimized Windows executable is
45,270,528 bytes with SHA-256
`d22690b52d975ce499fde67ef089f0036d4edac2979312c77da414f50f5a5ed4`.
Hosted Windows/macOS candidate evidence is recorded during closeout.

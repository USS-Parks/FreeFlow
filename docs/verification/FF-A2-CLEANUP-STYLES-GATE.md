# FF-A2 cleanup, backtrack, and styles gate

Date: 2026-07-19
Scope: cleanup strength, original per-application styles, semantic guards, raw/final review, and cleanup undo

## Delivered behavior

- The post-processing shortcut applies one explicit cleanup level: None,
  Light, Medium, or High. None leaves the cleanup stage byte-for-byte; Light
  performs deterministic whitespace repair; Medium and High run the same
  deterministic stage before an optional verified local transform.
- Email, messaging, document, code, terminal, and other application categories
  have original FreeFlow style profiles: Natural, Concise, Warm, Professional,
  or Literal. Classification uses the target captured when dictation starts.
- Spoken punctuation, lists, numbers, and backtrack remain deterministic and
  execute before an optional transform. Snippets still expand after cleanup so
  their saved text is never rewritten by the model.
- Local transform output is rejected unless it stays within a frozen brevity
  bound, preserves capitalized names, numbers, identifiers, paths, URLs, email
  addresses, and code tokens, introduces at most two unsupported word classes,
  and contains no prompt scaffolding. Rejection, cancellation, timeout, model
  failure, and invalid output retain the deterministic pre-transform text and
  the raw engine transcript remains stored separately.
- History presents raw and final text together whenever they differ. “Use raw
  transcript” immediately switches the entry's active review/copy text, and
  “Use cleaned transcript” reapplies the final version without mutating either
  stored value. The target application's normal undo remains available for the
  inserted operation.

## Frozen fixture coverage

Rust fixtures freeze all FF-FIX-015 classes:

- None and Light behavior;
- Medium/High prompt and brevity policy;
- proper-name, number, identifier, path, and code preservation;
- unsupported-word and expansion rejection;
- question/instruction isolation and prompt-scaffolding rejection;
- concise output without invention;
- deterministic backtrack before transformation; and
- distinct Professional and Literal application style instructions.

Settings migration fixtures verify existing profiles receive deterministic
category defaults and cleanup starts at Medium without resetting unrelated
settings. The history UI exposes both stored variants and reversible review.

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

The local candidate gate passes all 569 translation keys across 23 catalogs,
217 runnable Rust tests with the two explicit 731 MB Parakeet live-install
tests ignored, strict warnings-denied Clippy, native no-bundle build, RustSec
policy, provenance, and diff checks. RustSec reports no denied vulnerability
and 28 policy-allowed upstream warnings. The optimized Windows executable is
44,902,400 bytes with SHA-256
`543461f516d5ab2f3eb7be5406596bef63081e4b083832f6cfee69ec100f077b`.
Hosted Windows/macOS candidate evidence is recorded during closeout.

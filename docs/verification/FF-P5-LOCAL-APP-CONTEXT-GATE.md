# FF-P5 local application context gate

Date: 2026-07-19
Scope: local application classification, formatting profiles, and optional surrounding-text context

## Delivered behavior

- FreeFlow classifies the foreground process locally into email, messaging,
  document, code, terminal, or other. Classification uses the process identifier,
  never window or document text.
- Each category has a typed profile for standard, compact, or literal boundary
  formatting, optional surrounding-text access, and optional trailing space.
  Code and terminal default to literal formatting with surrounding text disabled.
- Surrounding-text access has a separate global opt-in that is disabled for new
  and migrated settings. The Windows UI Automation and macOS Accessibility
  adapters do not request text unless the global setting and category profile
  both permit it.
- Exact process identifiers can be denylisted. Remote desktop and screen-sharing
  targets are denied independently. Secure fields and targets whose security
  status cannot be established fail closed.
- The adapter reads at most 16 characters immediately before the insertion
  point. Diagnostics display application metadata, category, policy decision,
  and captured character count; they do not display surrounding text. Disabled,
  denied, remote, secure, and unknown-security diagnostics redact window text and
  report zero captured characters.
- All classification, policy, formatting, settings, and diagnostics paths remain
  inside the existing Rust application service boundary. FF-P5 adds no network
  dependency, request, telemetry, or remote provider path.

## Frozen assertions

Production-path tests cover:

- case-insensitive process classification for representative email, messaging,
  document, code, terminal, and uncategorized applications;
- disabled-by-default and fail-closed settings migration behavior;
- exact denylist, remote-session, secure-field, unknown-security, and
  profile-disabled decisions;
- zero diagnostic text exposure for every rejected decision, even when a
  synthetic target contains private window and surrounding text;
- enabled diagnostics exposing only the captured character count;
- sentence-start and mid-sentence capitalization/spacing;
- literal preservation of mixed-case identifiers; and
- compact suppression of profile-requested trailing space.

## Verification commands

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run check:translations
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings -A clippy::pedantic
bun run tauri build --no-bundle
cargo audit -f src-tauri/Cargo.lock
scripts/check-foundation-provenance.sh
git diff --check
```

The full Rust suite passes 201 runnable tests with the two explicit 731 MB
live-model installation tests ignored. All 537 translation keys are present
across 23 catalogs. Frozen dependency installation, frontend/service-boundary,
formatting, TypeScript, production build, strict warnings-denied Clippy,
provenance, and diff gates pass. RustSec reports no denied vulnerability and 28
policy-allowed upstream warnings.

The optimized Windows executable is 44,581,888 bytes with SHA-256
`c588c93ba50e2ab64366486a30404eebc6d6c016a4cba379bc9f493a5ead34f1`.

Hosted Windows/macOS CI evidence is recorded after publication of the immutable
candidate SHA.

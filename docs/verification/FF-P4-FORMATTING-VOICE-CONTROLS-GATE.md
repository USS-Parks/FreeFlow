# FF-P4 deterministic formatting and voice-controls gate

Date: 2026-07-19
Scope: local deterministic formatting, correction controls, and guarded submit

## Delivered behavior

- The shared final-output pipeline recognizes explicit English spoken
  punctuation, paragraphs, line breaks, bullets, numbered items, backtrack,
  and opt-in number phrases without invoking an LLM.
- French and Spanish punctuation, line/paragraph, and backtrack commands are
  scoped to the effective dictation language. Unsupported languages retain
  literal text.
- Filler cleanup now uses the effective dictation language rather than the Hub
  display language. Newlines survive cleanup, and existing boundary formatting
  still preserves mixed-case names, numeric values, and code-like identifiers.
- Raw ASR text remains stored independently. Deterministic formatting and
  snippet expansion are recorded as final text, with voice controls parsed
  before snippets so saved expansion text cannot trigger a command.
- `press enter` is disabled by default. The first enablement requires a
  localized explicit confirmation, legacy enabled settings migrate to off, and
  the backend refuses enablement without persisted confirmation.
- Enter is requested only by the utterance-final phrase. Mid-sentence text is
  literal, `literal press enter` escapes the command, a command-only utterance
  reaches delivery without a false undo record, and secure, unknown, or changed
  targets fail closed before any insertion or submit key.

## Frozen assertions

Production-path tests cover:

- punctuation spacing, sentence capitalization, paragraphs, line breaks,
  bullets, and numbered lists;
- explicit number phrases and conservative preservation of ambiguous numbers,
  proper names, mixed-case identifiers, and snake-case identifiers;
- English, French, and Spanish command scoping plus unsupported-language
  literal behavior;
- filler and stutter policy, including language-specific false-positive words;
- backtrack removal at the current sentence boundary;
- disabled, confirmed, utterance-final, mid-sentence, command-only, and literal
  `press enter` paths;
- legacy settings migration and backend confirmation enforcement; and
- secure/unknown/changed target rejection and content-free command-only undo.

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

The full Rust suite passes 195 runnable tests with the two explicit 731 MB
live-model installation tests ignored. All 502 translation keys are present
across 23 catalogs. Frozen dependency installation, frontend/service-boundary,
formatting, TypeScript, production build, strict warnings-denied Clippy,
provenance, and diff gates pass. RustSec reports no denied vulnerability and 28
policy-allowed upstream warnings.

The optimized Windows executable is 44,486,656 bytes with SHA-256
`cba653ba97c876abede24e777212d21187ffa4e4631261ee324c2868f339654d`.

Hosted CI run
[`29698366361`](https://github.com/USS-Parks/FreeFlow/actions/runs/29698366361)
passed native Windows and macOS foundation jobs, provenance, and security at
exact candidate commit `f1d66200f1b345a8a82157bd78965d84044dac2c`.

The retained interactive Windows/macOS press-enter application matrix remains
mandatory at FF-R2 under the PSPR's 2026-07-19 consolidation. This evidence is
the deterministic/native implementation gate, not a public-release claim.

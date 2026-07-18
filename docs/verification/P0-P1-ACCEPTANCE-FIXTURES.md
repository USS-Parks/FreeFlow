# FreeFlow P0/P1 Acceptance Fixtures

Status: FF-G1 frozen specification
Applies to behavior-matrix P0 and P1 rows
Implementation evidence: Pending

## Fixture provenance

- Spoken scripts below are original FreeFlow test material, not Wispr outputs.
- Audio assets will be recorded by consenting project contributors or obtained from clearly licensed test corpora. Each asset manifest records speaker consent or license, language, microphone, environment, duration and SHA-256.
- Expected text describes FreeFlow behavior. It is not intended to reproduce a proprietary model verbatim.
- Live OS claims require interactive evidence. Mocks can test state machines but cannot close a platform gate.

## Standard hosts and application matrix

The final host manifest records the exact OS build, CPU, RAM, GPU, microphone, display scale and model hash.

- Windows: Windows 10 and 11 x64; Notepad, Chrome text input/contenteditable, Microsoft Word where available, VS Code editor, Windows Terminal/PowerShell, Cursor or equivalent IDE terminal and Microsoft Remote Desktop.
- macOS: oldest supported plus current macOS on Apple Silicon and Intel; TextEdit, Notes, Safari/Chrome input/contenteditable, Microsoft Word where available, VS Code editor, Terminal.app/iTerm2 and Microsoft Remote Desktop.
- Security variants: normal target, elevated Windows target, macOS Accessibility revoked, secure/password field and remote clipboard enabled/disabled.

## Core fixtures

| Fixture | Behavior | Procedure | Passing evidence |
|---|---|---|---|
| FF-FIX-001 | BF-001 shortcut lifecycle | Configure a safe single key, chord and mouse button where supported. Run 50 hold/release cycles, 20 toggle cycles, cancellation during recording/processing, focus changes and sleep/wake. | Exactly one state transition per activation; no stuck recording; Escape cancels; conflicts are rejected; key repeats do not create duplicate sessions. |
| FF-FIX-002 | BF-002 microphone and first-word integrity | Cycle built-in/external mics, hot-plug, deny/regrant permission, then let Zoom/Teams/browser acquire and release the mic. Speak "Marigold begins this sentence" immediately after activation in 50 cycles. | Feedback appears within 150 ms p95; `Marigold` is present in every valid capture; selected device is truthful; failures name blocked, missing, busy or disconnected state. |
| FF-FIX-003 | BF-003 local transcription | Install a verified model, block outbound networking, restart, dictate short/medium/noisy samples and switch language. | ASR completes locally; process makes no network attempt; raw text, engine/model/hash, latency and error evidence are recorded. |
| FF-FIX-004 | BF-004 cross-app insertion | Insert 100 independently authored results across the standard application matrix, including Unicode, multiline, leading/trailing whitespace and pre-existing clipboard text/file references. | At least 98% direct/fallback delivery; target receives current result, never stale result; clipboard is restored; failure remains copyable and recoverable. |
| FF-FIX-005 | BF-005 recovery | Force engine error, app crash after capture, paste refusal, cancellation and process restart. Retry from history and use copy/paste-last. | Captured audio survives until policy expiry; no duplicate/stale transcript; retry is idempotent; user can always copy the newest successful raw/final result. |
| FF-FIX-006 | BF-006 status bar | Exercise every state, drag bottom/left/right, cancel drag, restart, multi-monitor, work-area changes, 100-200% scaling and full-screen apps. | Original FreeFlow control never steals typing focus, remains within work area, persists its dock and exposes accessible state names. |
| FF-FIX-007 | BF-007 tray/menu bar | With Hub closed/open, use tray/menu commands for record, microphone, language, history, paste last, settings and quit. | Commands target the single running instance, reflect current state, are keyboard accessible and never start a hidden duplicate process. |
| FF-FIX-008 | BF-008 history and metrics | Complete, fail, retry, edit, search, copy and delete sessions across dates; verify WPM/word counts from known fixtures. | Raw/final/audio linkage is correct; metrics reconcile exactly; deletion/retention applies atomically; search excludes deleted content. |
| FF-FIX-009 | BF-009 dictionary | Add Unicode names, acronyms, singular/plural, case variants and misspelling replacements; test starring, duplicates, CSV import rollback and unsupported engine boosting. | Deterministic replacements respect boundaries/precedence; import is atomic; unsupported boosting is disclosed; changes apply next session. |
| FF-FIX-010 | BF-010 snippets | Define whole-phrase triggers including overlaps, punctuation, Unicode and a 4,000-character multiline expansion; import/export JSON. | Matching is case-insensitive but boundary safe; saved expansion casing/format is exact; conflicts and duplicates are deterministic; import is atomic. |
| FF-FIX-011 | BF-011 formatting/backtrack | Speak original scripts containing fillers, a self-correction, spoken punctuation, lists, numbers, paragraph commands, proper noun and code identifier. | Deterministic commands are exact; cleanup preserves names/numbers/code and intended correction; raw text remains available; uncertainty falls back conservatively. |
| FF-FIX-012 | BF-012 press-enter | Say the command alone, at utterance end, in the middle, with punctuation and before/after first-use consent. Repeat in normal and secure fields. | Enter is sent only at an enabled valid utterance end; first use requires confirmation; literal mid-sentence phrase remains; secure targets are not submitted without explicit action. |
| FF-FIX-013 | BF-013 languages | Run selected English, Czech-with-English-terms, French punctuation, Chinese/Japanese spacing and wrong-language recovery scripts on engines declaring support. | Selection/detection matches engine capability; code-switching and orthography meet frozen expected classes; unsupported combinations are disclosed. |
| FF-FIX-014 | BF-014 context | Test sentence-start/mid-sentence, email/message/code categories, disabled context, denied app, secure field and remote session. | Only minimum local context is read; category/spacing/case are correct; disabled/denied/secure/remote paths expose no surrounding text and use default formatting. |
| FF-FIX-015 | BF-015 styles/cleanup | Apply None/Light/Medium/High and each original FreeFlow app-category style to scripts with facts, names, numbers, negation, tone and formatting. | None preserves raw policy; higher levels preserve frozen facts; raw/final diff and undo work; timeout pastes raw text. |
| FF-FIX-016 | BF-016 transforms | Select text in each standard editor, run built-in/custom local transforms, cancel, timeout, retry, return unchanged output and exceed 1,000 words. | Correct selection is replaced only on success; diff is accurate; undo restores exact source; failure/cancel changes nothing and leaves output copyable. |
| FF-FIX-021 | BF-021 onboarding/permissions | Run clean install, no-network-before-model, low disk, corrupt model, mic denial, Accessibility denial/revocation, autostart, reset, upgrade and uninstall choices. | Every prerequisite is explained/recoverable; no account is requested; no implicit download; reset/uninstall honor selected data policy. |
| FF-FIX-022 | BF-022 privacy/local-only | Deny outbound traffic, inspect process connections/files/logs/clipboard, exercise all retention modes, delete all data and restart. | No undisclosed network attempt or sensitive log; never-store persists no audio/text; deletion removes indexed/backing data at app layer; model/update exceptions are explicit user actions. |
| FF-FIX-023 | BF-023 terminal/remote/security delivery | Dictate short and 100-line text into Windows/macOS terminals, WSL/VM, Claude Code/Codex TUI, RDP with clipboard on/off and elevated Windows target. | App-specific paste/chunking avoids collapsed or stale content; internal completion events, not clipboard timing, drive delivery; blocked boundaries show truthful copy/paste-last guidance; no blanket elevation is required. |

## Cross-cutting quality corpus

Every applicable fixture includes:

- quiet built-in microphone;
- common wired or USB headset;
- Bluetooth headset where the OS exposes a supported input profile;
- office/background speech and keyboard noise;
- short one-word and 10-second utterances;
- 2-minute and segmented 20-minute sessions where supported;
- names, acronyms, numbers, negation, URLs, paths, identifiers and multilingual terms;
- deliberate pause, filler, restart and self-correction; and
- raw ASR scoring separately from cleanup and final-delivery scoring.

## Evidence bundle format

Each run records fixture ID, commit SHA, platform manifest, model manifest/hash/license, input asset hash, expected result class, actual raw/final result, timing, memory/CPU, application target, clipboard before/after hash or type inventory, network capture reference, pass/fail and known variance.

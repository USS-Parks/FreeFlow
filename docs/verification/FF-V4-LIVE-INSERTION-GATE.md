# FF-V4 live cross-application insertion gate

Status: **In progress — implementation candidate verified headlessly; live Windows and macOS matrices pending**

This gate freezes the evidence required by FF-V4 before interactive results are
observed. Unit tests, mocks, compilation, and hosted non-interactive runners do
not substitute for this matrix.

## Candidate and invariants

- Starting commit: `9bf7583266eecbd8fa053c58db6d186bf78152f9`.
- Candidate commit: pending.
- Each platform must complete 100 normal insertion attempts and reach at least
  98 successful direct-or-clipboard deliveries.
- Every attempt records the expected and actual target, exact delivered text,
  insertion method, clipboard type inventory plus before/after SHA-256 where
  text is safely restorable, and manual-recovery availability.
- No attempt may deliver stale text or insert into a target different from the
  one captured when dictation began.
- Clipboard state must survive every direct and clipboard-fallback attempt.
  Rich, image, and file-reference clipboards that cannot be losslessly restored
  must bypass clipboard fallback and preserve their original formats.
- Secure fields and fields whose security state cannot be established must not
  expose surrounding text or receive automatic insertion.
- Successful insertions retain only target identity, inserted-text SHA-256,
  character count, and insertion timestamp as undo metadata. The metadata must
  not duplicate transcript content.

## Windows matrix — 100 normal attempts

Run ten independently authored payload classes against each target: short
ASCII, Unicode, multiline, trimmed leading/trailing whitespace, mid-sentence
case/spacing, sentence-start case, acronym case, 100 lines, pre-existing rich
clipboard content, and pre-existing file-reference clipboard content.

1. Notepad
2. Chrome text input
3. Chrome contenteditable surface
4. Microsoft Word
5. VS Code editor
6. Windows Terminal PowerShell prompt
7. Cursor or equivalent IDE terminal
8. Remote Desktop text editor with clipboard enabled
9. Remote Desktop terminal with clipboard disabled
10. Elevated text target

Retain the Windows edition/build, CPU/RAM, display scale, FreeFlow executable
SHA-256, application versions, remote endpoint manifest, per-attempt JSON, and
screen recording or equivalent target-visible proof.

## macOS matrix — 100 normal attempts

Use the same ten payload classes against each target:

1. TextEdit
2. Notes
3. Safari or Chrome text input
4. Safari or Chrome contenteditable surface
5. Microsoft Word
6. VS Code editor
7. Terminal.app
8. iTerm2 or equivalent IDE terminal
9. Microsoft Remote Desktop text editor with clipboard enabled
10. Microsoft Remote Desktop terminal with clipboard disabled

Retain the macOS version/build, architecture, CPU/RAM, display scale, FreeFlow
bundle SHA-256, application versions, remote endpoint manifest, per-attempt
JSON, and target-visible proof. Repeat the matrix on Apple Silicon and Intel if
both remain release architectures; both must at minimum compile before this
prompt can close.

## Security and recovery matrix

These checks are additional to the 100 normal attempts on each platform and do
not count as failed delivery when the expected result is a safe refusal.

1. Start dictation in a normal field, move focus to another field, and verify
   automatic insertion is refused while manual Copy remains available.
2. Repeat with a password field focused at capture and at insertion. Verify no
   surrounding text is retained or emitted and no automatic submit occurs.
3. Revoke macOS Accessibility permission and verify security-unknown refusal,
   truthful recovery guidance, and unchanged clipboard state.
4. Use an elevated Windows target from a non-elevated FreeFlow process and
   verify either correct delivery or a truthful manual fallback without blanket
   elevation.
5. Force direct-insertion failure with plain-text clipboard content and verify
   clipboard paste succeeds and restores the exact original text.
6. Repeat the forced failure with rich, image, and file-reference clipboards;
   verify clipboard fallback is refused, the formats remain intact, and manual
   Copy is available.
7. Use Paste Last Transcript after changing targets and verify it captures the
   new target, applies the same security policy, and never pastes a stale entry.
8. Confirm success events and managed undo state contain only the approved
   hash/length/target/time metadata.

## Completion rule

FF-V4 passes only when retained Windows and macOS evidence satisfies every
invariant above and each platform reaches at least 98/100 normal deliveries.
Until then, FF-V4 remains in progress and FF-V5 must not begin.

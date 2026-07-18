# Wispr Flow Observable-Behavior Source Ledger

Status: FF-G1 frozen research baseline
Retrieval cutoff: 2026-07-17
Scope: Publicly documented or publicly reported Windows/macOS behavior only

## Evidence policy

This ledger identifies user needs and failure hypotheses. It is not evidence about Wispr source code, internal algorithms, prompts, infrastructure, or non-public behavior.

Evidence is weighted as follows:

1. **Official documentation:** authoritative for publicly claimed controls and workflows, but not proof that every implementation works reliably.
2. **First-party case studies:** useful for discovering real workflows and value propositions, but selected and edited for marketing.
3. **Review platforms:** useful for recurring themes across reviewers, but subject to selection, verification, incentive, and aggregation bias.
4. **Open forums:** useful for reproducible failure hypotheses and edge cases, but individual reports are unverified and may reflect obsolete versions or local configuration.

No user-generated transcript or proprietary model output is copied into FreeFlow fixtures. All spoken scripts and expected results are independently authored.

## Official workflow sources

| Source | Type | Observable signal | Requirement impact |
|---|---|---|---|
| [Desktop navigation](https://docs.wisprflow.ai/articles/5096240724-navigating-the-wispr-flow-app-desktop-ios-and-android) | Official documentation | Hub, history, dictionary, snippets, styles, scratchpad, floating bar, tray controls and settings | BF-002, BF-006 through BF-010, BF-015, BF-018, BF-021 |
| [Smart Formatting and Backtrack](https://docs.wisprflow.ai/articles/5373093536-how-do-i-use-smart-formatting-and-backtrack) | Official documentation | Contextual spacing/case, lists, punctuation, filler/self-correction handling and press-enter | BF-011 and BF-012 |
| [Multiple languages](https://docs.wisprflow.ai/articles/3191899797-use-flow-with-multiple-languages) | Official documentation | Per-session language detection, explicit selection, regional orthography and code-switching limits | BF-013 |
| [Transforms](https://docs.wisprflow.ai/articles/8068950331-how-to-use-transforms-beta) | Official documentation | Selected-text rewrite, configurable prompts, diff, undo and fallback | BF-016 |
| [Command Mode](https://docs.wisprflow.ai/articles/4816967992-how-to-use-command-mode) | Official documentation | Spoken transformation over selected text or at cursor; confirmation before preferences change | BF-017 |
| [IDE integrations](https://docs.wisprflow.ai/articles/6434410694-use-flow-with-cursor-vs-code-and-other-ides) | Official documentation | IDE context, identifiers, file tags, Windows terminal paste strategy and long prompt chunking | BF-004, BF-019 and BF-023 |
| [Terminal and WSL support](https://docs.wisprflow.ai/articles/6478598909-using-flow-with-linux-wsl-and-terminal-applications) | Official documentation | Direct paste varies by terminal; WSL/VM fallback; privilege-level restrictions | BF-004 and BF-023 |
| [Remote desktop support](https://docs.wisprflow.ai/articles/7336156466-use-flow-with-remote-desktops-citrix-rdp-vdi) | Official documentation | Local mic/clipboard, clipboard-redirection dependency, manual fallback and loss of context awareness | BF-004 and BF-023 |
| [Accessibility support](https://docs.wisprflow.ai/articles/3941699399-keyboard-and-screen-reader-accessibility-in-wispr-flow) | Official documentation | Keyboard navigation, screen readers, focus visibility and reduced motion | BF-005 through BF-008 and BF-021 |
| [Data controls](https://wisprflow.ai/data-controls) | Official policy | Reference transcription is cloud-based; app/context data may influence formatting | BF-014 and BF-022; reinforces FreeFlow local-only behavior |

## Case-study and review signals

| Source | Bias note | Workflow or need | Derived FreeFlow hypothesis |
|---|---|---|---|
| [Dean Payn case study](https://wisprflow.ai/case-study/dean-payn) | First-party marketing | Voice plus snippets used for email, replies, notes, recurring explanations, AI prompts and call-recap context | Snippets must work inline and across app categories; app switching must not reset dictation workflow |
| [Gaurav Vohra case study](https://wisprflow.ai/case-study/gaurav-vohra) | First-party marketing | Natural speech and automatic punctuation reduce context-switching across Messages, email and Slack | Activation and delivery must be fast enough to become a default input loop; style must follow target context without losing intent |
| [Accessibility case accounts](https://wisprflow.ai/accessibility) | First-party marketing | Users with pain, fatigue, vision or mobility constraints depend on hands-free cross-app input | Recovery and accessibility are core correctness; no workflow may require precise pointer use |
| [Product Hunt reviews](https://www.producthunt.com/products/wisprflow) | Mixed reviews; page includes an AI summary | Praise clusters around speed, accuracy, cross-app use, cleanup, coding and mixed languages; complaints mention privacy, setup, resource use and Windows reliability | Freeze latency, offline, resource, Windows insertion, code-switching and setup gates |
| [G2 reviews](https://www.g2.com/products/wispr-flow/reviews) | Small reviewed sample and platform aggregation | Users value punctuation/grammar cleanup and time saved but still report occasional errors | Preserve raw transcripts and make cleanup strength reversible rather than assuming a polished result is correct |
| [Apple App Store reviews](https://apps.apple.com/us/app/wispr-flow-ai-voice-keyboard/id6497229487?see-all=reviews) | Primarily mobile; only general lessons transfer | Context, lists and paragraphs are valued; lost long-form dictations and noise robustness are trust breakers | Desktop recovery and noisy/headset corpus cases remain mandatory; mobile UI complaints stay out of scope |

## Open-forum failure hypotheses

These are test inputs, not accepted facts about current Wispr releases.

| Source | User-reported failure | FreeFlow acceptance response |
|---|---|---|
| [Reliability and accuracy discussion](https://www.reddit.com/r/WisprFlow/comments/1tx06rk/reliability_and_accuracy_update_what_happened/) | Cleanup regressions, microphone-session latency creep and swallowed first words after other apps use the mic | Separate ASR from cleanup scoring; add 50-cycle mic-contention and first-word-integrity fixtures; never hide raw text |
| [Windows clipboard bug](https://www.reddit.com/r/WisprFlow/comments/1sp2qb8/wisprflow_clipboard_bug_on_windows_desktop/) | Stale clipboard contents inserted instead of the new transcript | Transactional clipboard ownership, unique insertion state, restoration tests and paste-last recovery |
| [Windows auto-paste failures](https://www.reddit.com/r/WisprFlow/comments/1s2trmg/clipboard_function_not_working_anymore/) | Transcription succeeds but nothing reaches even simple targets | Treat transcription and delivery as separate states; show a copyable result and target-specific diagnostic |
| [Cursor terminal/TUI workaround](https://www.reddit.com/r/WisprFlow/comments/1sf3ysz/fix_for_wispr_flow_not_pasting_into_cursor/) | Clipboard watchers can create false-positive pastes; terminal shortcuts differ | Do not watch global clipboard timing to infer completion; emit an internal completion event and use an app-specific paste strategy |
| [Windows elevated-window report](https://www.reddit.com/r/WisprFlow/comments/1sh2c9d/wispr_flow_cant_paste_into_adminelevated_windows/) | Non-elevated app cannot inject into elevated targets | Detect integrity mismatch and provide safe copy/manual-paste guidance; never silently fail or request blanket elevation by default |
| [Reliability/service disruption discussion](https://www.reddit.com/r/WisprFlow/comments/1turxe0/what_is_going_on_with_wisprflow_recently/) | Service availability and stale paste failures make the tool slower than typing | Local ASR removes service dependency; stress tests must detect stale-result races and preserve latest audio |
| [Single-key shortcut discussion](https://www.reddit.com/r/WisprFlow/comments/1u03r04/shortcut_key/) | Shortcut friction can break established muscle memory | Support a safe single key where the OS permits it, chords, mouse buttons, conflict detection and accidental-activation controls |
| [Remote desktop report](https://www.reddit.com/r/WisprFlow/comments/1s424fg/wisprflow_does_not_work_with_an_rdp/) | Local-to-remote clipboard path may fail despite user expectations | Add RDP/VDI matrix with clipboard enabled/disabled and copy-last fallback; never claim context across an inaccessible remote boundary |
| [Clipboard history concern](https://www.reddit.com/r/WisprFlow/comments/1rcdjw7/prevent_dictation_from_copying_to_clipboard/) | Dictated text can pollute clipboard history and expose sensitive text | Prefer direct insertion; mark transient clipboard data non-persistent where supported; always restore and offer clipboard-free mode |

## Frozen cross-source product principles

1. **Reliability outranks cleverness.** A raw transcript that is recoverable is better than a polished result that is lost or silently wrong.
2. **Activation must feel immediate.** Feedback and first-word capture are separate gates; an animation alone does not prove audio integrity.
3. **Transcription and delivery are separate state machines.** Each has its own success, retry, cancellation and diagnostic evidence.
4. **The clipboard is a fallback transport, not implicit durable storage.** Preserve it transactionally and avoid polluting history where the OS allows.
5. **Cleanup is reversible.** Store and expose raw and final text, measure both, and fall back to raw on timeout or uncertainty.
6. **Local means independently usable offline.** No authentication, subscription, telemetry or remote inference may be required after model installation.
7. **Accessibility is a release gate.** Keyboard, screen-reader, reduced-motion and hands-free recovery paths are required on both platforms.
8. **App context is minimum-necessary and honest.** Never claim surrounding-text or remote context when the platform adapter cannot obtain it.
9. **Resource cost is visible.** Model and application footprints are measured and disclosed instead of hidden behind one recommendation.

## Monitoring rule

This ledger is frozen for FF-G1. Later changes do not silently rewrite it. A monthly or pre-milestone refresh may append sources and reclassify hypotheses, with retrieval dates and plan impact recorded in PSPR history.

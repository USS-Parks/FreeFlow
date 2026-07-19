# FF-V5 status bar and tray gate

Status: **Implementation slice complete — deterministic and hosted native gates pass; interactive matrix retained at FF-R2**

Candidate commit: `b02751dc4e0edb06ec0827dfc3fbcdb24e2d7d6d`

## Frozen behavior

- The original FreeFlow status bar represents recording, streaming,
  transcribing, processing, success, warning, and error without taking typing
  focus.
- The bar remains inside the selected monitor work area at 100–200% scaling and
  can persist top, bottom, left, or right placement. A completed drag snaps to
  the nearest supported left, right, or bottom dock and persists it atomically.
- A stale transient completion timer cannot hide a newer recording state.
- The status region has an accessible live-state name; its cancel control has a
  localized accessible name.
- The native tray/menu exposes the single running instance, start/stop/cancel,
  open Hub, microphone, language, history, paste last, copy last, model,
  settings, and quit surfaces, with a state-bearing tooltip.
- Hub navigation uses native buttons with visible keyboard focus.

## Automated evidence

- Rust work-area tests prove every dock remains inside the work area and drag
  classification selects left, right, or bottom deterministically.
- The production pipeline uses non-activating/topmost native windows on Windows
  and a non-activating full-screen auxiliary panel on macOS.
- The same transcription coordinator serializes tray and global-shortcut
  starts/stops, preventing a hidden duplicate recording path.
- All 23 locale catalogs contain the new status, dock, and tray keys.

## Retained FF-R2 matrix

Exercise multi-monitor work areas, 100–200% scaling, full-screen applications,
focus preservation, drag and drag-cancel, restart persistence, keyboard tray
operation, and screen-reader announcements on Windows and macOS. This retained
interactive evidence is required before release under the PSPR's 2026-07-19
consolidation.

## Hosted native evidence

GitHub Actions run
[`29680441253`](https://github.com/USS-Parks/FreeFlow/actions/runs/29680441253)
passed at the exact candidate commit. Native Windows and macOS foundation jobs,
provenance, and RustSec all completed successfully.

# Upstream dependency ledger

Status: FF-G3 foundation import
Reviewed: 2026-07-18

The Handy foundation imports the following Git dependencies. They are retained
at the exact revisions in `src-tauri/Cargo.lock` until released crates can
replace them without changing verified behavior.

| Package               | Repository                                      | Exact revision                             | Declared/shipped license                                              | FF-G3 disposition                                                                               |
| --------------------- | ----------------------------------------------- | ------------------------------------------ | --------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `hf-hub 0.5.0`        | `cjpais/hf-hub`, branch `cancellable-downloads` | `9918f11ab3473135eb7865aa4a5d79d597e6e810` | Apache-2.0                                                            | Retain temporarily for cancellable local-model plumbing; network model installation is disabled |
| `rdev 0.5.0-2`        | `rustdesk-org/rdev`                             | `a90dbe1172f8832f54c97c62e823c5a34af5fdfe` | MIT                                                                   | Retain for cross-platform keyboard events                                                       |
| `rodio 0.20.1`        | `cjpais/rodio`                                  | `fed30292db417cb95305c118c0e1d804fb74cbff` | MIT OR Apache-2.0                                                     | Retain temporarily; replace with a released upstream crate after audio regression tests exist   |
| `tao 0.35.3`          | `tauri-apps/tao`                                | `07f3742b1833b64be27b1ef991e38d557d4276c9` | Apache-2.0                                                            | Retain matched UI fix until the required change is released                                     |
| `tao-macros 0.1.3`    | `tauri-apps/tao`                                | `07f3742b1833b64be27b1ef991e38d557d4276c9` | MIT OR Apache-2.0                                                     | Must match the `tao` revision                                                                   |
| `tauri-nspanel 2.1.0` | `ahkohd/tauri-nspanel`, branch `v2.1`           | `da9c9a8d4eb7f0524a2508988df1a7d9585b4904` | LICENSE_MIT and LICENSE_APACHE-2.0 shipped; manifest metadata missing | Temporary macOS overlay dependency; explicit metadata exception                                 |
| `vad-rs 0.1.6`        | `cjpais/vad-rs`                                 | `2a412ed858695b9251f3f5a1a20d95b59fa7c498` | MIT                                                                   | Retain engine seam, but no VAD weight is distributed and VAD defaults off                       |

## RustSec disposition

FF-G3 updates `rustls-webpki` to `0.103.13`, `tar` to `0.4.45`, `plist` to
`1.10.0`, and the active Windows/macOS `quick-xml` path to `0.41.0`.

`quick-xml 0.38.4` remains locked only through `wayland-scanner 0.31.8`, a
Linux-target build dependency. Linux packaging is explicitly parked and is not
part of the Windows/macOS FreeFlow release scope. RustSec advisories
`RUSTSEC-2026-0194` and `RUSTSEC-2026-0195` are therefore temporarily ignored
with this target-specific record. They must be removed before Linux builds or
packaging are enabled, and the ignore is reviewed monthly with upstream.

# FF-A1 local transform runtime gate

Date: 2026-07-19
Scope: optional local transform runtime, model selection, bounded execution, and raw-text fallback

## Delivered behavior

- FreeFlow exposes one local-only post-processing provider. Legacy remote and
  custom providers, stored endpoint selection, and API keys migrate to the
  verified local provider.
- Nothing downloads until the user reviews one combined consent record covering
  both artifacts' immutable revisions, exact sizes, SHA-256 hashes, licenses,
  source URLs, and managed destinations.
- The selected runtime is llama.cpp `b10068` at source revision
  `571d0d540df04f25298d0e159e520d9fc62ed121`. Platform archives are pinned for
  Windows x64 Vulkan, macOS arm64, and macOS x64.
- The selected model is `SmolLM2-135M-Instruct-Q4_K_M.gguf`, artifact revision
  `09816acd5d99df7be770d85ea30822623dab342c`, base revision
  `7e27bd9f95328f0f3b08261d1252705110c806f8`, 105,454,432 bytes, SHA-256
  `2e8040ceae7815abe0dcb3540b9995eaa1fa0d2ca9e797d0a635ae4433c68c2d`.
- Installation is resumable and cancellable. Exact identity is reverified
  before every use, corrupt artifacts are rejected, and deletion removes the
  managed runtime/model directory.
- Each transform starts a hidden, single-slot server bound only to
  `127.0.0.1`, disables its web UI and logs, strips proxy/model-download
  environment variables, sends only separate bounded system/user messages, and
  kills the child on completion, timeout, cancellation, or future drop.
- System instructions are limited to 4,000 characters, input to 12,000
  characters, output to 512 tokens, and the user timeout to 5-120 seconds.
  Missing install, memory-pressure policy, cancellation, corruption, empty
  output, timeout, or process failure returns no replacement, so the raw
  transcript remains authoritative.
- Resource recommendations report logical CPUs, total memory, a conservative
  512 MB peak budget, and available CPU/Vulkan or CPU/Metal paths. Core
  transcription never depends on this optional runtime.

## Windows live evidence

The headless application path used the same install and transform services as
the desktop UI without opening a window or microphone.

- Combined accepted manifest digest:
  `8d9eb4e5d7358b68d9334b250eb010887a238014c9a499bc7da199808ef82947`.
- Explicit install transferred 138,726,136 bytes and returned
  `runtime_verified: true`, `model_verified: true`, and `installed: true`.
- CPU (`--gpu-layers 0`) transformed `hello world period` to
  `Hello world period` in 6,322 ms including runtime/model startup.
- Vulkan (`--gpu-layers 99`) produced the same result in 6,880 ms including
  startup.
- A one-second forced timeout exited nonzero with
  `Local transform exceeded the 1-second timeout`.
- Replacing the verified model with a same-destination corrupt fixture exited
  nonzero before process launch. Restoring the artifact reproduced its exact
  105,454,432-byte length and approved SHA-256.

## Frozen assertions

Automated tests cover immutable complete manifests and combined consent,
restricted HTTPS redirects, bounded prompt/input/output construction,
resource rejection, active-transfer cancellation, exact artifact verification,
resumable cancellation, corrupt model handling, and preservation of raw text
for cancellation, corruption, timeout, and memory-pressure failures.

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

The dedicated `FF-A1 local transform live gate` workflow independently
downloaded and verified the same pinned artifacts at exact candidate
`ba0bc09b0cc3d19d1d4bbc7b3c0157808fc9a9fb`. Hosted run `29703174180` passed
CPU/Vulkan on Windows and CPU/Metal on Apple Silicon. Windows CPU completed in
1,280 ms and Vulkan in 605 ms. Apple Silicon CPU completed in 21,168 ms and
Metal in 214,180 ms. Every path returned nonempty output. The hosted Metal
number includes cold runtime/shader startup; FreeFlow's bounded 30-second
default preserves raw text when a cold transform exceeds that limit.

Normal hosted run `29703174188` passed Windows and macOS foundation jobs,
provenance, and RustSec at the same exact candidate commit.

The local candidate gates pass 210 runnable Rust tests with the two explicit
731 MB Parakeet live-install tests ignored. All 552 translation keys are present
across 23 catalogs. Frozen dependency installation, frontend/service-boundary,
formatting, TypeScript, production build, strict warnings-denied Clippy,
provenance, and diff gates pass. RustSec reports no denied vulnerability and 28
policy-allowed upstream warnings.

The optimized Windows executable is 44,963,328 bytes with SHA-256
`3ad039dcc205abf2a78afdc64a431912cd97507a5d0ba207ba493a41c4bb65a1`.

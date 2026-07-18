# FreeFlow

FreeFlow is a local-first, open-source desktop dictation application for
Windows and macOS. Hold a shortcut, speak naturally, and insert the transcript
into the active application without sending audio to a cloud service.

Canonical public repository: [USS-Parks/FreeFlow](https://github.com/USS-Parks/FreeFlow)

FreeFlow is currently under active clean-room development. The imported native
dictation foundation comes from Handy under the MIT License; provenance and
retained notices are recorded in `NOTICE.md` and the architecture decision
record.

## Current development setup

- Rust (latest stable)
- Bun
- Tauri 2 platform prerequisites
- Vulkan SDK on Windows for the accelerated transcription backend

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
bun run tauri build --no-bundle
```

Model weights are not distributed with the repository. Locally downloaded
evaluation models belong in `models/` and are ignored by Git.

## Canonical documents

- [FreeFlow Clean-Room PSPR](PLANNING/FREEFLOW-CLEAN-ROOM-PSPR.md)
- [Upstream foundation decision](docs/architecture/ADR-0001-UPSTREAM-FOUNDATION.md)
- [Behavior parity matrix](docs/product/BEHAVIOR-PARITY-MATRIX.md)
- [Observable-behavior source ledger](docs/research/WISPR-OBSERVABLE-BEHAVIOR-SOURCE-LEDGER.md)
- [P0/P1 acceptance fixtures](docs/verification/P0-P1-ACCEPTANCE-FIXTURES.md)
- [Clean-room policy](docs/legal/CLEAN-ROOM-POLICY.md)
- [Development log](docs/DEVLOG.md)

## Non-affiliation

FreeFlow is an independent project. It is not affiliated with, endorsed by, or
derived from Wispr AI, Inc. "Wispr Flow" is used in planning documents only to
identify a publicly documented reference product.

FreeFlow is also independent from Handy and its authors. Handy is the
open-source foundation of this fork; its authors do not endorse FreeFlow.

## License

The imported Handy code is available under the MIT License. See `LICENSE` and
`NOTICE.md` for retained attribution and provenance.

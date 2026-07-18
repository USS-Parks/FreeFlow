# FreeFlow repository guidance

## Governance

`PLANNING/FREEFLOW-CLEAN-ROOM-PSPR.md` is the canonical execution roster.
Follow its prompts in dependency order and do not mark a prompt complete until
its stated gate passes. Preserve the Handy MIT notice and record imported or
extracted source in `NOTICE.md` and the development log.

Do not copy proprietary Wispr Flow code, assets, prompts, outputs, network
traffic, or non-public observations. Public behavior claims belong in the
source ledger and independently authored acceptance fixtures.

## Development gates

```text
bun install --frozen-lockfile
bun run lint
bun run format:check
bun run build
cargo test --manifest-path src-tauri/Cargo.toml
bun run tauri build --no-bundle
```

Windows native builds require the Vulkan SDK. Model weights are not committed.
Network model installation remains disabled until FF-V1 supplies immutable
source revisions, sizes, hashes, licenses, and explicit confirmation.

## Code style

- Keep Rust managers and Tauri commands as the application service boundary.
- Keep user-facing React strings in i18next translation files.
- Prefer explicit errors and cancellation over panics in production paths.
- Run the provenance gate before committing foundation changes.
- Use conventional commit prefixes and record prompt ID, files, gates, and SHA
  in `docs/DEVLOG.md`.

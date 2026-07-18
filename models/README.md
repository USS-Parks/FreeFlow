# Local model cache

This directory stores user-downloaded model weights for local development and
evaluation. Model binaries are intentionally ignored by Git and are not
redistributed with FreeFlow.

## Parakeet evaluation pin

- Artifact: `parakeet-unified-en-0.6b-Q8_0.gguf`
- Install manifest: `manifests/parakeet-unified-en-0.6b-q8_0.json`
- Artifact mirror: `memoravox/parakeet-unified-en-0.6b-gguf`
- Artifact revision: `17cf15519695fed7891fe1e81bfc512f3a58cc7b`
- Expected size: `731357568` bytes
- Expected SHA-256: `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`
- Base model: `nvidia/parakeet-unified-en-0.6b` at
  `d4ac9928f3bf238223ff0779c06b8149bf8ac4e1`
- Governing base-model terms: NVIDIA Open Model License Agreement
- Converted-repository declaration: `CC-BY-4.0`
- Redistribution status: FreeFlow does not bundle or redistribute this weight.
  An explicit user-approved install fetches the pinned public artifact directly,
  or verifies a user-selected local file against the same size and SHA-256.

The application presents both license scopes, their URLs, required attribution,
the exact source revision, size, hash, and destination before installation. The
checked-in manifest is the machine-readable source of truth.

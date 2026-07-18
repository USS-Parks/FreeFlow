# ADR-0003: Consent-bound local model supply chain

Status: **Accepted on Windows; FF-V1 remains in progress pending macOS live evidence**

Date: 2026-07-18

## Decision

FreeFlow permits model installation only through checked-in, immutable JSON
manifests. A manifest binds a stable model id to the exact artifact repository
and revision, base repository and revision, filename, byte count, SHA-256,
format, quantization, license disclosures, attribution, and redistribution
status. Runtime callers cannot provide or override a network URL.

Before either a direct transfer or manual import, the frontend obtains the
backend-generated install plan and shows its source pins, exact size, hash,
destination, both license scopes, attribution, and distribution status. The
user's acceptance returns the manifest digest; a missing or stale digest is
rejected by the backend.

Downloads use an exact-size partial file with HTTP Range resume. The HTTP client
allows only HTTPS delivery on approved Hugging Face host suffixes and rejects
cross-host redirects outside that set. A server that ignores Range causes the
partial to be deleted and disk space to be checked again before a fresh request.
Cancellation preserves a bounded partial for
resume. Offline errors and short responses cannot create an installed model;
oversized or hash-mismatched artifacts are deleted. Manual imports copy into the
same partial path and pass the same exact-size and hash verification.

Successful installation atomically moves the verified artifact and writes a
manifest-bound receipt. Discovery re-hashes the installed bytes, so receipt
tampering or same-size model tampering invalidates the installation. Deletion
removes both weight and receipt.

## First approved manifest

- Model id: `parakeet-unified-en-0.6b-q8_0`
- Artifact: `memoravox/parakeet-unified-en-0.6b-gguf` at
  `17cf15519695fed7891fe1e81bfc512f3a58cc7b`
- Base: `nvidia/parakeet-unified-en-0.6b` at
  `d4ac9928f3bf238223ff0779c06b8149bf8ac4e1`
- Size: `731357568` bytes
- SHA-256: `4b50b6dd862bf6e346929aaf4f5eaacec003bfa3f56462d6c874b41ef2f38795`
- Base-model terms: NVIDIA Open Model License Agreement
- Conversion-repository declaration: `CC-BY-4.0`
- Distribution: direct user-approved transfer only; not bundled or
  redistributed by FreeFlow

The two public license declarations are disclosed separately rather than being
silently collapsed into one claim.

## Verification status

Windows evidence passes: corrupt, hash mismatch, low disk, offline, cancel,
resume, receipt tampering, same-size weight tampering, and manual-install restart
tests pass. The pinned artifact was manually installed through the production
installer logic, re-hashed, receipted, loaded on the Vulkan backend, and used to
create a transcribe.cpp session. A separate live test exercised the production
restricted HTTP client end to end: direct download, manifest-digest consent,
exact-size and SHA verification, receipt discovery, Vulkan load, and session
creation all passed.

The workflow `.github/workflows/macos-model-live-gate.yml` requires separate
acknowledgements of the base-model and conversion-repository license
declarations, then executes that same ignored live test on the `macos-15` Apple
Silicon and `macos-15-intel` x86-64 hosted runners. Manual dispatch collects
both acknowledgements as Boolean inputs. Before the workflow reaches the
default branch, a push scoped to `codex/ff-v1-model-install` can run it only when
the repository variables `FREEFLOW_ACCEPT_NVIDIA_OPEN_MODEL_LICENSE` and
`FREEFLOW_ACCEPT_CONVERSION_CC_BY_4_0` are both exactly `true`. Path filters
avoid repeating the large transfer for unrelated changes. The workflow has not
yet supplied live macOS evidence.

FF-V1 is not complete because equivalent macOS installation and live model-load
evidence is not available in this Windows session. The next PSPR prompt must not
start until that gate is satisfied or the user approves a documented re-scope.

# ADR-0002: Versioned service contracts and durable local storage

Status: Accepted for FF-G4
Date: 2026-07-18

## Context

The imported foundation contains capable concrete managers, but product orchestration must not become permanently coupled to one ASR engine, one audio implementation, or one operating-system insertion path. Settings were stored through a UI-capable plugin without an explicit crash-safe write contract, history migrations had no rollback definitions, and a few React components accessed filesystem or clipboard APIs directly.

FF-G4 requires explicit, testable seams before the vertical slice adds model installation and live dictation behavior.

## Decision

### Contract versioning

`src-tauri/src/contracts/mod.rs` defines the v1 interfaces for:

- audio capture;
- ASR;
- post-processing;
- text insertion;
- platform context; and
- deterministic text transforms.

Every adapter exposes an identifier and `{major, minor}` contract version. A major-version mismatch is incompatible. An adapter with the same major version and an equal or newer minor version satisfies a requested contract.

Every asynchronous adapter call receives `OperationControl`. Orchestration wraps adapter futures with `enforce_operation`, which enforces cancellation and deadlines even when an adapter has not reached its own checkpoint. Adapters must also call `checkpoint` around expensive or externally blocking stages.

Concrete manager adapters will be attached to these seams as their owning PSPR prompts are implemented. The contract layer deliberately contains no model catalog, provider URL, operating-system singleton, or Tauri handle.

### SQLite schema ownership

`src-tauri/src/storage/migrations.rs` owns a contiguous, reversible history schema. A migration has both `up_sql` and `down_sql`; the runner applies the complete requested change in one SQLite `IMMEDIATE` transaction and advances `PRAGMA user_version` inside that transaction. Any failure rolls back both schema and version.

History startup migrates to the latest supported version. A database newer than the application is rejected rather than modified.

### Typed atomic settings

`AppSettings` remains the typed settings schema and carries `settings_schema_version`. `AtomicSettingsFile` replaces the webview-capable store plugin and writes the compatible `{ "settings": ... }` envelope through a same-directory temporary file, file synchronization, and atomic persistence.

Invalid JSON is never overwritten silently: the original is renamed with a `corrupt` suffix, defaults are written atomically, and the preserved path is logged without exposing setting values. Individually invalid fields in otherwise valid JSON continue through the existing field-level salvage and schema-migration path.

### UI service boundary

React may call generated Rust commands and subscribe to declared application events. It may not directly read/write application files, databases, settings, clipboard state, or network services. Audio-file reads and clipboard writes now have Rust commands. Theme `localStorage` is the sole narrow exception: it is a non-authoritative synchronous paint cache, while `AppSettings` remains the source of truth.

`scripts/check-service-boundary.ts` enforces the boundary during `bun run lint`.

## Verification obligations

- Cancellation and timeout contract tests must fail closed.
- Migration tests must prove forward migration, explicit rollback, transactional failure rollback, and reopen persistence.
- Settings tests must prove valid atomic replacement, corrupt-file preservation/recovery, and reopen persistence.
- The frontend boundary scan must report no prohibited direct APIs.
- Existing application tests and clean developer builds must remain green.

## Consequences

- New engines and platform implementations have a stable injection seam.
- Settings writes no longer require a webview permission or store plugin.
- History schema changes require a reviewed rollback statement.
- Feature UI cannot quietly grow an alternate persistence or network architecture.
- Migrating all existing concrete managers behind these interfaces remains owned by the vertical-slice prompts that change those behaviors; FF-G4 establishes and tests the boundary rather than pretending imported managers already implement it.

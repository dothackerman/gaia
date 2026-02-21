# ADR 003 — Plan migration to `wasm32v1-none` runtime target

## Context

During `cargo check`, the build emits warnings from the runtime WASM build step indicating that Rust >= 1.84 supports the newer `wasm32v1-none` target and recommends migrating away from `wasm32-unknown-unknown`.

Today the runtime still builds using `wasm32-unknown-unknown` and the workspace toolchain is pinned to `stable` (see `rust-toolchain.toml`). Migrating targets affects the runtime build pipeline and developer setup.

## Decision

Keep `wasm32-unknown-unknown` as the active runtime target for now, and schedule migration work to `wasm32v1-none` once the toolchain and CI environments are confirmed to support it consistently.

## Consequences

**Positive**

- Avoids unplanned build/CI breakage during initial template import.
- Keeps local development aligned with the template’s current behavior.

**Negative / accepted trade-offs**

- Ongoing warnings during builds until the migration is completed.
- Delays adoption of the newer target and any related improvements.

## Timeline

- **Next minor release**: evaluate and implement migration to `wasm32v1-none` (update toolchain/CI, update build scripts if needed, and document the change).

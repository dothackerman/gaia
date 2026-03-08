# ADR 014 — Runtime upgrade via Constitutional-class governance proposal

## Context

Waves 2–3 introduced typed governance actions, governance-origin dispatch, and
class-based thresholds/time-locks. The missing capability was runtime logic
replacement through the same member-governed proposal flow.

## Decision

Wave 4 adds runtime-upgrade governance through `GovernanceAction::UpgradeRuntime`.

- New action variant:
  - `UpgradeRuntime { code_hash: [u8; 32] }`
- New upload flow:
  - active members call `upload_runtime_code(code)`
  - blob is stored in `PendingRuntimeCode`
  - pallet emits `RuntimeCodeUploaded { uploader, code_hash }`
- Execution flow in `execute_proposal`:
  - requires Constitutional class (submission mapping)
  - loads pending blob and verifies hash matches proposal payload
  - dispatches `frame_system::set_code(Root, code)`
  - clears `PendingRuntimeCode`
  - emits `RuntimeUpgradeExecuted { code_hash }`

Safety checks:

- `NoPendingRuntimeCode`
- `RuntimeCodeHashMismatch`
- `RuntimeCodeTooLarge`

## Consequences

**Positive**

- Runtime logic is now governable on-chain without developer root intervention.
- Proposal hash binds approval to a specific reviewed WASM blob.
- Single pending blob plus hash verification prevents executing unexpected code.

**Negative / trade-offs**

- Single-slot pending blob model is last-write-wins.
- A malicious overwrite can force execution failure, but cannot force execution
  of an unapproved blob because hash mismatch blocks it.

## Follow-up

- Improve operational UX for metadata refresh and CLI runtime-upgrade commands.
- Evaluate multi-blob pending queue if upgrade throughput becomes a constraint.

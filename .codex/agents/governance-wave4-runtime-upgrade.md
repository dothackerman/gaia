# Agent: Governance Wave 4 — Runtime Upgrade via Governance

## Session start

Before any action:
1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.
3. Read `docs/plans/governance-on-chain.md` for full milestone context.
4. Read all merged governance ADRs from Waves 1–3 in `docs/decisions/`
   (proposal storage, membership storage, generalized execution, time-locks,
   proposal classes).

Do not write any code until all are loaded.

---

## Prerequisite

**Both Wave 3A and Wave 3B PRs must be merged into `main` before this agent starts.**
Run `git pull origin main` first.

**Branch:** create from `main` as `codex/governance-wave4-runtime-upgrade`

**Bump `spec_version`** in `runtime/src/lib.rs` to one above whatever Wave 2
set (Wave 2 sets 103, so this wave sets 104).
Bump `transaction_version` if extrinsic encoding of `submit_proposal` changes
(it does: new `UpgradeRuntime` variant added to the action enum).

---

## Goal

Allow active members to vote on a runtime upgrade. A Constitutional-class
proposal with `GovernanceAction::UpgradeRuntime` payload, when executed,
replaces the on-chain WASM runtime via `frame_system::set_code`.

This is the final layer of on-chain governance: any logic can now change via
member vote, including governance rules themselves.

---

## Implementation steps

### Step 1 — Add UpgradeRuntime to GovernanceAction

In `pallets/proposals/src/lib.rs`, add the new variant to the existing enum:

```rust
GovernanceAction::UpgradeRuntime { code_hash: [u8; 32] },
```

The `code_hash` is the Blake2-256 hash of the WASM blob that must already be
uploaded on-chain before the proposal executes.

### Step 2 — Add pending WASM storage

```rust
#[pallet::storage]
pub type PendingRuntimeCode<T: Config> =
    StorageValue<_, BoundedVec<u8, T::MaxRuntimeCodeSize>>;
```

Add `MaxRuntimeCodeSize: Get<u32>` to the `Config` trait with a sensible
default in the runtime (e.g., `ConstU32<{ 10 * 1024 * 1024 }>` = 10 MB).

### Step 3 — Add upload_runtime_code dispatchable

Any active member may upload the WASM blob before the proposal executes:

```rust
pub fn upload_runtime_code(
    origin,
    code: Vec<u8>,
) -> DispatchResult {
    let caller = ensure_signed(origin)?;
    ensure!(T::Membership::is_active_member(&caller), Error::<T>::NotActiveMember);
    let bounded: BoundedVec<_, _> = code.try_into().map_err(|_| Error::<T>::RuntimeCodeTooLarge)?;
    PendingRuntimeCode::<T>::put(bounded);
    let hash = sp_io::hashing::blake2_256(&code);
    Self::deposit_event(Event::RuntimeCodeUploaded { uploader: caller, code_hash: hash });
    Ok(())
}
```

Emit `RuntimeCodeUploaded { uploader: AccountId, code_hash: [u8; 32] }`.
Members verify the hash matches the binary they reviewed off-chain.

### Step 4 — Handle UpgradeRuntime in execute_proposal

Add arm to the `match proposal.action` block:

```rust
GovernanceAction::UpgradeRuntime { code_hash } => {
    let code = PendingRuntimeCode::<T>::get()
        .ok_or(Error::<T>::NoPendingRuntimeCode)?;
    let actual_hash = sp_io::hashing::blake2_256(&code);
    ensure!(actual_hash == code_hash, Error::<T>::RuntimeCodeHashMismatch);
    frame_system::Pallet::<T>::set_code(frame_system::RawOrigin::Root.into(), code.into_inner())
        .map_err(|e| e.error)?;
    PendingRuntimeCode::<T>::kill();
    Self::deposit_event(Event::RuntimeUpgradeExecuted { code_hash });
    Ok(())
}
```

Emit `RuntimeUpgradeExecuted { code_hash: [u8; 32] }`.

### Step 5 — Enforce Constitutional class in class-action mapping

In `submit_proposal`, the class-action mapping check (added in Wave 2) must
include the new variant:

```
GovernanceAction::UpgradeRuntime { .. } → ProposalClass::Constitutional
```

If not already present, add it. Return `Error::ProposalClassMismatch` if the
caller submits with a non-Constitutional class.

### Step 6 — Add new error variants

- `NoPendingRuntimeCode` — execute called but no code uploaded.
- `RuntimeCodeHashMismatch` — uploaded code does not match proposal hash.
- `RuntimeCodeTooLarge` — uploaded code exceeds `MaxRuntimeCodeSize`.

---

## Tests to write

Unit tests in `pallets/proposals/src/lib.rs`:

- `upload_runtime_code_stores_blob_and_emits_hash()` — upload code, verify
  stored, verify event with correct hash.
- `upload_runtime_code_rejects_non_member()` — non-member upload returns
  `NotActiveMember`.
- `execute_runtime_upgrade_fails_with_wrong_hash()` — proposal hash does not
  match uploaded blob → `RuntimeCodeHashMismatch`.
- `execute_runtime_upgrade_fails_without_pending_code()` — no code uploaded →
  `NoPendingRuntimeCode`.
- `execute_runtime_upgrade_requires_constitutional_class()` — submit with
  Governance class → `ProposalClassMismatch` at submission.
- `execute_runtime_upgrade_clears_pending_code_on_success()` — after execution,
  `PendingRuntimeCode` is cleared.

Note: `set_code` in a mock environment will not run a real WASM replacement.
Test that the call is dispatched correctly using mock setup. If `set_code`
requires `EnsureRoot`, verify the root origin is correctly synthesised.

Integration test in `tests/proposals.rs`:

- `runtime_upgrade_proposal_end_to_end()` — upload code, submit Constitutional
  proposal, meet 90% threshold, tally → approved, wait delay, execute →
  `RuntimeUpgradeExecuted` event emitted, pending code cleared.

---

## CLI updates (tester-cli/)

Add subcommand: `proposals upload-runtime-code --file <path>`
Reads the file, submits `upload_runtime_code` extrinsic, prints the Blake2-256
hash to stdout so the user can include it in the proposal.

Add action option: `proposals submit --class constitutional --action upgrade-runtime --hash <hex>`

Regenerate `tester-cli/artifacts/gaia.scale` after `cargo build`.

---

## ADR required

Create `docs/decisions/<next>-runtime-upgrade-governance.md`.
Expected number is ADR-014 if no additional ADRs land before Wave 4 merge.

Title: "Runtime upgrade via Constitutional-class governance proposal"

Document:
- Why runtime upgrade must be Constitutional class (highest threshold).
- The two-step upload-then-propose flow and why it is safe (hash binds the
  proposal to a specific binary reviewed off-chain).
- The `PendingRuntimeCode` storage and why it allows only one pending blob
  (last-write wins; any member can overwrite, but the hash in the proposal
  is fixed at submission time).
- Risk: a malicious member can overwrite `PendingRuntimeCode` after a proposal
  is submitted. Mitigation: the hash check at execution time ensures only the
  correct blob is used; an overwrite would cause the execution to fail, not
  succeed with wrong code. Document this explicitly.
- Accepted trade-off: no multi-blob support; one pending upgrade at a time.

---

## Update docs/current-state.md

After all tests pass and before committing, update `docs/current-state.md`:
- Bump `spec_version` entry.
- Add `UpgradeRuntime` to the GovernanceAction list.
- Update "Governance hardening status" section to reflect completion of this
  milestone.
- Update total test count.

---

## Cargo sequence

Use `just all` to run the full sequence, or step through manually:

```
cargo check                                      # 1. must pass first
cargo clippy -p gaia-proposals -- -D warnings    # 2. fix ALL warnings
cargo fmt --all -- --check                       # 3. formatting clean
cargo test --workspace                           # 4. all tests pass
cargo build --workspace                          # 5. full build
```

Never skip a level. `just all` runs steps 1–5 in order.

---

## Completion

Commit: `feat(proposals): add runtime upgrade governance via Constitutional-class proposal`

Push to `codex/governance-wave4-runtime-upgrade` and open a PR targeting `main`.
PR title: "Add runtime upgrade governance via Constitutional-class proposal"

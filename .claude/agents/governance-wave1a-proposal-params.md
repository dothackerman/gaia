# Agent: Governance Wave 1A — Proposal Parameter Storage

## Session start

Before any action:
1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.
3. Read `docs/plans/governance-on-chain.md` for full milestone context.

Do not write any code until all three are loaded.

---

## Context

This agent implements Wave 1A of the on-chain governance milestone.
It runs **in parallel with Wave 1B** (`governance-wave1b-membership-params.md`).
The two agents do not touch each other's files.

**This agent owns:** `pallets/proposals/` and `runtime/src/configs/proposals.rs` only.
**Do not touch:** membership pallet, treasury pallet, `runtime/src/lib.rs`.

**Branch:** create from `main` as `claude/governance-wave1a-proposal-params`

---

## Goal

Move all hardcoded proposal-governance constants from compile-time into
on-chain `StorageValue` items with genesis defaults. Add `EnsureRoot`-gated
setter dispatchables as a placeholder (upgraded to `GovernanceOrigin` in Wave 2).

---

## StorageValues to add in `pallets/proposals/src/lib.rs`

```rust
#[pallet::storage]
pub type ProposalVotingPeriod<T: Config> =
    StorageValue<_, BlockNumberFor<T>, ValueQuery>;

#[pallet::storage]
pub type ExecutionDelay<T: Config> =
    StorageValue<_, BlockNumberFor<T>, ValueQuery>;

#[pallet::storage]
pub type StandardApprovalNumerator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type StandardApprovalDenominator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type GovernanceApprovalNumerator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type GovernanceApprovalDenominator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type ConstitutionalApprovalNumerator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type ConstitutionalApprovalDenominator<T: Config> =
    StorageValue<_, u32, ValueQuery>;
```

Genesis defaults (from `docs/plans/governance-on-chain.md`):

| StorageValue | Default |
|---|---|
| `ProposalVotingPeriod` | `100_800` (fast-local build: `20`) |
| `ExecutionDelay` | `0` |
| `StandardApprovalNumerator` | `1` |
| `StandardApprovalDenominator` | `2` |
| `GovernanceApprovalNumerator` | `4` |
| `GovernanceApprovalDenominator` | `5` |
| `ConstitutionalApprovalNumerator` | `9` |
| `ConstitutionalApprovalDenominator` | `10` |

---

## Implementation steps

1. Add the eight `StorageValue` declarations above to `pallets/proposals/src/lib.rs`.

2. Add `#[pallet::genesis_config]` and `#[pallet::genesis_build]` blocks
   initialising all eight values. For the `fast-local` feature gate on
   `ProposalVotingPeriod`, use `#[cfg(feature = "fast-local")]` inside
   `genesis_build` to set 20 vs 100_800.

3. Remove the `VotingPeriod` associated type from `Config` (or keep with a
   `#[deprecated]` marker if removal causes cascading issues — document why).
   Replace every `T::VotingPeriod::get()` call site with
   `ProposalVotingPeriod::<T>::get()`.

4. Add setter dispatchables (origin: `ensure_root(origin)?` for now):
   ```rust
   pub fn set_proposal_voting_period(origin, blocks: BlockNumberFor<T>)
   pub fn set_execution_delay(origin, blocks: BlockNumberFor<T>)
   pub fn set_standard_approval_threshold(origin, numerator: u32, denominator: u32)
   pub fn set_governance_approval_threshold(origin, numerator: u32, denominator: u32)
   pub fn set_constitutional_approval_threshold(origin, numerator: u32, denominator: u32)
   ```
   Guards:
   - `denominator == 0` → `Error::InvalidThreshold`
   - `numerator > denominator` → `Error::InvalidThreshold`

5. Emit events for each setter: `ProposalVotingPeriodSet`, `ExecutionDelaySet`,
   `StandardThresholdSet`, `GovernanceThresholdSet`, `ConstitutionalThresholdSet`.

6. Update `runtime/src/configs/proposals.rs` to provide genesis config values
   instead of feature-gated `ConstU32`. Remove the `VotingPeriod` Config binding
   from the runtime impl if step 3 removed it from the trait.

7. Update `pallets/proposals/src/mock.rs` genesis to include the new StorageValues.

8. **Migrate `tests/src/common.rs` integration test helper.**
   The shared helper `advance_past_voting_period()` (line ~129) currently reads
   from the `VotingPeriod` Config associated type:
   ```rust
   // BEFORE (will not compile once VotingPeriod is removed from Config):
   let period = <<Runtime as gaia_proposals::pallet::Config>::VotingPeriod as Get<u32>>::get();
   ```
   Replace with a storage read:
   ```rust
   // AFTER:
   let period = gaia_proposals::ProposalVotingPeriod::<Runtime>::get();
   ```
   This is your file to own. Wave 1B owns the adjacent membership helper on line ~136.
   The changes are on different lines — no merge conflict expected.

---

## Tests to write

Add inside `#[cfg(test)] mod tests` in `pallets/proposals/src/lib.rs`:

- `set_proposal_voting_period_updates_storage()` — root sets period; read back matches.
- `set_execution_delay_updates_storage()` — root sets delay; read back matches.
- `set_standard_threshold_updates_storage()` — valid numerator/denominator; stored.
- `set_threshold_rejects_zero_denominator()` — denominator 0 returns `InvalidThreshold`.
- `set_threshold_rejects_numerator_greater_than_denominator()` — n > d returns `InvalidThreshold`.
- `non_root_cannot_call_setters()` — signed origin returns `BadOrigin`.

All existing 17 proposal unit tests must still pass without modification.

---

## ADR required

Create `docs/decisions/008-governance-parameter-storage.md`.

Title: "On-chain storage for proposal governance parameters"

Document:
- Why parameters move from compile-time constants to on-chain StorageValues.
- Genesis defaults and why they preserve current behaviour.
- The placeholder `EnsureRoot` origin and why it will be upgraded to
  `GovernanceOrigin` in Wave 2.
- Accepted trade-off: `EnsureRoot` means a developer can still change parameters
  directly between now and Wave 2 completion.

---

## Cargo sequence

Use `just all` to run the full sequence, or step through manually:

```
cargo check                                           # 1. must pass first
cargo clippy -p gaia-proposals -- -D warnings         # 2. fix ALL warnings
cargo fmt --all -- --check                            # 3. formatting clean
cargo test --workspace                                # 4. all tests pass
cargo build --workspace                               # 5. full build
```

Never skip a level. `just all` runs steps 1–5 in order.

---

## Completion

Commit with message: `feat(proposals): move governance parameters to on-chain storage`

Push to `claude/governance-wave1a-proposal-params` and open a PR targeting `main`.
PR title: "Move proposal governance parameters to on-chain storage"

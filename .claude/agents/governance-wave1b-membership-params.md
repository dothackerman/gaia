# Agent: Governance Wave 1B — Membership Parameter Storage

## Session start

Before any action:
1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.
3. Read `docs/plans/governance-on-chain.md` for full milestone context.
4. Read `docs/decisions/005-suspension-unanimity.md` — ADR-005 semantics must be
   preserved as the genesis default.
5. Read `docs/decisions/007-membership-proposal-lifecycle-and-threshold-baseline.md`
   — ADR-007 follow-up direction is part of what this wave begins to address.

Do not write any code until all five are loaded.

---

## Context

This agent implements Wave 1B of the on-chain governance milestone.
It runs **in parallel with Wave 1A** (`governance-wave1a-proposal-params.md`).
The two agents do not touch each other's files.

**This agent owns:** `pallets/membership/` and `runtime/src/configs/membership.rs` only.
**Do not touch:** proposals pallet, treasury pallet, `runtime/src/lib.rs`.

**Branch:** create from `main` as `claude/governance-wave1b-membership-params`

---

## Goal

Move all hardcoded membership-governance constants from compile-time and from
magic multipliers in dispatchable logic into on-chain `StorageValue` items with
genesis defaults. Preserve ADR-005 (unanimity) as the default suspension
threshold. Add `EnsureRoot`-gated setter dispatchables as placeholders.

---

## StorageValues to add in `pallets/membership/src/lib.rs`

```rust
#[pallet::storage]
pub type MembershipVotingPeriod<T: Config> =
    StorageValue<_, BlockNumberFor<T>, ValueQuery>;

#[pallet::storage]
pub type MembershipApprovalNumerator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type MembershipApprovalDenominator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type SuspensionNumerator<T: Config> =
    StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub type SuspensionDenominator<T: Config> =
    StorageValue<_, u32, ValueQuery>;
```

Genesis defaults:

| StorageValue | Default | Preserves |
|---|---|---|
| `MembershipVotingPeriod` | `100_800` (fast-local: `20`) | Current feature-gated constant |
| `MembershipApprovalNumerator` | `4` | `lib.rs:547` magic `4` |
| `MembershipApprovalDenominator` | `5` | `lib.rs:547` magic `5` |
| `SuspensionNumerator` | `1` | ADR-005 unanimity |
| `SuspensionDenominator` | `1` | ADR-005 unanimity |

---

## Implementation steps

1. Add the five `StorageValue` declarations above.

2. Add `#[pallet::genesis_config]` and `#[pallet::genesis_build]` blocks.
   Use `#[cfg(feature = "fast-local")]` inside `genesis_build` for the voting
   period (20 vs 100_800).

3. **Replace the membership approval multipliers** at approximately `lib.rs:547`:
   ```rust
   // Before:
   yes_votes.saturating_mul(5) >= active_member_snapshot.saturating_mul(4)

   // After:
   let n = MembershipApprovalNumerator::<T>::get();
   let d = MembershipApprovalDenominator::<T>::get();
   yes_votes.saturating_mul(d) >= active_member_snapshot.saturating_mul(n)
   ```

4. **Replace the unanimity check** at approximately `lib.rs:516-517`:
   ```rust
   // Before:
   let required = ActiveMemberCount::<T>::get().saturating_sub(1);
   if approvals == required { … }

   // After:
   let n = SuspensionNumerator::<T>::get();
   let d = SuspensionDenominator::<T>::get();
   let others = ActiveMemberCount::<T>::get().saturating_sub(1);
   if approvals.saturating_mul(d) >= others.saturating_mul(n) { … }
   ```
   Default `(1, 1)` makes `approvals * 1 >= others * 1` → same as
   `approvals == others` (unanimity). ADR-005 semantics preserved.

5. Remove or deprecate the `VotingPeriod` associated type from `Config`.
   Replace all `T::VotingPeriod::get()` calls with
   `MembershipVotingPeriod::<T>::get()`.

6. Add setter dispatchables (origin: `ensure_root(origin)?` for now):
   ```rust
   pub fn set_membership_voting_period(origin, blocks: BlockNumberFor<T>)
   pub fn set_membership_approval_threshold(origin, numerator: u32, denominator: u32)
   pub fn set_suspension_threshold(origin, numerator: u32, denominator: u32)
   ```
   Guards:
   - `denominator == 0` → `Error::InvalidThreshold`
   - `numerator > denominator` → `Error::InvalidThreshold`

7. Emit events: `MembershipVotingPeriodSet`, `MembershipApprovalThresholdSet`,
   `SuspensionThresholdSet`.

8. Update `runtime/src/configs/membership.rs` to use genesis config values
   instead of feature-gated `ConstU32`. Remove `VotingPeriod` from runtime impl
   if removed from Config.

9. Update `pallets/membership/src/mock.rs` genesis.

10. **Migrate `tests/src/common.rs` integration test helper.**
    The shared helper `advance_past_membership_voting_period()` (line ~136) currently
    reads from the `VotingPeriod` Config associated type:
    ```rust
    // BEFORE (will not compile once VotingPeriod is removed from Config):
    let period = <<Runtime as gaia_membership::pallet::Config>::VotingPeriod as Get<u32>>::get();
    ```
    Replace with a storage read:
    ```rust
    // AFTER:
    let period = gaia_membership::MembershipVotingPeriod::<Runtime>::get();
    ```
    This is your file to own. Wave 1A owns the adjacent proposal helper on line ~130.
    The changes are on different lines — no merge conflict expected.

11. **Implement `MembershipGovernance` trait on the membership pallet.**
    The trait is already stubbed in `pallets/proposals/src/lib.rs`.
    Add an `impl` in `pallets/membership/src/lib.rs` that delegates to the setter
    dispatchables you add in step 6. The signatures **must match exactly**:
    ```rust
    impl<T: Config> gaia_proposals::MembershipGovernance<
        frame_system::pallet_prelude::OriginFor<T>,
        frame_support::pallet_prelude::BlockNumberFor<T>,
    > for Pallet<T> {
        fn set_voting_period(origin: OriginFor<T>, blocks: BlockNumberFor<T>) -> DispatchResult {
            Self::set_membership_voting_period(origin, blocks)
        }
        fn set_approval_threshold(origin: OriginFor<T>, numerator: u32, denominator: u32) -> DispatchResult {
            Self::set_membership_approval_threshold(origin, numerator, denominator)
        }
        fn set_suspension_threshold(origin: OriginFor<T>, numerator: u32, denominator: u32) -> DispatchResult {
            Self::set_suspension_threshold(origin, numerator, denominator)
        }
    }
    ```
    This makes Wave 2 (Agent C) a clean wire-up, not a signature negotiation.

---

## Tests to write

Add inside `#[cfg(test)] mod tests` in `pallets/membership/src/lib.rs`:

- `set_membership_voting_period_updates_storage()` — root sets period; stored.
- `set_membership_approval_threshold_updates_storage()` — valid n/d; stored.
- `set_suspension_threshold_updates_storage()` — valid n/d; stored.
- `set_threshold_rejects_zero_denominator()` — d=0 returns `InvalidThreshold`.
- `set_threshold_rejects_numerator_greater_than_denominator()` — n>d returns error.
- `membership_approval_uses_stored_threshold()` — with default 4/5, 4 yes of 5
  snapshot approves; 3 yes of 5 does not.
- `suspension_threshold_default_requires_unanimity()` — with 3 active members,
  2 suspenders must both approve before suspension triggers (ADR-005 preserved).
- `non_root_cannot_call_setters()` — signed origin returns `BadOrigin`.

All 30 existing membership unit tests must still pass without modification.

---

## ADR required

Create `docs/decisions/009-governance-membership-parameter-storage.md`.

Title: "On-chain storage for membership governance parameters"

Document:
- Why the 80% threshold and voting period move to StorageValues.
- The new suspension formula (`approvals * d >= others * n`) and how
  default `(1, 1)` preserves ADR-005 unanimity semantics.
- Genesis defaults and why they match current behaviour exactly.
- The placeholder `EnsureRoot` origin and the Wave 2 upgrade plan.
- Cross-reference ADR-005 (suspension) and ADR-007 (follow-up direction).

---

## Cargo sequence

Use `just all` to run the full sequence, or step through manually:

```
cargo check                                           # 1. must pass first
cargo clippy -p gaia-membership -- -D warnings        # 2. fix ALL warnings
cargo fmt --all -- --check                            # 3. formatting clean
cargo test --workspace                                # 4. all tests pass
cargo build --workspace                               # 5. full build
```

Never skip a level. `just all` runs steps 1–5 in order.

---

## Completion

Commit with message: `feat(membership): move governance parameters to on-chain storage`

Push to `claude/governance-wave1b-membership-params` and open a PR targeting `main`.
PR title: "Move membership governance parameters to on-chain storage"

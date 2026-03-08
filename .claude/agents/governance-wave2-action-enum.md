# Agent: Governance Wave 2 — GovernanceAction Enum and Generalized Proposal Execution

## Session start

Before any action:
1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.
3. Read `docs/plans/governance-on-chain.md` for full milestone context.
4. Read `docs/decisions/008-governance-parameter-storage.md` (written in Wave 1A).
5. Read `docs/decisions/009-governance-membership-parameter-storage.md` (Wave 1B).

Do not write any code until all five are loaded.

---

## Prerequisite

**Both Wave 1A and Wave 1B PRs must be merged into `main` before this agent starts.**
Run `git pull origin main` first.

---

## Context

**This agent owns:**
- `pallets/proposals/src/lib.rs` (GovernanceAction enum, GovernanceOrigin,
  updated submit/execute, trait extensions)
- `runtime/src/lib.rs` (GovernanceOrigin wiring)
- `runtime/src/configs/proposals.rs` (if Config changes needed)
- `tests/proposals.rs` (integration tests migrated to new signature)
- `tester-cli/src/` (CLI commands updated)
- `tester-cli/artifacts/gaia.scale` (regenerated metadata)

**Branch:** create from `main` as `claude/governance-wave2-action-enum`

**Bump `spec_version`** in `runtime/src/lib.rs`: 101 → 102.
Bump `transaction_version` if the extrinsic encoding of `submit_proposal` changes
(it will, because the signature changes).

---

## Goal

1. Define the `GovernanceAction` enum in the proposals pallet.
2. Define a `GovernanceOrigin` so setter extrinsics cannot be called by arbitrary accounts.
3. Update `submit_proposal` to accept a `class` and `action` payload.
4. Update `execute_proposal` to dispatch the `GovernanceAction` payload.
5. Replace `EnsureRoot` in setter dispatchables with `EnsureGovernance`.
6. Migrate all integration tests and CLI to the new proposal signature.

---

## GovernanceAction enum

Add to `pallets/proposals/src/lib.rs` (before or after the Proposal struct):

```rust
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
pub enum GovernanceAction<AccountId, Balance, BlockNumber> {
    DisburseToAccount { recipient: AccountId, amount: Balance },
    SetProposalVotingPeriod { blocks: BlockNumber },
    SetExecutionDelay { blocks: BlockNumber },
    SetStandardApprovalThreshold { numerator: u32, denominator: u32 },
    SetGovernanceApprovalThreshold { numerator: u32, denominator: u32 },
    SetConstitutionalApprovalThreshold { numerator: u32, denominator: u32 },
    SetMembershipVotingPeriod { blocks: BlockNumber },
    SetMembershipApprovalThreshold { numerator: u32, denominator: u32 },
    SetSuspensionThreshold { numerator: u32, denominator: u32 },
    // UpgradeRuntime is added in Wave 4 — do not add it here.
}
```

Resolve generic parameters through `T::Config`:
`GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>`.

---

## ProposalClass enum

```rust
#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ProposalClass { Standard, Governance, Constitutional }
```

---

## GovernanceOrigin

**Implementation approach:** Sovereign-account origin.

The proposals pallet config gets a new associated constant:
```rust
#[pallet::constant]
type GovernancePalletId: Get<PalletId>;
```

The governance account is `T::GovernancePalletId::get().into_account_truncating::<T::AccountId>()`.

In `execute_proposal`, after all checks pass, call the setter dispatchable by
constructing a signed origin from this account:
```rust
let governance_account = T::GovernancePalletId::get()
    .into_account_truncating::<T::AccountId>();
let governance_origin = RawOrigin::Signed(governance_account.clone()).into();
```

Setter dispatchables (in both pallets) check:
```rust
let caller = ensure_signed(origin)?;
ensure!(
    caller == T::GovernancePalletId::get().into_account_truncating::<T::AccountId>(),
    Error::NotGovernanceOrigin
);
```

This replaces the `ensure_root` from Wave 1. The governance account is
unfundable by normal users (it is a PalletId-derived account, not a key pair),
so no one can impersonate it from outside `execute_proposal`.

Add `GovernancePalletId: PalletId = PalletId(*b"ga/govn0")` in
`runtime/src/configs/proposals.rs`.

---

## Updated Proposal struct

Add two fields:
```rust
pub struct Proposal<AccountId, Balance, BlockNumber> {
    // existing fields …
    pub class: ProposalClass,
    pub action: GovernanceAction<AccountId, Balance, BlockNumber>,
    pub approved_at: Option<BlockNumber>,  // populated by tally_proposal in Wave 3
}
```

`approved_at` is `None` until Wave 3 (Agent D) populates it. Add it now to
avoid a second storage migration.

---

## Updated submit_proposal signature

```rust
pub fn submit_proposal(
    origin,
    title: BoundedVec<u8, ConstU32<MAX_TITLE_LEN>>,
    description: BoundedVec<u8, ConstU32<MAX_DESC_LEN>>,
    class: ProposalClass,
    action: GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>,
) -> DispatchResult
```

Enforce class-action mapping (see `docs/plans/governance-on-chain.md`).
Mismatched class → `Error::ProposalClassMismatch`.

Remove the old `amount` and `recipient` parameters (now part of the action).

---

## Updated execute_proposal

Match on `proposal.action` and dispatch:

```rust
match proposal.action {
    GovernanceAction::DisburseToAccount { recipient, amount } =>
        T::Treasury::disburse(&recipient, amount)?,

    GovernanceAction::SetProposalVotingPeriod { blocks } =>
        Self::set_proposal_voting_period(governance_origin.clone(), blocks)?,

    GovernanceAction::SetExecutionDelay { blocks } =>
        Self::set_execution_delay(governance_origin.clone(), blocks)?,

    GovernanceAction::SetStandardApprovalThreshold { numerator, denominator } =>
        Self::set_standard_approval_threshold(governance_origin.clone(), numerator, denominator)?,

    GovernanceAction::SetGovernanceApprovalThreshold { numerator, denominator } =>
        Self::set_governance_approval_threshold(governance_origin.clone(), numerator, denominator)?,

    GovernanceAction::SetConstitutionalApprovalThreshold { numerator, denominator } =>
        Self::set_constitutional_approval_threshold(governance_origin.clone(), numerator, denominator)?,

    GovernanceAction::SetMembershipVotingPeriod { blocks } =>
        T::MembershipGovernance::set_voting_period(governance_origin.clone(), blocks)?,

    GovernanceAction::SetMembershipApprovalThreshold { numerator, denominator } =>
        T::MembershipGovernance::set_approval_threshold(governance_origin.clone(), numerator, denominator)?,

    GovernanceAction::SetSuspensionThreshold { numerator, denominator } =>
        T::MembershipGovernance::set_suspension_threshold(governance_origin.clone(), numerator, denominator)?,
}
```

Invariant I-3 (single execution) still enforced before this match.

---

## New cross-pallet trait: MembershipGovernance

Define in `pallets/proposals/src/lib.rs` (proposals defines the trait,
membership implements it — following existing pattern):

```rust
pub trait MembershipGovernance<Origin, BlockNumber> {
    fn set_voting_period(origin: Origin, blocks: BlockNumber) -> DispatchResult;
    fn set_approval_threshold(origin: Origin, numerator: u32, denominator: u32) -> DispatchResult;
    fn set_suspension_threshold(origin: Origin, numerator: u32, denominator: u32) -> DispatchResult;
}
```

Implement in `pallets/membership/src/lib.rs` by delegating to the existing
setter dispatchables.

Wire in runtime: `type MembershipGovernance = pallet_membership::Pallet<Runtime>;`

---

## Integration tests

Migrate `tests/proposals.rs` to new `submit_proposal` signature.
Every existing test that calls `submit_proposal` must be updated to pass
`class: ProposalClass::Standard` and
`action: GovernanceAction::DisburseToAccount { recipient, amount }`.

New integration tests to add:
- `governance_proposal_changes_voting_period()` — submit Governance-class proposal with
  `SetProposalVotingPeriod`, vote, tally, execute — verify StorageValue updated.
- `governance_proposal_changes_membership_threshold()` — same flow for
  `SetMembershipApprovalThreshold`.
- `submit_proposal_rejects_class_action_mismatch()` — DisburseToAccount with
  Governance class returns `ProposalClassMismatch`.
- `non_governance_origin_cannot_call_setters()` — direct call to setter with
  signed origin (not governance account) returns `NotGovernanceOrigin`.

---

## CLI updates (tester-cli/)

Update `proposals submit` command to accept:
- `--class standard|governance|constitutional`
- `--action disburse|set-proposal-voting-period|set-execution-delay|…`
- Action-specific flags (e.g., `--recipient`, `--amount` for disburse;
  `--blocks` for period changes; `--numerator`/`--denominator` for thresholds)

Default class for `disburse` is `standard` (backwards-compatible ergonomic default).

Regenerate `tester-cli/artifacts/gaia.scale` after `cargo build`.

---

## ADR required

Create `docs/decisions/010-generalized-proposal-execution.md`.

Title: "Generalized proposal execution with GovernanceAction enum"

Document:
- Why typed enum over raw RuntimeCall.
- The sovereign-account GovernanceOrigin approach and why it is safe.
- The class-action mapping and why it is enforced at submission time.
- `DisburseToAccount` as a GovernanceAction variant replacing the implicit
  treasury-only proposal model.
- The `MembershipGovernance` trait as the cross-pallet coupling mechanism.

---

## Cargo sequence

```
cargo check   ← must pass before proceeding
cargo clippy  ← fix ALL warnings
cargo test    ← ALL tests must pass (including migrated integration tests)
cargo build   ← regenerate .scale metadata after this
```

---

## Completion

Commit with message: `feat(proposals): add GovernanceAction enum and generalize proposal execution`

Push to `claude/governance-wave2-action-enum` and open a PR targeting `main`.
PR title: "Generalize proposal execution with GovernanceAction enum"

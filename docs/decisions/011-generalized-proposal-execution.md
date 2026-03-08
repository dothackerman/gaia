# ADR 011 — Generalized proposal execution with GovernanceAction enum

## Context

Before Wave 2, treasury proposals carried implicit disbursement fields
(`amount`, `event_block`) and execution was limited to organizer-triggered
`Treasury::disburse`. Governance parameter setters were root-only placeholders,
which prevented member-approved proposals from directly changing governance
configuration.

## Decision

Wave 2 introduces typed proposal payloads and governance-controlled execution:

- Added `ProposalClass` and typed
  `GovernanceAction<AccountId, Balance, BlockNumber>` to proposals.
- Updated `submit_proposal` to accept `(class, action)` and reject mismatched
  class/action pairs with `ProposalClassMismatch`.
- Added pallet-owned governance origin via `GovernancePalletId` sovereign
  account (`ga/govn0`).
- Replaced `EnsureRoot` setter authorization with governance-origin checks in
  both proposals and membership pallets (`NotGovernanceOrigin` on direct calls).
- Updated `execute_proposal` to dispatch the typed action payload and preserve
  single-execution invariant I-3.
- Added `approved_at` field to proposal storage in preparation for Wave 3
  execution-delay enforcement.
- Wired runtime integration:
  - `type MembershipGovernance = gaia_membership::Pallet<Runtime>`
  - shared `GovernancePalletId` across proposals + membership configs
- Bumped runtime versions:
  - `spec_version`: `102 -> 103`
  - `transaction_version`: `1 -> 2` (submit extrinsic encoding changed)

## Consequences

**Positive**

- Proposal execution now supports treasury and governance parameter actions in a
  single typed pathway.
- Governance parameter changes can be applied only through approved proposal
  execution, not arbitrary signed accounts.
- The cross-pallet coupling remains trait-based (`MembershipGovernance`),
  matching AGENTS architecture constraints.

**Negative / accepted trade-offs**

- CLI metadata/UX still requires follow-up to expose full action surface in a
  user-friendly way.
- Proposal tallying remains `yes > no` in Wave 2 by design; class-threshold
  routing is deferred to Wave 3.

## Follow-up

- Wave 3A enforces `ExecutionDelay` using `approved_at`.
- Wave 3B activates class-based threshold routing during tally.
- Wave 4 adds `UpgradeRuntime` governance flow.

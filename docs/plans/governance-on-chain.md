# Milestone: True On-Chain Governance

> **Status:** Planned — not yet implemented.
> **Last updated:** 2026-03-08
> **Prerequisite reading:** `AGENTS.md`, `docs/current-state.md`, `docs/domain-model.md`

---

## Motivation

The current implementation has **no on-chain governance system**. Every
governance parameter (approval thresholds, voting windows, suspension rules) is
a compile-time constant or hardcoded pallet logic. Changing anything requires:

1. Code edit in `pallets/*/src/lib.rs` or `runtime/src/configs/*.rs`
2. Recompile with correct feature flags
3. `spec_version` bump
4. Runtime upgrade extrinsic by a developer

This milestone makes all those parameters and, ultimately, all logic changeable
by active member vote — without developer intervention.

It also directly addresses the follow-up direction documented in ADR-007:

> Future governance hardening should evaluate:
> - quorum/turnout requirements for all voting systems,
> - explicit threshold policy consistency across membership, treasury proposals,
>   and suspension,
> - automatic expiry/finalization mechanisms if manual finalize becomes an
>   operational burden.

---

## Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Proposal payload | Typed `GovernanceAction` enum | Human-readable, safe, extensible via runtime upgrade |
| Governance parameters | On-chain `StorageValue` with genesis defaults | Changeable without recompile |
| Governance privilege | `GovernanceOrigin` — satisfied only by `execute_proposal` | Prevents accounts from calling setters directly |
| Meta-governance | `ProposalClass` enum: `Standard / Governance / Constitutional` | Threshold to change a threshold must be ≥ the threshold being changed |
| Time-locks | `ExecutionDelay` StorageValue enforced in `execute_proposal` | Guards against surprise governance attacks |
| Runtime upgrade | `UpgradeRuntime` GovernanceAction variant (Wave 4) | Allows any logic to change via Constitutional-class vote |

---

## Relationship to Domain Model

`docs/domain-model.md` describes aspirational concepts not yet implemented:

- **`pending` member status** — candidates awaiting admission are stored in
  `Candidates` storage separately; the domain model treats this as `pending`.
  This milestone does not address this gap (separate future work).
- **`draft` and `disputed` proposal states** — proposals go live immediately;
  the domain model describes these as aspirational.
  This milestone does not address this gap (separate future work).
- **`votingThreshold` on Community** — the domain model places `votingThreshold`
  as an attribute of `Community`, not hardcoded in a pallet. This milestone
  implements that relationship by moving thresholds into on-chain storage.

---

## Governance Parameter Specification

### In `pallets/proposals/`

| StorageValue | Type | Genesis Default | Replaces |
|---|---|---|---|
| `ProposalVotingPeriod` | `BlockNumber` | `100_800` (fast-local: `20`) | `runtime/src/configs/proposals.rs:17` |
| `ExecutionDelay` | `BlockNumber` | `0` | Does not exist yet |
| `StandardApprovalNumerator` | `u32` | `1` | implicit `yes > no` |
| `StandardApprovalDenominator` | `u32` | `2` | implicit |
| `GovernanceApprovalNumerator` | `u32` | `4` | Does not exist yet |
| `GovernanceApprovalDenominator` | `u32` | `5` | Does not exist yet |
| `ConstitutionalApprovalNumerator` | `u32` | `9` | Does not exist yet |
| `ConstitutionalApprovalDenominator` | `u32` | `10` | Does not exist yet |

> Standard `(1, 2)`: `yes * 2 >= total * 1` → equivalent to `yes > no`.
> Governance `(4, 5)`: 80%.
> Constitutional `(9, 10)`: 90%.

### In `pallets/membership/`

| StorageValue | Type | Genesis Default | Replaces |
|---|---|---|---|
| `MembershipVotingPeriod` | `BlockNumber` | `100_800` (fast-local: `20`) | `runtime/src/configs/membership.rs:8` |
| `MembershipApprovalNumerator` | `u32` | `4` | `lib.rs:547` magic `4` |
| `MembershipApprovalDenominator` | `u32` | `5` | `lib.rs:547` magic `5` |
| `SuspensionNumerator` | `u32` | `1` | `lib.rs:516` unanimity |
| `SuspensionDenominator` | `u32` | `1` | same |

> Suspension: `approvals * SuspensionDenominator >= (active_count - 1) * SuspensionNumerator`.
> Default `(1, 1)` = unanimity. ADR-005 semantics are preserved as genesis default.

---

## GovernanceAction Enum

Defined in `pallets/proposals/src/lib.rs`.

```rust
pub enum GovernanceAction<AccountId, Balance, BlockNumber> {
    // Treasury
    DisburseToAccount { recipient: AccountId, amount: Balance },
    // Proposal parameters
    SetProposalVotingPeriod { blocks: BlockNumber },
    SetExecutionDelay { blocks: BlockNumber },
    SetStandardApprovalThreshold { numerator: u32, denominator: u32 },
    SetGovernanceApprovalThreshold { numerator: u32, denominator: u32 },
    SetConstitutionalApprovalThreshold { numerator: u32, denominator: u32 },
    // Membership parameters
    SetMembershipVotingPeriod { blocks: BlockNumber },
    SetMembershipApprovalThreshold { numerator: u32, denominator: u32 },
    SetSuspensionThreshold { numerator: u32, denominator: u32 },
    // System (Wave 4 only)
    UpgradeRuntime { code_hash: [u8; 32] },
}
```

---

## ProposalClass Enum and Action Mapping

```rust
pub enum ProposalClass { Standard, Governance, Constitutional }
```

| GovernanceAction variant | Required class |
|---|---|
| `DisburseToAccount` | `Standard` |
| `SetProposalVotingPeriod`, `SetExecutionDelay` | `Governance` |
| `SetMembershipVotingPeriod` | `Governance` |
| `SetStandardApprovalThreshold` | `Constitutional` |
| `SetGovernanceApprovalThreshold` | `Constitutional` |
| `SetConstitutionalApprovalThreshold` | `Constitutional` |
| `SetMembershipApprovalThreshold` | `Constitutional` |
| `SetSuspensionThreshold` | `Constitutional` |
| `UpgradeRuntime` | `Constitutional` |

Mismatched class → `Error::ProposalClassMismatch`.

---

## Implementation Waves

```
Wave 1 (parallel):
  Agent A  →  docs: .codex/agents/governance-wave1a-proposal-params.md
  Agent B  →  docs: .codex/agents/governance-wave1b-membership-params.md

Wave 1 stabilization (serial, required before Wave 2):
  - Backfill migrations for new `StorageValue` governance keys on runtime upgrade
    (protects in-place upgrades from zero-default behavior).

Wave 2 (sequential, after Wave 1 merged):
  Agent C  →  docs: .codex/agents/governance-wave2-action-enum.md

Wave 3 (parallel, after Wave 2 merged):
  Agent D  →  docs: .codex/agents/governance-wave3a-time-locks.md
  Agent E  →  docs: .codex/agents/governance-wave3b-proposal-classes.md

Wave 4 (sequential, after Wave 3 merged):
  Agent F  →  docs: .codex/agents/governance-wave4-runtime-upgrade.md
```

Dependency graph:

```
main
 ├─ [Wave 1 parallel] Agent A + Agent B
 │                          ↓ both merged
 │          [Wave 2] Agent C
 │                          ↓ merged
 │   [Wave 3 parallel] Agent D + Agent E
 │                          ↓ both merged
 │          [Wave 4] Agent F
 │                          ↓ merged
 └─ main (full governance live)
```

---

## ADR Sequence

| Status | Title | Wave | Numbering guidance |
|---|---|---|---|
| Implemented | On-chain storage for proposal governance parameters | 1 | ADR-009 |
| Implemented | On-chain storage for membership governance parameters | 1 | ADR-010 |
| Planned | Generalized proposal execution with GovernanceAction enum | 2 | Next sequential ADR at merge time (expected ADR-011 if no new ADRs land first) |
| Planned | Mandatory execution delay after proposal approval | 3 | Parallel wave: author as ADR draft, merger promotes to next sequential ADR |
| Planned | Three-tier proposal class system for meta-governance | 3 | Parallel wave: author as ADR draft, merger promotes to next sequential ADR |
| Planned | Runtime upgrade via Constitutional-class governance proposal | 4 | Next sequential ADR at merge time after Wave 3 promotions |

---

## Invariants Preserved Throughout

| # | Invariant |
|---|---|
| I-1 | Treasury balance ≥ 0 |
| I-2 | Only active members vote |
| I-3 | A proposal executes at most once |

GovernanceOrigin cannot override these. They are enforced at pallet level, below governance.

---

## Runtime Spec Versions

| After wave | spec_version |
|---|---|
| Before this milestone | 101 |
| Wave 1 stabilization (migration backfill) | 102 |
| Wave 2 (Agent C) | 103 |
| Wave 4 (Agent F) | 104 |

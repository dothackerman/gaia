# ADR 009 — On-chain storage for proposal governance parameters

## Context

The proposals pallet previously used compile-time constants for governance
configuration (notably `VotingPeriod`) and hardcoded majority behavior. Any
parameter change required code edits, a rebuild, and a runtime upgrade by a
developer.

Wave 1 moves proposal-governance parameters into on-chain storage so the runtime
can read and update them through dispatchables without changing pallet code.

## Decision

The proposals pallet now stores governance parameters in `StorageValue` items
initialized at genesis:

- `ProposalVotingPeriod`
- `ExecutionDelay`
- `StandardApprovalNumerator` / `StandardApprovalDenominator`
- `GovernanceApprovalNumerator` / `GovernanceApprovalDenominator`
- `ConstitutionalApprovalNumerator` / `ConstitutionalApprovalDenominator`

Genesis defaults preserve existing behavior:

- voting period: `100_800` blocks (`20` with `fast-local`)
- execution delay: `0`
- standard threshold: `1/2`
- governance threshold: `4/5`
- constitutional threshold: `9/10`

Root-only setter dispatchables were added as a Wave 1 placeholder:

- `set_proposal_voting_period`
- `set_execution_delay`
- `set_standard_approval_threshold`
- `set_governance_approval_threshold`
- `set_constitutional_approval_threshold`

Each setter validates threshold fractions (`denominator != 0`,
`numerator <= denominator`) and emits an event.

## Consequences

**Positive**

- Proposal-governance parameters are now first-class on-chain state.
- Defaults keep current governance behavior intact at genesis.
- The runtime no longer binds proposals to a compile-time `VotingPeriod`
  associated type.

**Negative / accepted trade-offs**

- Wave 1 uses `EnsureRoot` for setters, so a developer can still modify these
  parameters directly.
- This is temporary; Wave 2 replaces root authority with governance-controlled
  origin and action routing.

## Follow-up

Wave 2 will upgrade setter authorization to governance execution flow
(`GovernanceOrigin`) so parameter changes are controlled by approved proposals
instead of root-only intervention.

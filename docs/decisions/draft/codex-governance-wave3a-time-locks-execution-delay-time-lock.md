# ADR DRAFT — Mandatory execution delay after proposal approval

## Context

Wave 2 introduced `ExecutionDelay` as an on-chain parameter and added
`approved_at` to proposal storage, but execution still happened immediately
after approval. This left no mandatory review window between approval and
state-changing execution.

## Decision

Wave 3A enforces a time-lock in `execute_proposal`:

- `tally_proposal` writes `approved_at = Some(current_block)` when a proposal
  transitions to `Approved`.
- `execute_proposal` now requires:
  - proposal status is `Approved`
  - `approved_at` is present (`ProposalNotYetApproved` if missing)
  - current block is at or past `approved_at + ExecutionDelay`
    (`ExecutionTooEarly` otherwise)
- Default `ExecutionDelay = 0` preserves prior behavior at genesis.

## Consequences

**Positive**

- Adds a mandatory governance reaction window once delay is non-zero.
- The delay remains governable via `SetExecutionDelay` action.
- Keeps backward compatibility with existing deployments at default settings.

**Negative / trade-offs**

- If governance leaves delay at `0`, the protection is effectively disabled.
- Operationally, users must wait extra blocks before execution when delay is
  configured.

## Follow-up

Wave 3B will route approval threshold by proposal class so approval rigor and
execution delay compose into the complete governance-hardening model.

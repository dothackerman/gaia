## What changed
- Added membership governance parameter storage:
  `MembershipVotingPeriod`, membership threshold numerator/denominator, and suspension threshold numerator/denominator.
- Initialized those storages at genesis with defaults preserving current behavior
  (`100_800`/`20`, `4/5`, `1/1`).
- Replaced hardcoded membership threshold math with storage-backed threshold
  formula.
- Replaced hardcoded suspension unanimity check with configurable threshold
  formula while preserving ADR-005 default semantics.
- Added root-gated setters:
  `set_membership_voting_period`,
  `set_membership_approval_threshold`,
  `set_suspension_threshold`.
- Added setter events and `InvalidThreshold` error.
- Updated membership unit tests and mock genesis to cover new setters and
  threshold behavior.
- Updated integration helper `advance_past_membership_voting_period()` to read
  `MembershipVotingPeriod` storage.
- Added `gaia_proposals` dependency and implemented
  `gaia_proposals::MembershipGovernance` for membership pallet.
- Added ADR draft:
  `docs/decisions/draft/codex-governance-wave1b-membership-parameter-storage.md`.

## Build state
- `cargo check`: **blocked on cross-branch dependency**
- `cargo clippy`: not run due `cargo check` blocker
- `cargo test`: not run due `cargo check` blocker
- `cargo build`: not run due `cargo check` blocker

## Open issues
- Branch depends on Worker A trait introduction in `gaia-proposals`
  (`MembershipGovernance`) to compile. Standalone branch fails until merged with
  `codex/governance-wave1a-proposal-params`.

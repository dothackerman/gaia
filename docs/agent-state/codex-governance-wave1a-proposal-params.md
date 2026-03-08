## What changed
- Added on-chain proposal-governance parameter storage in `gaia-proposals`:
  `ProposalVotingPeriod`, `ExecutionDelay`, and three threshold numerator/denominator pairs.
- Added proposals pallet genesis config/build to initialize those values with
  current-behavior defaults (`100_800`/`20`, `0`, `1/2`, `4/5`, `9/10`).
- Removed proposals `Config::VotingPeriod` associated type and switched voting
  window reads to `ProposalVotingPeriod` storage.
- Added root-gated setter dispatchables with threshold validation and emitted
  events for each setter.
- Added a `MembershipGovernance` trait in proposals for Wave 2 wiring.
- Updated proposals mock genesis and unit tests for new storage-backed config.
- Updated integration helper `advance_past_voting_period()` to read
  `ProposalVotingPeriod` storage.
- Added ADR draft:
  `docs/decisions/draft/codex-governance-wave1a-proposal-params-governance-parameter-storage.md`.

## Build state
- `cargo check`: pass
- `cargo clippy -p gaia-proposals -- -D warnings`: pass
- `cargo test --workspace`: pass
- `cargo build --workspace`: pass

## Open issues
- `runtime` `fast-local` feature currently lives at runtime level; downstream
  pallet feature propagation is not adjusted in this branch.

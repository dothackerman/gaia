# Current build state

Last updated: 2026-03-02 (membership governance + CLI UX upgrade).

## Node (`node/`)

- Status: **imported** (solochain template node)

## Runtime (`runtime/`)

- Status: **GAIA pallets wired**
- Pallets wired: `template`, `membership`, `treasury`, `proposals`
- Runtime `spec_version`: **101** (bumped for membership interface/storage changes)
- Development preset endows tester personas (`Alice`, `Bob`, `Charlie`, `Dave`, `Eve`, `Ferdie`)
- Voting periods:
  - normal: `100_800` blocks (~7 days)
  - `fast-local`: `20` blocks

## Pallet: membership (`pallets/membership/`)

- Status: **implemented** (`gaia-membership`)
- Membership governance model is now **ID-based, time-bounded proposals**
- Storage:
  - `Members`, `ActiveMemberCount`
  - `MembershipProposalCount`, `MembershipProposals`
  - `ActiveProposalByCandidate`
  - `MembershipProposalVotes`, `MembershipProposalYesCount`, `MembershipProposalNoCount`
  - `SuspensionVotes`, `SuspensionApprovalCount`
- Membership proposal status: `Active`, `Approved`, `Rejected`
- Dispatchables:
  - `propose_member(candidate, name)`
  - `vote_on_candidate(proposal_id, approve)`
  - `finalize_proposal(proposal_id)`
  - `suspend_self()`
  - `vote_suspend_member(target, approve)`
- Behavior:
  - One active proposal per candidate
  - Approval threshold is 80% of a submit-time active-member snapshot
  - Early approval if threshold is met before deadline
  - Active-member-gated finalize after deadline rejects if threshold not met
- Tests: 30 unit tests passing

## Pallet: treasury (`pallets/treasury/`)

- Status: **implemented** (`gaia-treasury`)
- Runtime integration: wired
- Storage: `TreasuryBalance`
- Dispatchables: `deposit_fee`, `disburse`
- Events: `FeeDeposited`, `Disbursed`
- Trait implementation: `TreasuryHandler<AccountId, Balance>` for proposals
- Account model: PalletId-derived sovereign account backed by `pallet_balances`
- Tests: 10 unit tests passing

## Pallet: proposals (`pallets/proposals/`)

- Status: **implemented** (`gaia-proposals`)
- Runtime integration: wired
- Interface ownership: defines cross-pallet traits `MembershipChecker` and `TreasuryHandler`
- Storage: `ProposalCount`, `Proposals`, `ProposalVotes`, `ProposalYesCount`, `ProposalNoCount`
- Dispatchables: `submit_proposal`, `vote_on_proposal`, `tally_proposal`, `execute_proposal`
- Lifecycle: `Active -> Approved/Rejected -> Executed`
- Voting semantics: simple majority at finalize (`yes > no`) after voting window
- Invariants enforced:
  - I-2: active-member check on every vote
  - I-3: single execution guard
- Tests: 17 unit tests passing

## Integration tests (`tests/`)

- Status: **comprehensive** (`gaia-integration-tests`)
- Module counts:
  - `membership.rs`: 20 tests
  - `proposals.rs`: 19 tests
  - `treasury.rs`: 9 tests
  - `cross_pallet.rs`: 10 tests
- Total integration tests: 58

## Tester CLI (`tester-cli/`)

- Status: **implemented** (`gaia-tester-cli`)
- API mode: typed Subxt bindings using committed metadata (`tester-cli/artifacts/gaia.scale`)
- Tests: 6 parser tests passing
- Command namespaces:
  - `personas`
  - `memberships`
  - `proposals`
  - `treasury`
  - `watch`
  - `local`
- Contract changes:
  - Membership voting targets `proposal_id` (not candidate account)
  - Membership finalize command: `memberships finalize`
  - Treasury proposal finalize command at CLI level: `proposals finalize` (runtime call is `tally_proposal`)
- Watch UX:
  - `watch proposals [id]`
  - `watch memberships [id]`
  - List defaults: `--state active --order newest`
  - State/order filters supported for lists
  - Pager behavior:
    - TTY: auto pager
    - non-TTY: raw output
    - uses `$PAGER`, fallback `less -FR`
    - explicit overrides: `--pager`, `--no-pager`

## Governance hardening status

- Implemented thresholds are intentionally simple:
  - treasury proposals: `yes > no` (no quorum yet)
  - membership proposals: 80% snapshot threshold (no turnout requirement yet)
  - suspension: unanimity of all other active members
- This is flagged for future hardening work (quorum/turnout policy and threshold maturity).

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass |
| `cargo clippy` | pass (existing node-template warnings remain) |
| `cargo test` | pass (134 tests total) |
| `cargo build` | pass |

## Upstream warnings

- Node-template clippy warning family remains in `node/` (`clippy::result_large_err`).
- `polkadot-overseer`: cycle-detection informational output during build.
- `trie-db v0.30.0`: future-incompatibility warning from upstream dependency.

## Latest branch changes

- Refactored membership governance to proposal IDs with deadline-based finalization.
- Added membership voting period runtime constant aligned with proposal defaults.
- Migrated membership and integration tests to ID-based membership workflow.
- Refactored CLI namespaces to plural nouns (`personas`, `memberships`, `proposals`).
- Renamed treasury proposal CLI verb from `tally` to `finalize`.
- Added watch list/detail UX for proposals and memberships with state/order filters.
- Added pager integration for long watch list output (`$PAGER`, fallback `less -FR`).
- Refreshed `tester-cli/artifacts/gaia.scale` to match runtime interface changes.

# Current build state

Last updated: 2026-03-08 (Wave 1 governance parameter storage integrated).

## Node (`node/`)

- Status: **imported** (solochain template node)

## Runtime (`runtime/`)

- Status: **GAIA pallets wired**
- Pallets wired: `template`, `membership`, `treasury`, `proposals`
- Runtime `spec_version`: **102** (Wave 1 storage migration backfill hardening)
- Development preset endows tester personas (`Alice`, `Bob`, `Charlie`, `Dave`, `Eve`, `Ferdie`)
- Governance parameter model:
  - proposal + membership voting periods now read from on-chain storage
  - runtime `fast-local` forwards to `gaia-proposals/fast-local` and `gaia-membership/fast-local`
  - genesis defaults still: normal `100_800` blocks (~7 days), `fast-local` `20` blocks

## Pallet: membership (`pallets/membership/`)

- Status: **implemented** (`gaia-membership`)
- Membership governance model is now **ID-based, time-bounded proposals**
- Storage:
  - `Members`, `ActiveMemberCount`
  - `MembershipProposalCount`, `MembershipProposals`
  - `ActiveProposalByCandidate`
  - `MembershipProposalVotes`, `MembershipProposalYesCount`, `MembershipProposalNoCount`
  - `SuspensionVotes`, `SuspensionApprovalCount`
  - `MembershipVotingPeriod`
  - `MembershipApprovalNumerator`, `MembershipApprovalDenominator`
  - `SuspensionNumerator`, `SuspensionDenominator`
- Membership proposal status: `Active`, `Approved`, `Rejected`
- Dispatchables:
  - `propose_member(candidate, name)`
  - `vote_on_candidate(proposal_id, approve)`
  - `finalize_proposal(proposal_id)`
  - `suspend_self()`
  - `vote_suspend_member(target, approve)`
  - `set_membership_voting_period(blocks)` (root placeholder)
  - `set_membership_approval_threshold(numerator, denominator)` (root placeholder)
  - `set_suspension_threshold(numerator, denominator)` (root placeholder)
- Behavior:
  - One active proposal per candidate
  - Approval threshold formula is storage-backed: `yes * d >= snapshot * n` (genesis default `4/5`)
  - Early approval if threshold is met before deadline
  - Active-member-gated finalize after deadline rejects if threshold not met
  - Suspension threshold formula is storage-backed: `approvals * d >= others * n` (genesis default `1/1`, preserving ADR-005 unanimity)
- Trait implementation: `MembershipGovernance` for `gaia-proposals` (Wave 2 wiring contract)
- Tests: 38 unit tests passing

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
- Interface ownership: defines cross-pallet traits `MembershipChecker`, `TreasuryHandler`, and `MembershipGovernance`
- Storage:
  - `ProposalCount`, `Proposals`, `ProposalVotes`, `ProposalYesCount`, `ProposalNoCount`
  - `ProposalVotingPeriod`, `ExecutionDelay`
  - `StandardApprovalNumerator`, `StandardApprovalDenominator`
  - `GovernanceApprovalNumerator`, `GovernanceApprovalDenominator`
  - `ConstitutionalApprovalNumerator`, `ConstitutionalApprovalDenominator`
- Dispatchables:
  - `submit_proposal`, `vote_on_proposal`, `tally_proposal`, `execute_proposal`
  - `set_proposal_voting_period(blocks)` (root placeholder)
  - `set_execution_delay(blocks)` (root placeholder)
  - `set_standard_approval_threshold(numerator, denominator)` (root placeholder)
  - `set_governance_approval_threshold(numerator, denominator)` (root placeholder)
  - `set_constitutional_approval_threshold(numerator, denominator)` (root placeholder)
- Lifecycle: `Active -> Approved/Rejected -> Executed`
- Voting semantics:
  - `ProposalVotingPeriod` is active in submit/vote window logic (`vote_end` derives from storage)
  - tallying remains `yes > no` in Wave 1
  - `ExecutionDelay` and class threshold parameters are stored and settable in Wave 1, with enforcement deferred to later waves
- Invariants enforced:
  - I-2: active-member check on every vote
  - I-3: single execution guard
- Tests: 23 unit tests passing

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
  - treasury proposals: currently tallied as `yes > no` in Wave 1
  - proposal voting period is storage-backed and active at submission time
  - membership proposals: storage-backed default snapshot threshold `4/5` (80%)
  - suspension: storage-backed default `1/1` (unanimity of all other active members)
  - governance/constitutional proposal thresholds and execution delay are stored with defaults (`4/5`, `9/10`, `0`) but not enforced yet
- Setter authority is temporarily `EnsureRoot` in Wave 1 and planned to move to governance origin in Wave 2.
- Runtime-upgrade migrations now backfill missing governance parameter storage keys for
  proposals + membership to preserve safe defaults on in-place upgrades.
- This is flagged for future hardening work (quorum/turnout policy and threshold maturity).

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass |
| `cargo clippy` | pass (existing node-template warnings remain) |
| `cargo test` | pass (148 tests total) |
| `cargo build` | pass |

## Upstream warnings

- Node-template clippy warning family remains in `node/` (`clippy::result_large_err`).
- `polkadot-overseer`: cycle-detection informational output during build.
- `trie-db v0.30.0`: future-incompatibility warning from upstream dependency.

## Latest branch changes

- Wave 1A integrated: proposal governance parameters moved from compile-time config to on-chain storage with genesis defaults.
- Wave 1B integrated: membership governance parameters (including suspension threshold) moved from hardcoded logic/constants to on-chain storage.
- Added root-gated parameter setter dispatchables in proposals and membership as Wave 1 placeholders.
- Added per-pallet `on_runtime_upgrade` backfill migrations for governance parameter storage keys.
- Bumped runtime `spec_version` to `102` for Wave 1 stabilization.
- Wired runtime `fast-local` to proposals + membership pallet `fast-local` defaults.
- Updated runtime configs to remove compile-time `VotingPeriod` associated type bindings for proposals and membership.
- Updated proposal + membership mock genesis config and unit tests for storage-backed parameter behavior.
- Updated integration test helpers to read voting periods from pallet storage.
- Promoted ADR drafts to:
  - `ADR 009 — On-chain storage for proposal governance parameters`
  - `ADR 010 — On-chain storage for membership governance parameters`

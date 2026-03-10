# Current build state

Last updated: 2026-03-10 (tester CLI refreshed against current runtime metadata; docs/build state reconciled).

## Node (`node/`)

- Status: **imported** (solochain template node)

## Runtime (`runtime/`)

- Status: **GAIA pallets wired**
- Pallets wired: `template`, `membership`, `treasury`, `proposals`
- Runtime `spec_version`: **104** (Wave 4 runtime-upgrade governance)
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
  - `set_membership_voting_period(blocks)` (governance-origin gated)
  - `set_membership_approval_threshold(numerator, denominator)` (governance-origin gated)
  - `set_suspension_threshold(numerator, denominator)` (governance-origin gated)
- Behavior:
  - One active proposal per candidate
  - Approval threshold formula is storage-backed: `yes * d >= snapshot * n` (genesis default `4/5`)
  - Early approval if threshold is met before deadline
  - Active-member-gated finalize after deadline rejects if threshold not met
  - Suspension threshold formula is storage-backed: `approvals * d >= others * n` (genesis default `1/1`, preserving ADR-005 unanimity)
- Trait implementation: `MembershipGovernance` for `gaia-proposals` (Wave 2 wiring contract)
- Tests: 41 unit tests passing

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
  - `PendingRuntimeCode`
- Dispatchables:
  - `submit_proposal(class, action)`, `vote_on_proposal`, `tally_proposal`, `execute_proposal`
  - `set_proposal_voting_period(blocks)` (governance-origin gated)
  - `set_execution_delay(blocks)` (governance-origin gated)
  - `set_standard_approval_threshold(numerator, denominator)` (governance-origin gated)
  - `set_governance_approval_threshold(numerator, denominator)` (governance-origin gated)
  - `set_constitutional_approval_threshold(numerator, denominator)` (governance-origin gated)
  - `upload_runtime_code(code)`
- Lifecycle: `Active -> Approved/Rejected -> Executed`
- Voting semantics:
  - `ProposalVotingPeriod` is active in submit/vote window logic (`vote_end` derives from storage)
  - `tally_proposal` routes approval threshold by class (`Standard`, `Governance`, `Constitutional`)
  - `ExecutionDelay` is enforced before execution using `approved_at`
  - `UpgradeRuntime` constitutional actions execute `frame_system::set_code` after code-hash verification
- Invariants enforced:
  - I-2: active-member check on every vote
  - I-3: single execution guard
- Tests: 37 unit tests passing

## Integration tests (`tests/`)

- Status: **comprehensive** (`gaia-integration-tests`)
- Module counts:
  - `membership.rs`: 20 tests
  - `proposals.rs`: 22 tests
  - `treasury.rs`: 9 tests
  - `cross_pallet.rs`: 10 tests
- Total integration tests: 61

## Tester CLI (`tester-cli/`)

- Status: **implemented** (`gaia-tester-cli`)
- API mode: typed Subxt bindings using refreshed committed metadata (`tester-cli/artifacts/gaia.scale`)
- Tests: 8 parser tests passing
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
  - Proposal submission now uses `proposals submit <typed-action-subcommand> ...`
  - Runtime-code upload is explicit: `proposals upload-runtime-code <signer> <code_path>`
  - Proposal finalize command at CLI level: `proposals finalize` (runtime call is `tally_proposal`)
- Watch UX:
  - `watch proposals [id]`
  - `watch memberships [id]`
  - List defaults: `--state active --order newest`
  - State/order filters supported for lists
  - Proposal list/detail output now describes `class`, `action`, `submitted_at`, `vote_end`, and `approved_at`
  - Pager behavior:
    - TTY: auto pager
    - non-TTY: raw output
    - uses `$PAGER`, fallback `less -FR`
    - explicit overrides: `--pager`, `--no-pager`
  - Typed proposal subcommands:
    - `disbursement`
    - `set-proposal-voting-period`
    - `set-execution-delay`
    - `set-membership-voting-period`
    - `set-standard-threshold`
    - `set-governance-threshold`
    - `set-constitutional-threshold`
    - `set-membership-threshold`
    - `set-suspension-threshold`
    - `upgrade-runtime`

## Governance hardening status

- Wave 2 complete:
  - typed `GovernanceAction` proposal payload
  - class-action validation at submission
  - governance-origin setter authority (`ga/govn0` sovereign account)
- Wave 3 complete:
  - execution delay enforced (`ExecutionDelay`, `approved_at`, `ExecutionTooEarly`)
  - class-based tally thresholds (`Standard`, `Governance`, `Constitutional`)
- Wave 4 complete:
  - constitutional `UpgradeRuntime` action
  - hash-bound uploaded code (`upload_runtime_code`, `PendingRuntimeCode`)
  - runtime code applied via `set_code` and pending code cleared on success
- Remaining hardening backlog: turnout/quorum policy and richer runtime-code queueing UX.

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test` | pass (170 tests total) |
| `cargo build` | pass |

## Upstream warnings

- `gaia-runtime`: build warns that stable Rust now supports `wasm32v1-none`; the workspace still builds Wasm with `wasm32-unknown-unknown`.
- `polkadot-overseer`: cycle-detection informational output during build.
- `trie-db v0.30.0`: future-incompatibility warning from upstream dependency.

## Latest branch changes

- Wave 2 integrated:
  - generalized execution with typed `GovernanceAction`
  - governance-origin gating for proposals + membership setters
  - runtime version bump to `spec_version 103`
  - `ADR 011 — Generalized proposal execution with GovernanceAction enum`
- Wave 3 integrated:
  - mandatory execution delay enforcement
  - class-routed threshold tallying
  - promoted ADRs:
    - `ADR 012 — Execution delay time-lock`
    - `ADR 013 — Proposal class system`
- Wave 4 integrated:
  - constitutional runtime-upgrade governance flow
  - runtime code upload + hash verification + `set_code` execution path
  - runtime version bump to `spec_version 104`
  - `ADR 014 — Runtime upgrade governance`

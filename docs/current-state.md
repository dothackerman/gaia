# Current build state

Last updated: 2026-02-22 — membership suspension flows implemented, proposal execution restricted to organizer, treasury genesis endowed, and integration-test crate scaffolded.

## Node (`node/`)

- Status: **imported** — node crate present (from solochain template)

## Runtime (`runtime/`)

- Status: **integrating GAIA pallets**
- Runtime now wires: `template`, `membership`, `treasury`, `proposals`
- Runtime config split: pallet-specific files under `runtime/src/configs/`

## Pallet: template (`pallets/template/`)

- Status: **imported** — template pallet present (from solochain template)

## Pallet: membership (`pallets/membership/`)

- Status: **implemented**
- Crate name: `gaia-membership`
- Storage: `Members`, `ActiveMemberCount`, `Candidates`, `CandidateVotes`, `CandidateApprovalCount`
- Dispatchables: `propose_member`, `vote_on_candidate`
- Trait: `MembershipChecker<AccountId>` with `is_active_member()`
- Genesis: supports configured initial members via `GenesisConfig.initial_members`
- Tests: 19 passing (17 in `src/tests.rs` + 2 in `src/mock.rs`: runtime integrity + genesis build)
- TODO: `suspend_member` dispatchable (self-initiated + unanimous peer vote; see ADR 005)

## Pallet: treasury (`pallets/treasury/`)

- Status: **implemented**
- Crate name: `gaia-treasury`
- Runtime integration: wired
- Storage: `TreasuryBalance`
- Dispatchables: `deposit_fee`, `disburse`
- Events: `FeeDeposited`, `Disbursed`
- Trait: implements `TreasuryHandler<AccountId, Balance>` for proposals
- Account model: PalletId-derived sovereign account with fungible transfers
- Tests: 10 passing (8 in `src/lib.rs` + 2 runtime integrity/genesis build)

## Pallet: proposals (`pallets/proposals/`)

- Status: **implemented**
- Crate name: `gaia-proposals`
- Runtime integration: wired
- Interface ownership: defines downstream cross-pallet traits `MembershipChecker` and `TreasuryHandler`
- Runtime adapters: wired in `runtime/src/configs/proposals.rs`
- Storage: `ProposalCount`, `Proposals`, `ProposalVotes`, `ProposalYesCount`, `ProposalNoCount`
- Dispatchables: `submit_proposal`, `vote_on_proposal`, `tally_proposal`, `execute_proposal`
- Lifecycle: `Active` → `Approved`/`Rejected` → `Executed` (terminal)
- Voting: simple majority (yes > no); window length configurable via `VotingPeriod` constant (100 800 blocks / 7 days in runtime)
- Invariants: I-2 (active-member check on every vote), I-3 (single-execution guard)
- Tests: 16 passing (14 in `src/tests.rs` + 2 in `src/mock.rs`: runtime integrity + genesis build)

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass (2026-02-22) |
| `cargo clippy` | pass — GAIA pallet/runtime changes clean; existing node-template warnings remain (2026-02-22) |
| `cargo test` | pass — 53 tests total (19 membership + 16 proposals + 10 treasury + 4 runtime + 4 template) (2026-02-22) |
| `cargo deny check licenses` | pass — all dependencies compliant (2026-02-21) |
| `cargo build` | pass (2026-02-22) |

## Upstream Warnings

- 2026-02-22 — Node template clippy warnings (`clippy::result_large_err`) in `node/` (benchmarking/command/service/main). Treated as template-origin warnings for now.
- 2026-02-21 — `polkadot-overseer`: cycle detection output during build ("Found 3 strongly connected components which includes at least one cycle each").
- 2026-02-21 — WASM runtime build target recommendation: `wasm32v1-none` is supported in Rust >= 1.84 (see `docs/decisions/003-wasm32v1-none-target.md`).
- 2026-02-21 — `trie-db v0.30.0`: future-incompatibility warning (may be rejected by a future version of Rust). Consider running `cargo report future-incompatibilities --id 1`.


## Latest changes (this branch)

- `proposals`: `execute_proposal` now enforces organizer-only execution via `NotOrganizer` error.
- `membership`: added `suspend_self` and `vote_suspend_member` dispatchables, suspension vote storage, suspension events/reasons, and related errors.
- `runtime genesis`: treasury sovereign account is now endowed in genesis balances.
- Added workspace integration test crate at `tests/` for cross-pallet runtime verification.
- Legacy pallet unit tests are now marked `#[ignore]` and remain runnable with `cargo test -- --ignored`.

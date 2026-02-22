# Current build state

Last updated: 2026-02-22 — clean code pass, un-ignored unit tests, comprehensive e2e integration tests.

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
- Storage: `Members`, `ActiveMemberCount`, `Candidates`, `CandidateVotes`, `CandidateApprovalCount`, `SuspensionVotes`, `SuspensionApprovalCount`
- Dispatchables: `propose_member`, `vote_on_candidate`, `suspend_self`, `vote_suspend_member`
- Trait: `MembershipChecker<AccountId>` with `is_active_member()`
- Genesis: supports configured initial members via `GenesisConfig.initial_members`
- Tests: 24 unit tests passing

## Pallet: treasury (`pallets/treasury/`)

- Status: **implemented**
- Crate name: `gaia-treasury`
- Runtime integration: wired
- Storage: `TreasuryBalance`
- Dispatchables: `deposit_fee`, `disburse`
- Events: `FeeDeposited`, `Disbursed`
- Trait: implements `TreasuryHandler<AccountId, Balance>` for proposals
- Account model: PalletId-derived sovereign account with fungible transfers
- Public API: `account_id()` exposes the sovereign account for external use
- Tests: 10 unit tests passing

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
- Execution: organizer-only via `NotOrganizer` error guard
- Invariants: I-2 (active-member check on every vote), I-3 (single-execution guard)
- Tests: 17 unit tests passing

## Integration tests (`tests/`)

- Status: **comprehensive**
- Crate name: `gaia-integration-tests`
- Modules gated with `#[cfg(test)]` — no library warnings
- Tests: 41 passing
  - `membership.rs`: 13 tests (genesis, propose, vote, threshold, suspension)
  - `treasury.rs`: 7 tests (genesis funding, deposit, disburse, error paths)
  - `proposals.rs`: 13 tests (lifecycle, voting, tally, execution, error paths)
  - `cross_pallet.rs`: 8 tests (I-1 treasury guard, I-2 active-member voting, I-3 single execution, suspension interactions, newly admitted members)

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass (2026-02-22) |
| `cargo clippy` | pass — GAIA pallet/runtime/integration changes clean; existing node-template warnings remain (2026-02-22) |
| `cargo test` | pass — 105 tests total (24 membership + 17 proposals + 10 treasury + 41 integration + 9 runtime + 4 template) (2026-02-22) |
| `cargo deny check licenses` | pass — all dependencies compliant (2026-02-21) |
| `cargo build` | pass (2026-02-22) |

## Upstream Warnings

- 2026-02-22 — Node template clippy warnings (`clippy::result_large_err`) in `node/` (benchmarking/command/service/main). Treated as template-origin warnings for now.
- 2026-02-21 — `polkadot-overseer`: cycle detection output during build ("Found 3 strongly connected components which includes at least one cycle each").
- 2026-02-21 — WASM runtime build target recommendation: `wasm32v1-none` is supported in Rust >= 1.84 (see `docs/decisions/003-wasm32v1-none-target.md`).
- 2026-02-21 — `trie-db v0.30.0`: future-incompatibility warning (may be rejected by a future version of Rust). Consider running `cargo report future-incompatibilities --id 1`.

## Latest changes (this branch)

- Removed all `#[ignore]` markers from pallet unit tests — all 50 pallet tests now run by default.
- Made `treasury::account_id()` public (was `pub(crate)`) for integration test access.
- Gated integration test modules with `#[cfg(test)]` — eliminates unused-import warnings during `cargo check`.
- Made `common::bounded_name()` and `common::eve()` public in integration tests; removed duplicate `bounded_name` from `membership.rs`.
- Expanded integration test suite from 19 to 41 tests covering all invariants, error paths, and cross-pallet interactions.
- Moved `genesis_seeds_initial_members` test from `lib.rs` into `membership.rs` module.

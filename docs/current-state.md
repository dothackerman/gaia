# Current build state

Last updated: 2026-02-24 — completed tester CLI local-member UX baseline.

## Node (`node/`)

- Status: **imported** — node crate present (from solochain template)

## Runtime (`runtime/`)

- Status: **integrating GAIA pallets**
- Runtime now wires: `template`, `membership`, `treasury`, `proposals`
- Runtime config split: pallet-specific files under `runtime/src/configs/`
- Development preset now endows tester personas (`Alice`, `Bob`, `Charlie`, `Dave`, `Eve`, `Ferdie`) for deterministic local CLI fee payment.

## Pallet: template (`pallets/template/`)

- Status: **imported** — template pallet present (from solochain template)

## Pallet: membership (`pallets/membership/`)

- Status: **implemented**
- Crate name: `gaia-membership`
- Storage: `Members`, `ActiveMemberCount`, `Candidates`, `CandidateVotes`, `CandidateApprovalCount`, `SuspensionVotes`, `SuspensionApprovalCount`
- Dispatchables: `propose_member`, `vote_on_candidate`, `suspend_self`, `vote_suspend_member`
- Trait: `MembershipChecker<AccountId>` with `is_active_member()`
- Genesis: supports configured initial members via `GenesisConfig.initial_members`
- Tests: 25 unit tests passing

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
- Tests: 54 passing
  - `membership.rs`: 15 tests (genesis, propose, vote, threshold, suspension, single-member genesis, 5-member threshold boundary)
  - `treasury.rs`: 9 tests (genesis funding, deposit, disburse, error paths, non-member deposit, multiple deposit accumulation)
  - `proposals.rs`: 19 tests (lifecycle, voting, tally, execution, error paths, concurrent proposals, zero-amount proposal, majority boundary, non-member tally, vote storage persistence)
  - `cross_pallet.rs`: 11 tests (I-1 treasury guard, I-2 active-member voting, I-3 single execution, suspension interactions, newly admitted members, suspended organizer execution, tally after all voters suspended)


## Tester CLI (`tester-cli/`)

- Status: **implemented** (local tester UX baseline)
- Crate name: `gaia-tester-cli`
- Scope: human local tester workflows (persona, membership, proposal, treasury, watch, local helpers)
- API mode: typed Subxt bindings using committed metadata artifact (`tester-cli/artifacts/gaia.scale`)
- Metadata artifact: refreshed from local node RPC (non-empty, committed SCALE bytes)
- UX output: finalized extrinsic hash + typed pallet event summaries for membership/proposal/treasury actions
- Error mapping: runtime dispatch failures surfaced as `Pallet::Error` labels when available
- Vote CLI contract: explicit `yes|no` values (no positional bool ambiguity)
- Local mode: runtime `fast-local` feature shortens voting period for practical manual lifecycle testing

## Build status

| Command | Status |
|---|---|
| `cargo check` | pass — GAIA workspace clean; known upstream warnings only (2026-02-24) |
| `cargo clippy` | pass — GAIA pallet/runtime/integration changes clean; existing node-template warnings remain (2026-02-24) |
| `cargo test` | pass — 124 tests total (25 membership + 17 proposals + 10 treasury + 54 integration + 9 runtime + 4 template + 5 tester-cli) (2026-02-24) |
| `cargo deny check licenses` | pass — all dependencies compliant (2026-02-21) |
| `cargo build` | pass (2026-02-24) |

## Upstream Warnings

- 2026-02-24 — Node template clippy warnings (`clippy::result_large_err`) in `node/` (benchmarking/command/service/main). Treated as template-origin warnings for now.
- 2026-02-24 — `polkadot-overseer`: cycle detection output during build ("Found 3 strongly connected components which includes at least one cycle each").
- 2026-02-24 — WASM runtime build target recommendation: `wasm32v1-none` is supported in Rust >= 1.84 (see `docs/decisions/003-wasm32v1-none-target.md`).
- 2026-02-24 — `trie-db v0.30.0`: future-incompatibility warning (may be rejected by a future version of Rust). Consider running `cargo report future-incompatibilities --id 1`.

## Latest changes (this branch)

- Updated `development_config_genesis` endowments to fund all seeded tester personas used by `gaia-tester-cli`.
- Regenerated `tester-cli/artifacts/gaia.scale` from a running local node RPC (`state_getMetadata`) and restored typed Subxt codegen.
- Aligned tester CLI extrinsics with current runtime metadata:
  - `membership propose` now submits both candidate account and bounded name.
  - `proposal submit` now uses bounded title/description arguments.
- Added shared transaction helper in `tester-cli/src/api.rs`:
  - waits for finalized success,
  - decodes runtime dispatch errors into readable `Pallet::Error` labels,
  - standardizes event/result reporting.
- Improved command feedback for human testers:
  - membership/proposal/treasury commands now print typed event summaries and finalized extrinsic hashes.
  - `watch proposal` now includes yes/no vote counts and `vote_end`.
- Simplified local helper command names to `start`, `reset`, `refresh-metadata`.
- Added tester CLI parser coverage to 5 unit tests, including membership vote (`yes|no`) and local refresh metadata command parsing.
- Removed all `#[ignore]` markers from pallet unit tests — all 50 pallet tests now run by default.
- Made `treasury::account_id()` public (was `pub(crate)`) for integration test access.
- Gated integration test modules with `#[cfg(test)]` — eliminates unused-import warnings during `cargo check`.
- Made `common::bounded_name()` and `common::eve()` public in integration tests; removed duplicate `bounded_name` from `membership.rs`.
- Expanded integration test suite from 19 to 41 tests covering all invariants, error paths, and cross-pallet interactions.
- Moved `genesis_seeds_initial_members` test from `lib.rs` into `membership.rs` module.
- Expanded integration test suite from 41 to 54 tests with edge-case coverage:
  - Added `ferdie()` helper and `new_test_ext_with_members()` custom genesis builder.
  - Proposals: concurrent proposals, treasury contention, zero-amount proposal, exact majority boundary, non-member tally, vote storage persistence.
  - Treasury: non-member deposit, multiple deposit accumulation.
  - Membership: single-member genesis admission, 5-member threshold boundary.
  - Cross-pallet: suspended organizer execution, tally after all voters suspended, newly admitted member + proposer suspension interaction.

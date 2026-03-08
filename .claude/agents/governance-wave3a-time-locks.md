# Agent: Governance Wave 3A — Execution Delay (Time-Locks)

## Session start

Before any action:
1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.
3. Read `docs/plans/governance-on-chain.md` for full milestone context.
4. Read `docs/decisions/010-generalized-proposal-execution.md` (Wave 2).

Do not write any code until all four are loaded.

---

## Prerequisite

**Wave 2 PR must be merged into `main` before this agent starts.**
Run `git pull origin main` first.

This agent runs **in parallel with Wave 3B** (`governance-wave3b-proposal-classes.md`).

**File ownership:**
- This agent (3A) modifies `execute_proposal` and `tally_proposal` in
  `pallets/proposals/src/lib.rs`, and proposal integration tests.
- Agent 3B modifies `tally_proposal` approval logic only.
- **Coordination:** If both agents touch `tally_proposal`, 3A adds the
  `approved_at` write; 3B adds the threshold routing. They must not conflict.
  3A should only add `approved_at = Some(now)` inside the approval branch;
  3B should only change the `yes > no` condition. Merge will be clean.

**Branch:** create from `main` as `claude/governance-wave3a-time-locks`

---

## Goal

Enforce a mandatory delay between proposal approval and execution.
The delay is stored in `ExecutionDelay` StorageValue (added in Wave 1A, default `0`).
A delay of `0` preserves the current "execute immediately after tally" behaviour.

---

## Implementation steps

1. **In `tally_proposal`:** when transitioning to `ProposalStatus::Approved`,
   record the current block:
   ```rust
   proposal.approved_at = Some(frame_system::Pallet::<T>::block_number());
   ```
   The `approved_at` field was added to the Proposal struct in Wave 2. Confirm
   it is present before proceeding.

2. **In `execute_proposal`:** add a delay check before the action dispatch:
   ```rust
   let delay = ExecutionDelay::<T>::get();
   let approved_at = proposal.approved_at.ok_or(Error::<T>::ProposalNotYetApproved)?;
   let executable_at = approved_at.saturating_add(delay);
   let now = frame_system::Pallet::<T>::block_number();
   ensure!(now >= executable_at, Error::<T>::ExecutionTooEarly);
   ```

3. **Add new error variants:**
   - `ExecutionTooEarly` — proposal is approved but the delay has not elapsed.
   - `ProposalNotYetApproved` — execute called on a non-approved proposal
     (should not happen if status check is correct, but belt-and-suspenders).

---

## Tests to write

In `pallets/proposals/src/lib.rs` unit tests:

- `execute_proposal_with_zero_delay_succeeds_immediately()` — default delay 0;
  execute in same block as tally → succeeds.
- `execute_proposal_fails_before_delay_expires()` — set delay to 10 blocks;
  tally at block 5, execute at block 10 → `ExecutionTooEarly`.
- `execute_proposal_succeeds_after_delay_expires()` — same setup; execute at
  block 15 → succeeds.
- `execution_delay_is_governable()` — submit a `SetExecutionDelay` Governance
  proposal, vote, tally, execute; subsequent proposal respects new delay.

In `tests/proposals.rs` integration tests:
- Update any integration test that calls `execute_proposal` immediately after
  `tally_proposal` to advance blocks by the delay period if `ExecutionDelay > 0`.
  With default delay of 0, no changes needed to existing tests.

---

## ADR required

Create `docs/decisions/011-execution-delay-time-lock.md`.

Title: "Mandatory execution delay after proposal approval"

Document:
- Why a delay protects against flash governance attacks.
- Default of `0` preserves current behaviour — no breaking change.
- The delay is itself governable via `SetExecutionDelay` (Governance class).
- Accepted trade-off: delay of 0 means no protection until governance
  explicitly sets a non-zero value.

---

## Cargo sequence

Use `just all` to run the full sequence, or step through manually:

```
cargo check                                      # 1. must pass first
cargo clippy -p gaia-proposals -- -D warnings    # 2. fix ALL warnings
cargo fmt --all -- --check                       # 3. formatting clean
cargo test --workspace                           # 4. all tests pass
cargo build --workspace                          # 5. full build
```

Never skip a level. `just all` runs steps 1–5 in order.

---

## Completion

Commit: `feat(proposals): add execution delay time-lock after proposal approval`

Push to `claude/governance-wave3a-time-locks` and open a PR targeting `main`.
PR title: "Add execution delay time-lock for approved proposals"

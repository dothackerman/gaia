# Agent: Governance Wave 3B — Proposal Class Threshold Routing

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

This agent runs **in parallel with Wave 3A** (`governance-wave3a-time-locks.md`).

**File ownership:**
- Agent 3A modifies `execute_proposal` and writes `approved_at` in `tally_proposal`.
- This agent (3B) modifies the approval *condition* in `tally_proposal` only.
- Coordination: in `tally_proposal`, 3A touches the `if approved { approved_at = ... }` branch.
  This agent changes the condition `if yes > no` to the class-based threshold.
  There is no line conflict as long as 3B only replaces the condition expression
  and 3A only adds to the approval branch body.

**Branch:** create from `main` as `claude/governance-wave3b-proposal-classes`

---

## Goal

Replace the hardcoded `yes > no` approval condition in `tally_proposal` with a
per-class threshold lookup. The `ProposalClass` enum and the threshold
`StorageValues` were added in earlier waves. This agent wires them together.

---

## Implementation steps

1. **In `tally_proposal`**, replace:
   ```rust
   if yes > no {
       // approved
   }
   ```
   With:
   ```rust
   let (num, den) = match proposal.class {
       ProposalClass::Standard => (
           StandardApprovalNumerator::<T>::get(),
           StandardApprovalDenominator::<T>::get(),
       ),
       ProposalClass::Governance => (
           GovernanceApprovalNumerator::<T>::get(),
           GovernanceApprovalDenominator::<T>::get(),
       ),
       ProposalClass::Constitutional => (
           ConstitutionalApprovalNumerator::<T>::get(),
           ConstitutionalApprovalDenominator::<T>::get(),
       ),
   };
   let yes = ProposalYesCount::<T>::get(proposal_id);
   let no = ProposalNoCount::<T>::get(proposal_id);
   let total = yes.saturating_add(no);
   // yes * den >= total * num
   if yes.saturating_mul(den) >= total.saturating_mul(num) {
       // approved
   }
   ```

   Verify with defaults:
   - Standard `(1, 2)`: `yes * 2 >= (yes+no) * 1` → `2*yes >= yes+no` → `yes >= no`
     → `yes > no` when votes are integers and `yes != no` (strict majority). ✓
   - Governance `(4, 5)`: `yes * 5 >= total * 4` → 80% of total votes. ✓
   - Constitutional `(9, 10)`: `yes * 10 >= total * 9` → 90% of total votes. ✓

   Note: unlike the membership threshold (which uses a snapshot), the proposal
   threshold is calculated over actual votes cast (`yes + no`). This is the
   existing behaviour and is preserved here.

2. No other changes to other dispatchables are needed in this wave.

---

## Tests to write

In `pallets/proposals/src/lib.rs` unit tests:

- `standard_class_approves_with_strict_majority()` — 3 yes / 2 no → approved.
- `standard_class_rejects_on_tie()` — 2 yes / 2 no → rejected.
- `governance_class_requires_80_percent()` — 4 yes / 1 no → approved (80%);
  3 yes / 2 no → rejected (60% < 80%).
- `constitutional_class_requires_90_percent()` — 9 yes / 1 no → approved;
  8 yes / 2 no → rejected.
- `threshold_change_is_itself_governed_by_constitutional_class()` — a proposal
  with `GovernanceAction::SetConstitutionalApprovalThreshold` that was submitted
  as `Governance` class is rejected at submission with `ProposalClassMismatch`
  (class-action mapping enforced in `submit_proposal`, Wave 2 work — confirm
  it is already in place).

In `tests/proposals.rs`:
- Add `governance_class_proposal_end_to_end()` — full lifecycle: submit
  Governance-class `SetProposalVotingPeriod`, vote with enough members to meet
  80% threshold, tally → approved, execute → StorageValue updated.
- Add `constitutional_class_threshold_change_end_to_end()` — full lifecycle
  for a `SetConstitutionalApprovalThreshold` proposal requiring 90% approval.

---

## ADR required

Create `docs/decisions/012-proposal-class-system.md`.

Title: "Three-tier proposal class system for meta-governance"

Document:
- The recursion problem: lowering a threshold with that threshold.
- Why Constitutional threshold must be ≥ Governance threshold ≥ Standard threshold.
- Default values (50%, 80%, 90%) and their relationship.
- The class-action mapping and where it is enforced (submit time).
- Accepted trade-off: initial thresholds are set at genesis; the community must
  use a Constitutional-class proposal to change them.

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

Commit: `feat(proposals): route tally threshold by proposal class`

Push to `claude/governance-wave3b-proposal-classes` and open a PR targeting `main`.
PR title: "Add class-based threshold routing to proposal tally"

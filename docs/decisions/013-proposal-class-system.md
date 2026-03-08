# ADR 013 — Three-tier proposal class system for meta-governance

## Context

Wave 2 introduced proposal classes and class/action mapping, but tally approval
logic still required explicit routing to class thresholds so governance and
constitutional proposals can be held to stricter approval requirements.

Without class-based threshold routing, all proposals effectively share one
approval rule and meta-governance protections are weakened.

## Decision

Route tally approval by proposal class using on-chain threshold storage:

- `Standard` uses `StandardApprovalNumerator/Denominator`.
- `Governance` uses `GovernanceApprovalNumerator/Denominator`.
- `Constitutional` uses `ConstitutionalApprovalNumerator/Denominator`.

Approval predicate in tally:

- Let `yes = yes_votes`, `no = no_votes`, `total = yes + no`.
- Proposal is approved when `yes * den >= total * num`.

This preserves vote-total semantics (thresholds apply to votes cast, not member
snapshot size).

## Consequences

**Positive**

- Governance and constitutional actions are now enforced at stricter,
  configurable thresholds.
- Meta-governance recursion risk is reduced by requiring constitutional class
  for threshold-changing actions.
- Threshold values remain on-chain and governable.

**Negative / trade-offs**

- With default standard threshold `1/2` and `>=` predicate, ties pass for
  standard class unless governance raises the threshold.
- Turnout/quorum is still not enforced in this wave.

## Follow-up

Wave 4 runtime-upgrade governance actions inherit this class-threshold model.
Future hardening may add explicit quorum/turnout policies.

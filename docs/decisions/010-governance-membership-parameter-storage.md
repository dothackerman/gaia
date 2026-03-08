# ADR 010 — On-chain storage for membership governance parameters

## Context

The membership pallet previously depended on compile-time voting-period config
and embedded threshold multipliers in pallet logic (80% admission threshold and
unanimity suspension logic).

Changing those governance parameters required runtime code edits and upgrade
coordination, which blocks policy evolution through governance.

## Decision

Membership governance parameters are now stored on-chain:

- `MembershipVotingPeriod`
- `MembershipApprovalNumerator` / `MembershipApprovalDenominator`
- `SuspensionNumerator` / `SuspensionDenominator`

Genesis initialization preserves current behavior:

- membership voting period: `100_800` blocks (`20` with `fast-local`)
- membership approval threshold: `4/5` (80%)
- suspension threshold: `1/1` (unanimity of other active members)

The runtime `fast-local` feature forwards to `gaia-membership/fast-local`, so
the 20-block membership default is active in fast-local builds.

Threshold checks are now formula-based:

- membership approval:
  `yes_votes * membership_denominator >= active_snapshot * membership_numerator`
- suspension:
  `approvals * suspension_denominator >= others * suspension_numerator`

Root-gated setter dispatchables were added as a Wave 1 placeholder:

- `set_membership_voting_period`
- `set_membership_approval_threshold`
- `set_suspension_threshold`

Each threshold setter validates `denominator != 0` and
`numerator <= denominator`.

## Consequences

**Positive**

- Membership governance settings are now runtime state, not code constants.
- Admission and suspension thresholds can be evolved without editing pallet
  logic.
- ADR-005 semantics are preserved by default because `(1, 1)` keeps peer
  suspension at unanimity.
- Runtime-upgrade backfill migration initializes any missing membership
  governance keys on existing chains, preserving safe defaults.

**Negative / accepted trade-offs**

- Setter authority is still `EnsureRoot` during Wave 1.
- This allows developer-level changes until Wave 2 governance origin wiring is
  complete.

## References

- ADR-005: Unanimity requirement for peer-initiated member suspension.
- ADR-007: Membership proposal lifecycle and threshold baseline follow-up.

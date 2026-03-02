# ADR 007 — Time-bounded membership proposals with snapshot threshold baseline

## Context

Membership admission was previously keyed directly by candidate account and had
no explicit proposal record lifecycle. That model made read-only UX harder
(no stable proposal IDs) and increased ambiguity around long-lived pending
admissions.

The project also needed CLI-level consistency with treasury proposal flows:

- stable proposal identifiers for list/detail views,
- explicit active/terminal states,
- clear finalize step after deadline.

At the same time, we needed to keep implementation scope small and preserve
existing governance assumptions for local testing.

## Decision

Adopt an explicit membership proposal lifecycle with IDs and deadlines:

1. Membership proposals are first-class records with monotonic `proposal_id`.
2. Lifecycle states are `Active`, `Approved`, `Rejected`.
3. Each proposal stores `vote_end` and an `active_member_snapshot` captured at
   submit time.
4. Voting references `proposal_id` (not candidate account).
5. Approval threshold remains `>= 80%` of snapshot active-member count.
6. Early approval is allowed when threshold is reached before deadline.
7. If still active after deadline, an active member may call `finalize_proposal`:
   threshold met -> approve, otherwise reject.
8. Enforce one active membership proposal per candidate.

## Consequences

### Positive

- Membership proposals cannot remain active indefinitely; they resolve by
  threshold or deadline+finalize.
- CLI/read-only UX becomes consistent with treasury proposals through stable IDs
  and list/detail semantics.
- Snapshot-based threshold avoids mid-vote drift from changing member counts.

### Negative / accepted trade-offs

- Membership threshold policy is still a baseline model:
  - no quorum/turnout requirement,
  - no abstention weighting,
  - no time-decay or dynamic thresholds.
- Finalization is explicit (manual call), not automatic.

These trade-offs are accepted for now to keep governance understandable during
early internal testing.

## Follow-up direction

Future governance hardening should evaluate:

- quorum/turnout requirements for all voting systems,
- explicit threshold policy consistency across membership, treasury proposals,
  and suspension,
- automatic expiry/finalization mechanisms if manual finalize becomes an
  operational burden.


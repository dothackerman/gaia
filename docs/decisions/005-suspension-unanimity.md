# ADR 005 — Unanimity requirement for peer-initiated member suspension

## Context

The membership pallet defines two future paths for suspending a member:

1. **Self-initiated** — a member voluntarily suspends their own account.
2. **Peer-initiated** — other active members vote to suspend a member.

For the peer-initiated path the question is what threshold of active-member
approval should be required. The obvious choices are:

- **Simple majority (> 50 %)** — easy to reach but risks factional abuse in a
  small community.
- **Supermajority (e.g. 80 %)** — the same threshold used for admitting new
  members.
- **Unanimity (100 % of other active members)** — the most protective option.

Suspension removes a member's ability to vote and propose. In a governance
system with equal-weight voting, involuntary removal of a participant is the
most severe action the community can take short of expulsion. The consequences
are asymmetric: a wrongful suspension silences a legitimate voice, while a
delayed suspension still leaves other governance mechanisms (e.g. proposal
rejection) intact.

## Decision

Require **unanimity of all other active members** to suspend a member through
peer vote. That is, every active member except the target must cast an approval
vote before the suspension takes effect.

## Consequences

**Positive**

- Maximises protection against factional or malicious suspension attempts.
- Aligns with the project's ethos of equal-weight governance — no minority
  can unilaterally silence a member.
- Simple to reason about and implement: the threshold is exactly
  `active_member_count - 1` approvals.

**Negative / accepted trade-offs**

- A single dissenting member can block a suspension, even when the target is
  clearly acting against the community's interest. This is accepted because
  other governance levers (rejecting proposals, refusing to co-sign) remain
  available.
- As the community grows, achieving unanimity becomes exponentially harder.
  If this proves impractical at scale, the threshold can be revisited in a
  future ADR.

**Rejected alternative**

A supermajority threshold (e.g. 80 %) was considered but rejected because
suspension is a higher-stakes action than admission and warrants a stricter
bar.

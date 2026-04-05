# GAIA Roadmap

GAIA is still young enough that its architecture can be shaped deliberately.
That window does not stay open forever. Early systems usually harden in one of
two ways: either their first clean ideas become the foundation for later work,
or convenience wins a few times in a row and the whole system slowly turns into
an explanation problem. This roadmap exists to prevent the second outcome.

GAIA began as a learning prototype, but it is no longer useful to think about
it as disposable. The code already expresses a serious ambition: a private,
sovereign governance chain whose rules are explicit, whose meaning is legible,
and whose future growth can remain understandable to both developers and users.
That ambition deserves a plan that does more than list features. It needs a
story for how the architecture should mature without losing its nerve.

## Where GAIA stands today

The current system is stronger than a typical prototype because it already
contains a few important architectural decisions that were made in the right
direction. Governance is divided into distinct domains. Membership determines
who is a legitimate active participant. Treasury owns community funds and the
non-negative balance invariant. Proposals owns the lifecycle of collective
decisions: submission, voting, tallying, approval, rejection, and execution.

That separation matters. It means the project is not organized merely around
files or convenience. It is organized around meanings. Each pallet has a
primary responsibility. The runtime is used as the place where concrete wiring
happens. Cross-pallet collaboration is mediated through traits instead of
directly reaching into each other's internals. Proposal flows and membership
flows are expressed as state machines rather than as a heap of booleans. The
system also uses typed governance actions and on-chain policy parameters, which
means the code is trying to describe the governance domain explicitly instead
of hiding it behind opaque calls and social assumptions.

Those are not decorative choices. They are the reason GAIA can still be read as
an architecture rather than as a sequence of patches.

## The patterns worth preserving

The first principle of this roadmap is conservative: do not casually replace
what is already working architecturally. The pallet split is good. It gives
invariants a natural home, keeps tests focused, and allows future refactors to
happen in slices. The trait-based boundaries are good. They keep the runtime as
the composition root and reduce the temptation to entangle domain ownership.
The explicit state machines are good. Governance systems live or die by whether
their transitions are visible and enforceable. Strong typing is good. It makes
proposal classes, actions, statuses, and reasons legible in code, and later it
will make them legible in user-facing clients as well. Policy stored on-chain
is also the right move. A governance system that cannot govern its own policy
is only pretending to be one.

These patterns should not merely survive. They should become stricter as the
project grows. The temptation later will be to weaken them in the name of speed.
That would be a mistake. Speed gained by blurring ownership is usually just
deferred confusion.

## Where the current architecture will strain

The current design is good, but it is not neutral. It already suggests its next
failure modes.

The biggest architectural risk is the growing centrality of `proposals`. At the
moment it is still a coherent pallet. It owns proposal lifecycle and acts as
the place where collective decisions are coordinated. That is reasonable. The
risk is what happens when every new governance capability is added there
because it feels natural to thread it through the proposal system. If that
continues unchecked, `proposals` stops being the coordinator of governance and
starts becoming the owner of every governance rule. Once that happens, the
project still compiles, but its architecture has quietly started to collapse.

The second strain point is the typed action model. Today it is one of the best
ideas in the codebase. It makes intent explicit and rejects ambiguity. But
typed action systems have a known trap: if everything becomes one larger and
larger enum, clarity flips into congestion. The pattern is sound only while the
surface area remains shaped by domain boundaries rather than by accumulation.

The third strain point is outside the runtime itself. The current CLI is for
testing, not for end-user experience. That means the first serious web
application will become the place where GAIA is interpreted by actual humans.
If the architecture does not prepare for that, the frontend will invent its own
semantic layer under delivery pressure. Frontends are very good at doing that.
They are much worse at doing it coherently.

The fourth strain point is operational rather than structural. GAIA can already
govern runtime upgrades, which is an important milestone. But technical power
arriving before operational maturity is a classic governance hazard. A system
can be perfectly capable of upgrading itself and still be weak at review,
trust, provenance, and safe release rhythm.

## The direction this roadmap recommends

GAIA should evolve toward a domain-driven governance architecture with strong
invariants, strong typing, explicit policy, and clear ownership boundaries. In
plain terms, the system should become more precise as it grows, not more
generic. The target is not an abstract governance framework that could mean
anything. The target is a governance system whose structures continue to say
what they mean.

That requires resisting two bad instincts. One is the instinct to do a large
rewrite once the prototype starts feeling cramped. Large rewrites are usually a
romantic way to discard learning. The other is the instinct to keep adding just
one more feature to the existing center of gravity until one pallet becomes the
de facto constitution, treasury office, and policy engine all at once. Neither
path is disciplined growth.

What disciplined growth looks like instead is gradual boundary hardening. Keep
`proposals` as the place where proposal lifecycle lives. Let it coordinate.
Do not let it become the natural home of every substantive rule forever. When a
new policy area becomes large enough to deserve its own vocabulary, invariants,
and tests, give it its own boundary rather than stretching an existing one past
the point where it still explains itself.

## What should be added later, and when

Not every good architectural pattern belongs in the current phase of the
project. Some belong later, when the problem has become real enough to justify
the extra structure.

One future improvement is stronger domain policy modeling. Today many policy
settings exist as individual storage items, which is acceptable for a small
system. Over time, related policies should become more coherent named objects
or at least be treated as clearer policy families. That will make audits,
reasoning, and future migrations cleaner. This should happen once the project
begins to feel policy-heavy rather than merely feature-complete.

Another later improvement is a clearer separation between write-side behavior
and read-side semantics. The runtime already expresses the write model fairly
well. The future web application will need stable, human-readable views of
proposal meaning, vote status, thresholds, and history. This should not be
forced into the runtime prematurely, but it should absolutely be planned before
the first user-facing interface grows large enough to invent its own local
truths.

A third future improvement is more explicit subdomain execution ownership. If
governance expands into new domains, the system should be able to preserve the
idea that proposals are the mechanism for collective choice, not the owner of
every outcome in detail. That means later refactors may need to separate
proposal coordination from the execution logic of expanding policy families.

## A staged path instead of a switch

The right improvement curve for GAIA is gradual and visible. Phase 0 is about
protecting what is already good. That means treating pallet ownership
boundaries as real constraints, continuing to use ADRs for architectural
choices, and keeping invariants as explicit contracts rather than oral
tradition. This phase is mostly about discipline, which is unfortunate because
discipline is less glamorous than new features and more valuable than most of
them.

Phase 1 is about clarifying domain language and policy shape. The project
should identify which rules belong together conceptually and make their naming
and documentation more coherent. This is a relatively cheap phase, but it pays
off because it makes future refactors smaller and less argumentative.

Phase 2 begins when the project starts preparing for a real application surface.
At that point the goal is not merely to expose chain data, but to make
governance meaning readable. The frontend should be able to explain proposal
classes, actions, thresholds, timing, and status without reverse-engineering
the runtime every time a new screen is built.

Phase 3 begins when governance scope starts to grow enough that one pallet is
doing too much explanatory work. That is the point to split large emerging
subdomains before they sprawl. Waiting longer is what produces the famous
"temporary central module" that never stops being central.

Phase 4 is about operational trust. By then the question is not whether GAIA
can govern powerful changes, but whether it can do so in a way that remains
reviewable and trustworthy as the stakes rise. Upgrade discipline, release
clarity, and provenance should become part of the architecture in practice, not
just part of the aspiration.

## How to read this roadmap

This document is intentionally written for human readers. It is not supposed to
function like `docs/current-state.md`, and it should not be edited as if it
were a status ledger. It exists to explain the shape of the architecture, the
risks in that shape, and the order in which change should happen. It should be
read as a design document, not skimmed as a checklist.

The simplest summary is this: GAIA already has several of the right patterns.
The real work now is to keep those patterns legible under growth. If the system
keeps its boundaries hard, its meanings explicit, and its evolution staged, it
can mature without a theatrical rewrite. If it starts optimizing for local
convenience, it will still function, but it will gradually become harder to
trust, harder to explain, and harder to change well.

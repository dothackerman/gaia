# ADR 015 — `docs/current-state.md` as system-facing current-state specification

## Context

The repository now contains two distinct documentation needs that should not be
collapsed into a single writing style.

The first need is **human-facing architectural explanation**. Files such as
`README.md` and `Roadmap.md` exist to be read as documents. Their job is not
merely to enumerate facts. Their job is to explain what GAIA is, why the
architecture is shaped the way it is, what risks exist in that shape, and how
the system should evolve. For those documents, legibility means narrative flow,
coherent prose, and conceptual continuity.

The second need is **system-facing implementation state**. Coding agents and
operator workflows need a compact, highly structured view of what is actually
implemented now: pallet boundaries, runtime wiring, dispatchables, storage
items, invariants, build state, and recent changes. For that document,
legibility means predictable structure, short factual statements, and low
ambiguity.

`docs/current-state.md` already serves that second purpose in practice. It is
read before coding and used as the live snapshot of the repository's last known
good state. However, its role has so far been implicit rather than formally
defined.

If these two needs are handled with the same style, several risks appear:

- human-facing documents become dry inventories instead of readable design
  documents
- agent-facing documents become more literary and less operational
- architecture discussion and implementation status become mixed together
- future contributors may edit `current-state.md` in a style that is pleasant
  for humans but harder for agents to parse and act on
- the project lacks a declared contract for what "current state" must include

GAIA now needs both document classes, and they should not be forced to do the
same job.

## Decision

Treat `docs/current-state.md` as the repository's **system-facing current-state
specification**.

This file is optimized for operator workflows, coding agents, and implementation
handoff. It is not the primary place for architectural persuasion, roadmap
argument, or broad product explanation.

Symmetrically, human-facing documents such as `README.md` and `Roadmap.md`
should optimize for reader comprehension rather than for terse machine-facing
scanability. They should read like documents, not like inventories.

### Contract for `docs/current-state.md`

`docs/current-state.md` should remain:

- **implementation-first**
- **high-signal**
- **structurally predictable**
- **easy to scan**
- **safe for agent consumption**

The file should answer:

- what components exist now
- what is wired now
- what behaviors are implemented now
- what invariants are enforced now
- what tests/build results are green now
- what hardening work remains open now

### Required content shape

The current-state specification should continue to include, at minimum:

- node status
- runtime status and wired pallets
- per-pallet implemented state
  - storage
  - dispatchables
  - lifecycle / behavioral notes
  - trait ownership or trait implementations
  - test counts where useful
- integration-test status
- tester or operator tooling status
- governance hardening status
- build status
- upstream warnings
- latest branch changes

### Style constraints

`docs/current-state.md` should prefer:

- short sections
- bullet lists
- stable headings
- implementation vocabulary
- direct statements of fact

`docs/current-state.md` should avoid:

- long architectural essays
- speculative future-state language
- persuasive roadmap prose
- duplicating ADR reasoning unless the decision is required to understand the
  present implementation

### Responsibility split across docs

The documentation split is now explicit:

- `README.md`
  - human-facing project introduction written for comprehension
- `Roadmap.md`
  - human-facing architectural direction and staged evolution written as a
    readable design document
- `docs/current-state.md`
  - system-facing snapshot of implemented state
- `docs/decisions/*.md`
  - why significant architectural decisions were made

### Update rule

When implementation changes materially affect system behavior, wiring, build
state, or governance capability:

1. update code
2. update or add ADRs if the change is architectural
3. update `docs/current-state.md` to reflect the new implemented truth

In parallel workflows, ADR 008 still applies:

- `docs/current-state.md` remains read-only during parallel branch work
- branch-local status lives in `docs/agent-state/`
- the merger updates `docs/current-state.md` as the integration step

## Consequences

**Positive**

- The project now has a clear split between human-facing explanation and
  system-facing implementation state.
- Coding agents get a stable operational briefing document instead of a moving
  mixture of design narrative and status notes.
- Human-facing documents can become more readable without losing the compact
  agent-oriented state snapshot.
- Architectural decisions, current implementation, and future direction each
  have a clearer home.
- The project can optimize legibility for the reader that actually uses each
  document instead of forcing one compromise style everywhere.

**Negative / accepted trade-offs**

- The repository now maintains multiple documentation surfaces with different
  purposes, which increases editorial discipline requirements.
- `docs/current-state.md` will sometimes feel terse or repetitive to human
  readers because it is intentionally optimized for operational consumption.
- Contributors must resist the temptation to turn every document into a hybrid
  "explainer plus status report" artifact.

## Follow-up

- Keep `docs/current-state.md` terse and structured as future features land.
- Use `Roadmap.md` for staged architectural evolution rather than overloading
  the current-state file with future intent.
- If the current-state document becomes too large, introduce stricter section
  templates rather than relaxing its system-facing role.

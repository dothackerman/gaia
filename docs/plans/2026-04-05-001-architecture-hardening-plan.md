---
title: Architecture Hardening Toward Domain-Driven Governance
type: refactor
status: active
date: 2026-04-05
---

# Architecture Hardening Toward Domain-Driven Governance

## Purpose

This plan describes how to move GAIA from its current strong-but-prototype
architecture toward the desired architectural state described in
[`Roadmap.md`](../../Roadmap.md).

The target state is not "more abstraction" in the abstract. The target state is
a governance system with:

- stronger domain ownership boundaries
- stronger policy legibility
- stronger client-facing semantics
- stronger operational trust around upgrades
- unchanged or improved invariant enforcement

This plan is written for a coding agent other than the author of the roadmap.
It is intentionally operational. It tells the implementing agent what to do,
how to do it, how to test it, and how to document it without turning the work
into a single broad rewrite.

## Executive Summary

GAIA already has several of the right patterns:

- bounded contexts through pallets
- trait-based cross-pallet contracts
- explicit state machines
- typed governance actions
- on-chain policy storage

The main architectural risks are also already visible:

- `pallets/proposals` becoming the center of every policy concern
- `GovernanceAction` growing into an unbounded catch-all
- future clients inventing their own semantic model of governance
- runtime-upgrade power outrunning operational review discipline

The implementation strategy is therefore conservative and staged:

1. make the current architecture more explicit and easier for agents to work in
2. make governance policy more coherent without changing ownership boundaries
3. prepare clean read-side and client-facing semantics before UI sprawl appears
4. split substantial new governance domains before `proposals` becomes a god pallet
5. harden upgrade and release operations as governance power increases

## Constraints

These constraints are non-negotiable for this plan:

- Preserve existing hard invariants from `AGENTS.md`.
- Keep changes minimal and sliceable.
- Do not perform a large architectural rewrite.
- Do not move logic across pallet boundaries merely to "clean things up."
- Keep `runtime/` changes serialized and deliberate.
- Treat documentation as part of the change, not as cleanup left for later.

## Non-Goals

This plan explicitly does not attempt to:

- build the future web application
- redesign the entire proposal system in one pass
- replace typed governance actions with opaque call blobs
- introduce generic governance framework abstractions
- weaken governance friction for convenience

## Desired Architectural State

When this plan is complete, GAIA should have the following properties:

1. The current architecture is easier for coding agents to understand and
   modify safely.
2. Governance policy is grouped and named coherently enough that the domain
   remains legible under growth.
3. Read-side semantics for external clients are clear before the first serious
   web application appears.
4. New governance domains can be added without permanently bloating
   `pallets/proposals`.
5. Runtime upgrade capability is paired with stronger operational discipline.

## Agent Modus Operandi

This section is part of the plan on purpose. The architecture will degrade if
an agent works quickly but not carefully.

### Required reading before each implementation slice

Before making code changes, the implementing agent must read:

- `AGENTS.md`
- `docs/current-state.md`
- `Roadmap.md`
- `docs/decisions/015-system-facing-current-state-specification.md`
- this plan file

For slices that touch governance history or protocol evolution, also read the
relevant ADRs in `docs/decisions/`.

### Required working style

The implementing agent should:

1. Choose one slice with one primary architectural purpose.
2. Identify the exact ownership boundary affected.
3. State what invariant, policy, or semantic contract is being changed.
4. Change code.
5. Add or update tests in the same slice.
6. Update documentation in the same slice.
7. Run the required cargo command hierarchy from `AGENTS.md`.
8. Commit the slice once it is green.

The implementing agent should not:

- mix unrelated refactors into the same slice
- change `runtime/` and multiple domain boundaries casually in one pass
- change docs as a separate cleanup phase
- add new abstractions without naming the concrete problem they solve

### Review questions before merging any slice

Every slice should answer these questions:

- Does this preserve or sharpen ownership boundaries?
- Does this make governance meaning clearer or murkier?
- Does this preserve all invariants?
- Does this add tests for the new contract?
- Does this keep the system easier to explain than before?

If any answer is "no" or "unclear", the slice is not ready.

## Workstreams

The plan is divided into five workstreams. They are intentionally sequential at
the top level even if some internal tasks can be parallelized.

### Workstream 0. Documentation Contract Alignment

#### Goal

Align the repository's major documents with the explicit split introduced by
ADR 015 so that coding agents and human readers stop competing for the same
document style.

#### Scope

- tighten `docs/current-state.md` into a more explicit system-facing template
- ensure `README.md` stays introductory and human-facing
- ensure `Roadmap.md` remains architectural and narrative
- document the update rules for future contributors

#### Proposed changes

1. Refactor `docs/current-state.md` headings into a stable agent-friendly shape
   if they are still inconsistent or too prose-heavy.
2. Add a short "document purpose" preface to `docs/current-state.md` if useful.
3. Review `README.md` and remove any content that belongs more naturally in
   `Roadmap.md` or `docs/current-state.md`.
4. Add cross-links among the three documents so contributors know where to edit.

#### Tests

Documentation-only slice, but still verify:

- links resolve
- references are internally consistent
- terminology matches `AGENTS.md` and `docs/domain-model.md`

#### Acceptance criteria

- An agent can read `docs/current-state.md` and get a crisp implementation snapshot.
- A human can read `Roadmap.md` and understand direction without wading through
  status bullets.
- A human can read `README.md` without being dragged into implementation-detail
  noise.

### Workstream 1. Policy Legibility and Domain Language

#### Goal

Make governance policy easier to reason about as a coherent domain without yet
performing major logic relocation.

#### Scope

- identify policy families already present in `membership` and `proposals`
- reduce "scattered storage value" semantics
- improve naming consistency
- clarify domain vocabulary for thresholds, timing, classes, and authority

#### Proposed changes

1. Audit governance-related storage items in:
   - `pallets/proposals/src/lib.rs`
   - `pallets/membership/src/lib.rs`
2. Introduce clearer grouping comments, helper types, or small policy structs
   where they improve legibility without forcing major migration complexity.
3. Standardize naming around:
   - proposal governance policy
   - membership governance policy
   - constitutional governance policy
4. Tighten helper methods so threshold logic and policy retrieval are easier to
   read and test.
5. Update ADRs if the grouping crosses from cleanup into real design change.

#### Tests

Add or expand unit tests to cover:

- threshold validation
- threshold retrieval and application paths
- governance-origin-gated policy updates
- equivalence of behavior before and after any refactor

Integration tests should verify:

- unchanged behavior for existing governance flows
- unchanged error behavior for invalid threshold changes

#### Documentation

- Update `docs/current-state.md` to describe policy families more coherently.
- Add or update ADRs only if the design changes materially.
- Add comments in code only where the policy grouping is non-obvious.

#### Acceptance criteria

- Policy logic is easier to locate and explain.
- No invariant or behavior regression occurs.
- Reviewers can point to a small number of named policy concepts rather than a
  dispersed set of magic storage items.

### Workstream 2. Read-Side and Client Semantic Preparation

#### Goal

Define stable, readable semantics for future external clients before a web app
becomes the place where governance meaning is improvised.

#### Scope

- proposal semantics
- membership proposal semantics
- watch/read interfaces
- human-readable descriptions for proposal classes and actions

#### Proposed changes

1. Audit `tester-cli` watch output and command vocabulary.
2. Identify the minimum stable read-side concepts clients will need:
   - proposal summary
   - proposal detail
   - membership proposal summary
   - lifecycle status
   - timing state
   - action explanation
3. Improve CLI watch output to reflect those semantics if current output is too
   raw or inconsistent.
4. Add a documentation artifact describing the intended external semantic model
   for proposal and membership views.
5. Avoid introducing a separate service layer until there is a concrete client
   need; for now, prioritize consistent meaning and output contracts.

#### Tests

Add tests for:

- CLI formatting/parsing where applicable
- human-readable class/action descriptions
- stability of output fields needed for future clients

If output contracts are documented, snapshot-style tests may be appropriate for
selected commands so semantic drift is visible.

#### Documentation

- Update `README.md` and `Roadmap.md` only where the new semantics change how
  users should understand the system.
- Update `docs/current-state.md` with any relevant tooling contract changes.
- Consider a dedicated reference doc if client semantics become large.

#### Acceptance criteria

- The future web app team could derive a first UI vocabulary from repository
  docs and CLI outputs without reverse-engineering the runtime.
- Proposal and membership states are described consistently across code, CLI,
  and docs.

### Workstream 3. `proposals` Boundary Defense and Domain Extraction

#### Goal

Prevent `pallets/proposals` from becoming the permanent owner of every future
governance concern.

#### Scope

- identify logic in `proposals` that is lifecycle coordination versus logic that
  is really another domain's policy
- extract only when a domain boundary is mature enough to deserve it
- preserve proposal lifecycle ownership inside `proposals`

#### Proposed changes

1. Audit `GovernanceAction` and its execution paths.
2. Classify each action as:
   - coordination concern
   - policy setter concern
   - external domain concern
3. Identify which future domains are likely to deserve their own boundary if
   expanded, for example:
   - richer membership governance
   - treasury intake policy
   - upgrade queue/release policy
4. Refactor execution helpers so `proposals` coordinates dispatch more cleanly,
   even before new pallets exist.
5. Only introduce a new pallet if a new domain has:
   - its own invariants
   - enough state to justify ownership
   - enough tests to justify isolation

#### Tests

Unit and integration tests must verify:

- proposal lifecycle remains unchanged
- action-class mapping remains correct
- execution still enforces delay, approval state, and single execution
- any extracted helper or domain execution path preserves error behavior

Regression tests should be added before logic movement when the movement risks
semantic drift.

#### Documentation

- Update `docs/current-state.md` whenever action ownership or execution flow changes.
- Add ADRs for any real extraction or new domain boundary.
- Update `Roadmap.md` only if the architectural direction itself changes.

#### Acceptance criteria

- `pallets/proposals` remains the place where proposals live and are coordinated.
- New policy complexity is not absorbed there by default.
- Reviewers can explain why a responsibility lives where it lives.

### Workstream 4. Upgrade and Operational Trust Hardening

#### Goal

Strengthen the operational architecture around governance-driven runtime change.

#### Scope

- runtime upgrade review discipline
- artifact provenance
- queueing semantics if needed
- documentation of operational procedure

#### Proposed changes

1. Review the current `UpgradeRuntime` flow and its documented trade-offs.
2. Decide whether the single pending code blob model needs better operational
   framing before deeper code changes.
3. Improve documentation and operator guidance for:
   - code upload
   - hash review
   - metadata refresh
   - execution timing
   - rollback expectations
4. If needed, plan a later slice for multi-blob queueing, but do not add it
   unless the single-slot model is demonstrably constraining work.

#### Tests

Add or maintain tests for:

- code-hash mismatch rejection
- missing blob rejection
- size-limit rejection
- pending blob clearing on success

If operational tooling changes, add tests around the CLI path where practical.

#### Documentation

- Update the relevant ADRs if runtime upgrade semantics change.
- Add operator-facing instructions if the workflow becomes safer or richer.
- Reflect new guarantees or limitations in `docs/current-state.md`.

#### Acceptance criteria

- Runtime upgrade flow is easier to review and explain.
- Operational trust grows with governance power.
- No upgrade hardening change weakens the underlying safety checks.

## Execution Order

Top-level order:

1. Workstream 0
2. Workstream 1
3. Workstream 2
4. Workstream 3
5. Workstream 4

This order is intentional.

Do not start with extraction. Start by making the current architecture easier to
read. Extraction before legibility usually produces new boundaries with old
confusion inside them.

## Slice Strategy

Each workstream should be broken into small slices. A good slice usually has:

- one architectural purpose
- one to three touched files in the core implementation
- tests added or updated in the same change
- documentation updated in the same change

Examples of good slices:

- tighten `docs/current-state.md` structure and cross-links
- group proposal threshold retrieval behind clearer helpers with tests
- improve watch output semantics for proposal detail view
- refactor runtime upgrade helper paths without changing behavior

Examples of bad slices:

- "clean up proposals architecture"
- "prepare for web app"
- "refactor governance"

If the slice title sounds like a theme rather than a change, it is too large.

## Testing Protocol

For any code slice, follow the command hierarchy from `AGENTS.md` exactly:

1. `cargo check`
2. `cargo clippy`
3. `cargo test`
4. `cargo build`

Testing expectations by layer:

- Pallet-local logic changes:
  - add or update unit tests in the pallet
- Cross-pallet behavior changes:
  - add or update integration tests in `tests/`
- CLI semantic/output changes:
  - add parser/output/contract tests in `tester-cli`
- Documentation-only slices:
  - verify links, terminology, and consistency manually

Regression-first guidance:

- before moving logic, capture the current behavior in tests
- then refactor
- then verify tests still prove the same contract

## Documentation Protocol

Documentation is part of the definition of done.

For each slice, decide which docs need changes:

- `docs/current-state.md`
  - when implemented behavior, status, wiring, or tooling contract changes
- `docs/decisions/*.md`
  - when an architectural decision or trade-off changes
- `Roadmap.md`
  - only when architectural direction changes, not for every implementation step
- `README.md`
  - when user-facing explanation of the project materially changes

If none of the documentation needs updating, the implementing agent should say
why.

## Risks and Failure Modes

### Failure mode 1. Broad cleanup disguised as architecture

Mitigation:

- demand a single architectural purpose per slice
- reject generalized cleanup commits

### Failure mode 2. `proposals` keeps growing because extraction feels expensive

Mitigation:

- track ownership in reviews
- add ADRs before a new policy area becomes permanent by accident

### Failure mode 3. Tests lag behind refactors

Mitigation:

- require regression tests before logic movement
- treat missing tests as a blocker, not a follow-up

### Failure mode 4. Documentation becomes inaccurate while code improves

Mitigation:

- update docs in the same slice
- use ADR 015 as the document responsibility contract

### Failure mode 5. Runtime changes become casual

Mitigation:

- serialize runtime-heavy work
- require explicit rationale for each runtime wiring change

## Milestones and Exit Criteria

### Milestone A. Legibility stabilized

Exit criteria:

- document responsibilities are clear
- `docs/current-state.md` is easier for agents to consume
- policy vocabulary is more coherent than before

### Milestone B. Client semantics prepared

Exit criteria:

- watch/read surfaces express stable meaning
- future client teams have a usable semantic contract

### Milestone C. Boundary pressure reduced

Exit criteria:

- `pallets/proposals` is still explainable as a coordinator rather than a god pallet
- new or expanded policy areas have cleaner ownership

### Milestone D. Upgrade trust improved

Exit criteria:

- runtime upgrade flow is easier to review, explain, and operate safely

## Recommended First Slice

Start with Workstream 0 and keep it modest:

1. align `docs/current-state.md` more tightly with ADR 015
2. add explicit cross-links among `README.md`, `Roadmap.md`, and
   `docs/current-state.md`
3. update `docs/current-state.md` headings only as much as needed for a stable
   system-facing template

Why this first:

- it improves every later agent's working context
- it creates low-risk clarity
- it forces the project to operationalize the document split before deeper code work

## Sources

- [`AGENTS.md`](../../AGENTS.md)
- [`Roadmap.md`](../../Roadmap.md)
- [`docs/current-state.md`](../current-state.md)
- [`docs/domain-model.md`](../domain-model.md)
- [`docs/decisions/015-system-facing-current-state-specification.md`](../decisions/015-system-facing-current-state-specification.md)

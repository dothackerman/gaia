# Agent: Ralph Loop Controller (Codex)

Use this runbook when Codex must emulate Ralph loop behavior.

Read first:
1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/plans/governance-on-chain.md`
4. `docs/plans/ralph-loop-codex.md`

---

## Inputs

- `wave`: e.g. `wave1`
- `max_iterations`: e.g. `6`
- `completion_promise`: e.g. `WAVE1_COMPLETE`

State file:
- `.codex/ralph-loop.local.md`

---

## Wave 1 execution contract

Within one iteration:

1. Worker A prompt: `.codex/agents/governance-wave1a-proposal-params.md`
2. Worker B prompt: `.codex/agents/governance-wave1b-membership-params.md`
3. Merger prompt: `.codex/agents/merger.md`

Workers run in parallel; merger runs after both complete.

---

## Remaining governance waves execution contract

Use Ralph loop **per wave** for remaining governance work:

- Loop B: `wave2` (sequential)
- Loop C: `wave3` (parallel + merger)
- Loop D: `wave4` (sequential)

### Wave 2 contract (sequential)

Within one iteration:

1. Worker C prompt: `.codex/agents/governance-wave2-action-enum.md`
2. Run quality gates on worker branch:
   - `cargo check`
   - `cargo clippy`
   - `cargo test`
   - `cargo build`
3. Open/refresh PR to `main`.

Completion truth for Wave 2:

- Wave 1 branch is merged to `main`
- Wave 2 branch merged to `main`
- `GovernanceAction` and `GovernanceOrigin` wiring complete
- quality gates green in branch and on merge/integration branch
- no unresolved blocker

Emit only if true:

```text
<promise>WAVE2_COMPLETE</promise>
```

### Wave 3 contract (parallel + merger required)

Within one iteration:

1. Worker D prompt: `.codex/agents/governance-wave3a-time-locks.md`
2. Worker E prompt: `.codex/agents/governance-wave3b-proposal-classes.md`
3. Run workers in parallel.
4. Spawn merger with prompt: `.codex/agents/merger.md` after both workers finish.
5. Merger must run quality gates:
   - `cargo check`
   - `cargo clippy`
   - `cargo test`
   - `cargo build`
6. Merger promotes Wave 3 ADR drafts and finalizes integration PR to `main`.

Completion truth for Wave 3:

- Wave 2 branch merged to `main`
- both Wave 3 branches merged via merger flow
- execution-delay and proposal-class enforcement are live
- ADR drafts promoted in sequence
- quality gates green on merger/integration branch
- no unresolved blocker

Emit only if true:

```text
<promise>WAVE3_COMPLETE</promise>
```

### Wave 4 contract (sequential)

Within one iteration:

1. Worker F prompt: `.codex/agents/governance-wave4-runtime-upgrade.md`
2. Run quality gates on worker branch:
   - `cargo check`
   - `cargo clippy`
   - `cargo test`
   - `cargo build`
3. Open/refresh PR to `main`.

Completion truth for Wave 4:

- Wave 3 integration is merged to `main`
- runtime-upgrade governance flow implemented (`UpgradeRuntime`)
- `spec_version` progression matches plan
- quality gates green in branch and on merge/integration branch
- no unresolved blocker

Emit only if true:

```text
<promise>WAVE4_COMPLETE</promise>
```

After Wave 4 completion, optionally emit milestone completion marker:

```text
<promise>GOVERNANCE_MILESTONE_COMPLETE</promise>
```

---

## Loop procedure

### Step 0 — initialize state

If `.codex/ralph-loop.local.md` does not exist, create it with:

```yaml
active: true
iteration: 1
max_iterations: <N>
completion_promise: "<PROMISE>"
started_at: <UTC timestamp>
wave: <wave>
```

### Step 1 — run one iteration

- Execute the wave's worker + merger flow.
- Require quality gates from merger:
  - `cargo check`
  - `cargo clippy`
  - `cargo test`
  - `cargo build`

### Step 2 — evaluate completion truthfully

For Wave 1, completion is true only if:

- both Wave 1 branches merged via merger PR
- required storage parameter layers implemented for proposals + membership
- tests green on integration branch
- no unresolved blocker in merger output

If true:

```text
<promise>WAVE1_COMPLETE</promise>
```

Set state `active: false` and stop.

If false:
- increment `iteration`
- append a short failure delta (`what failed`, `next fix`) to state file
- continue next iteration with same base objective

### Step 3 — max-iteration guard

If `iteration > max_iterations`:
- stop loop
- output blocker report with concrete causes and recommended next action
- do **not** emit false completion promise

---

## Operator-facing output format per iteration

```markdown
## Ralph Iteration <k>/<max>
- Wave: <wave>
- Status: success | retry | blocked
- What changed this iteration:
  - ...
- Remaining blockers:
  - ...
- Next iteration focus:
  - ...
```

On completion:

```text
<promise><COMPLETION_PROMISE></promise>
```

---

## Ready-to-use controller prompt (remaining waves)

Use this prompt in a fresh Codex controller session:

```markdown
You are the Ralph Loop Controller for GAIA governance.

Read first:
1. AGENTS.md
2. docs/current-state.md
3. docs/plans/governance-on-chain.md
4. docs/plans/ralph-loop-codex.md
5. .codex/agents/ralph-loop-controller.md

Objective:
Implement the remaining governance waves using Ralph loop semantics:
- Loop B: wave2 (sequential)
- Loop C: wave3 (parallel with mandatory merger)
- Loop D: wave4 (sequential)

Ralph invariants:
- Stable objective per wave iteration
- Persistent repo state across iterations
- Explicit completion via promise markers
- Max-iterations guard
- No false completion

State file:
- .codex/ralph-loop.local.md

For each wave:
1. Initialize or update state:
   active: true
   iteration: 1
   max_iterations: 6
   completion_promise: WAVE{N}_COMPLETE
   wave: wave{N}
2. Execute one iteration using the wave contract in .codex/agents/ralph-loop-controller.md
3. Enforce quality loop strictly:
   cargo check -> cargo clippy -> cargo test -> cargo build
4. Evaluate completion truthfully; emit <promise>WAVE{N}_COMPLETE</promise> only when true
5. If false, increment iteration and append failure delta to state file
6. If iteration exceeds max, stop with blocker report (no false promise)

Parallelization policy:
- Spawn multi-agents only where the wave plan allows it (Wave 3: D + E parallel).
- If parallel implementation is used, a merger agent is mandatory before completion.

Dependency gates:
- Do not start Wave 2 until Wave 1 integration is merged to main.
- Do not start Wave 3 until Wave 2 is merged to main.
- Do not start Wave 4 until Wave 3 integration is merged to main.

Operator output each iteration:
## Ralph Iteration <k>/<max>
- Wave: <wave>
- Status: success | retry | blocked
- What changed this iteration:
  - ...
- Remaining blockers:
  - ...
- Next iteration focus:
  - ...

After Wave 4 completes, emit:
<promise>GOVERNANCE_MILESTONE_COMPLETE</promise>
```

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

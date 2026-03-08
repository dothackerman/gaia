# Ralph Loop for Codex (GAIA)

## Why this exists

Claude has native stop-hook plumbing for Ralph loops. Codex does not.
So for Codex we implement Ralph as an explicit **controller protocol**.

The controller is a Codex session that re-issues prompts iteration-by-iteration,
checks completion criteria, and only stops when completion is genuinely true
or max iterations are reached.

---

## Core behavior (ported from Claude plugin semantics)

Ralph loop invariants:

1. **Prompt is stable across iterations** (same task intent).
2. **Files persist between iterations** (work accumulates in repo).
3. **Completion is explicit** via a promise marker.
4. **Max-iterations guard** prevents infinite runs.
5. **No false completion** allowed to force exit.

For Codex, this is implemented by a controller session instead of a stop hook.

---

## State file

Path:

```
.codex/ralph-loop.local.md
```

Format:

```markdown
---
active: true
iteration: 1
max_iterations: 10
completion_promise: "WAVE1_COMPLETE"
started_at: "2026-03-08T10:00:00Z"
wave: "wave1"
---

Implement Wave 1 from docs/plans/governance-on-chain.md.
```

This mirrors the Claude plugin's state-file approach and keeps loop state
inspectable in git history.

---

## Controller algorithm

For each iteration:

1. Read `.codex/ralph-loop.local.md`.
2. Spawn/drive worker sessions for the current wave scope.
3. Run merger session.
4. Evaluate completion criteria (tests + required files + acceptance checks).
5. If true, output:
   ```
   <promise>WAVE1_COMPLETE</promise>
   ```
   and set `active: false`.
6. If false, increment `iteration` and run next loop with same base prompt,
   appending what failed and what to fix next.
7. Stop if `iteration > max_iterations` and report blocker summary.

---

## Recommended loop granularity for GAIA

Use Ralph loop **per wave**, not for whole milestone:

- Loop A: Wave 1 (A+B parallel + merger)
- Loop B: Wave 2
- Loop C: Wave 3 (D+E parallel + merger)
- Loop D: Wave 4

This reduces blast radius and gives hard checkpoints.

---

## Safety / truth rule

The completion promise may be emitted **only if statement is true**.

Never output a false promise just to terminate the loop.

If max iterations reached without success, output blocker report instead of
false completion.

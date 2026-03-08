# ADR 008 — Multi-agent workflow protocol

## Context

As the project grows, single-agent sequential development becomes the
bottleneck. The pallet dependency graph (`treasury` and `membership` are
leaves; `proposals` depends on both) creates natural parallelization
boundaries. Two workflow patterns are in scope:

**Pattern A — Ralph loop (parallel Codex sessions):**
Two or more Codex instances work in parallel, each in an isolated git
worktree on a dedicated branch. When both complete, a third Codex session
acts as merger: reviews both branches, resolves conflicts, and integrates
into `main` via PR.

**Pattern B — Orchestrator + sub-agents:**
A single orchestrator Codex session decomposes the task, spawns sub-agent
sessions for implementation, collects results, and handles integration.
The orchestrator owns the merge.

Both patterns are valid. Pattern A maximises parallel throughput for
well-defined, independent tasks. Pattern B is better for tasks with
ambiguous boundaries or high interdependency.

## Problems with the naive approach

1. **ADR number collisions.** Sequential integers with no coordination
   mechanism. Two parallel agents creating `007-*.md` for different
   decisions silently collide.

2. **`docs/current-state.md` write conflicts.** Parallel agents reading
   and writing the same file produce divergent state and merge conflicts.

3. **No worktree lifecycle standard.** Each agent invents its own
   checkout pattern, leaving stale worktrees and broken refs.

4. **No merger acceptance criteria.** Without a spec, the merger role
   is undefined and inconsistent.

## Decision

### ADR draft protocol

Parallel agents **must not** claim a sequential ADR number directly.
Instead:

1. Create the draft in `docs/decisions/draft/<branch-slug>-<title>.md`.
2. Draft format is identical to numbered ADRs (this file as template).
3. The merger agent promotes drafts to numbered ADRs in sequence as
   the final step before opening the integration PR.
4. No draft file is ever committed to `main` — only promoted numbered
   ADRs reach `main`.

### Agent state during parallel sessions

`docs/current-state.md` is **read-only** for parallel agents. It
represents the last known good state from `main`.

Each parallel agent writes session progress to:
```
docs/agent-state/<branch-slug>.md
```

Format: free-form markdown. Required sections: `## What changed`,
`## Build state`, `## Open issues`. The merger reads all
`docs/agent-state/` files and incorporates them into `current-state.md`
as the final merge step.

After a successful merge, the merger **deletes** the consumed
`docs/agent-state/` files and the `agent-state/` directory is left
empty (or absent) between parallel sessions.

### Worktree lifecycle (Pattern A)

```bash
# Create (from project root, outside any existing worktree)
git worktree add ../gaia.worktrees/codex-<task>-<YYYY-MM-DD> -b codex/<task>

# Work inside the worktree — same repo, isolated working tree
cd ../gaia.worktrees/codex-<task>-<YYYY-MM-DD>

# Cleanup after merge
cd <project-root>
git worktree remove ../gaia.worktrees/codex-<task>-<YYYY-MM-DD>
git branch -d codex/<task>   # only after PR merged
```

Naming convention: `codex-<task>-<YYYY-MM-DD>` (e.g.,
`codex-treasury-fee-model-2026-03-07`).

### Merger runbook reference

See `.codex/agents/merger.md` for the step-by-step merger protocol for
both Pattern A and Pattern B.

## Consequences

- Parallel Codex sessions are safe to run without explicit coordination
  on `docs/current-state.md` or ADR numbering.
- The merger role has a defined spec and acceptance criteria.
- Worktree lifecycle is standardised and leaves no stale state.
- Sequential (single-agent) workflow is unaffected — the draft/ and
  agent-state/ conventions are only triggered in parallel contexts.

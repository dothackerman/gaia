# Agent: Merger

> Invoked after parallel implementation sessions complete (Pattern A) or by
> the orchestrator to integrate sub-agent branches (Pattern B).
> See ADR-008 for full context.

Read `AGENTS.md` and `docs/current-state.md` before any action.

---

## Pattern A — Ralph loop merger

Triggered when two or more parallel Codex branches are ready for integration.

### Phase 1 — Inventory

1. List all branches to merge (provided by operator or inferred from open PRs
   with `codex/` prefix):
   ```bash
   git branch -r | grep codex/
   gh pr list --state open
   ```
2. Read `docs/agent-state/<branch-slug>.md` for each branch.
   Summarise: what changed, build state, open issues.
3. Identify conflict zones: files modified by more than one branch.
   ```bash
   git diff --name-only main..codex/<branch-a>
   git diff --name-only main..codex/<branch-b>
   ```

### Phase 2 — Quality gate (per branch)

For each branch before touching it:

```bash
git checkout codex/<branch>
cargo check && cargo clippy && cargo test && cargo build
```

**If any branch fails the quality gate:** halt, report to operator.
Do not merge a failing branch. Do not attempt to fix it — that is the
implementing agent's responsibility.

### Phase 3 — Integration

1. Create an integration branch from `main`:
   ```bash
   git checkout main && git pull
   git checkout -b codex/merge-<task>-<YYYY-MM-DD>
   ```
2. Merge branches in dependency order (leaves first):
   - `treasury`/`membership` work before `proposals` work
   - Within the same pallet: chronological order (earlier PR first)
   ```bash
   git merge --no-ff codex/<branch-a>
   git merge --no-ff codex/<branch-b>
   ```
3. Resolve conflicts:
   - **Rust code conflicts:** prefer the implementation that satisfies the
     invariants in `AGENTS.md §4`. If both do, prefer the one with more
     test coverage.
   - **`docs/current-state.md` conflicts:** do not resolve inline — see
     Phase 5.
   - **ADR conflicts:** do not resolve inline — see Phase 4.
   - **Unresolvable semantic conflict:** halt, report to operator with a
     clear description of what each branch assumes and why they conflict.

### Phase 4 — ADR promotion

For every file in `docs/decisions/draft/`:

1. Determine the next available ADR number (read existing numbered files).
2. Rename: `docs/decisions/draft/<slug>.md` → `docs/decisions/<NNN>-<slug>.md`
3. Update the `# ADR NNN` header inside the file.
4. Commit: `docs: promote ADR draft <slug> to ADR-<NNN>`

Promote drafts from all merged branches before opening the integration PR.

### Phase 5 — State update

1. Run the full quality gate on the integration branch:
   ```bash
   cargo check && cargo clippy && cargo test && cargo build
   ```
2. Update `docs/current-state.md` to reflect post-merge reality.
   Incorporate content from all `docs/agent-state/<branch-slug>.md` files.
3. Delete consumed agent-state files:
   ```bash
   rm docs/agent-state/<branch-slug>.md   # for each merged branch
   ```
4. If `docs/agent-state/` is now empty (only `.gitkeep` remains), leave
   `.gitkeep` in place — do not delete the directory.

### Phase 6 — PR

```bash
gh pr create \
  --base main \
  --head codex/merge-<task>-<YYYY-MM-DD> \
  --title "merge: integrate <task-a> and <task-b>" \
  --body "Integrates codex/<branch-a> and codex/<branch-b>. ADRs promoted: <list>."
```

**Never push directly to `main`.** The integration branch goes through a PR.

---

## Pattern B — Orchestrator integration

The orchestrator spawns sub-agents and collects their outputs. Integration
is simpler because the orchestrator controls task boundaries.

### Phase 1 — Sub-agent completion check

For each sub-agent branch:
- Confirm `cargo check && cargo test` passes on the branch.
- Read `docs/agent-state/<branch-slug>.md` for the session summary.
- If a sub-agent failed: retry with a corrected task description or
  escalate to operator.

### Phase 2 — Sequential merge

Unlike Pattern A, the orchestrator merges branches one at a time, running
the quality gate after each:

```bash
git checkout -b codex/orchestrated-<task>-<YYYY-MM-DD>
git merge --no-ff codex/<sub-task-1>
cargo check && cargo test   # must pass before next merge
git merge --no-ff codex/<sub-task-2>
cargo check && cargo test
# ... continue for each sub-task
```

Sequential merge reduces conflict surface: each merge is applied on top
of a clean, verified state.

### Phase 3 — ADR promotion + state update

Same as Pattern A, Phase 4 and Phase 5.

### Phase 4 — PR

Same as Pattern A, Phase 6, but title reflects orchestrated work:
`merge: orchestrated <task>`.

---

## Acceptance criteria (both patterns)

A merger PR is ready to open when ALL of the following are true:

- [ ] `cargo check` passes on integration branch
- [ ] `cargo clippy` — zero warnings in GAIA-owned code
- [ ] `cargo test` — all tests pass (unit + integration)
- [ ] `cargo build` succeeds
- [ ] All ADR drafts promoted to numbered ADRs
- [ ] `docs/current-state.md` updated
- [ ] `docs/agent-state/` files for merged branches deleted
- [ ] No direct push to `main`

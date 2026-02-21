---
name: post-build
description: Invoke after a successful build to produce a structured analysis of all warnings and update living documentation. Use when you want a full report with ADR creation and current-state updates.
argument-hint: Paste the full cargo build/check output as the argument, or leave empty to have the agent read the most recent terminal output.
tools: [vscode, execute, read, agent, edit, search, web, todo]
---

Read `AGENTS.md` and `docs/current-state.md` before any action.

**Input:** the build output from the most recent cargo command.

**Step 1 — Classify every warning** into one of four buckets:
- AUTO_FIX: warnings in GAIA-owned code with a clear compiler suggestion
- LOG: upstream warnings in dependencies you do not own
- ADR: warnings implying a future architectural decision or migration
- OPERATOR: ambiguous warnings requiring human judgement

**Step 2 — Execute by bucket in order:**

AUTO_FIX: apply all fixes, run `cargo check` to confirm clean, report what was changed.

LOG: append each item to `docs/current-state.md` under a `## Upstream Warnings` section with the crate name, warning summary, and date.

ADR: create one ADR per distinct decision in `docs/decisions/` with the next available number. Include: what the warning says, what future action is required, and the recommended timeline (now / next minor / next major).

OPERATOR: list all ambiguous warnings clearly and halt. Do not proceed until operator provides classification.

**Step 3 — Commit** all changes with message:
`chore: post-build analysis and warning classification [date]`

Only commit if `cargo check` is clean after AUTO_FIX changes.
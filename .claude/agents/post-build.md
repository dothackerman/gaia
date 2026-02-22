# Agent: Post-Build Analysis

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

**Step 3 — License Audit:**

Run license audit with:
```bash
cargo deny check licenses -A parse-error > cargo-deny-licenses.log 2>&1
code=$?
echo "exit=$code"
tail -n 25 cargo-deny-licenses.log | sed -e 's/\x1b\[[0-9;]*m//g'
```

The `-A parse-error` flag is intentional and must not be removed. It tolerates upstream crates that ship non-SPDX license strings — malformed formatting in a dependency you do not own. It does not suppress actual license violations.

Classify the output as follows:
- `exit=0` — audit passes, log to `current-state.md` as clean with date
- `exit=1` with a license violation — classify as OPERATOR, halt and report
- `exit=1` with only parse errors — something changed upstream, investigate before proceeding

Never treat a non-zero exit code as ignorable without reading the tail output first.

**Step 4 — Commit** all changes with message:
`chore: post-build analysis and warning classification [date]`

Only commit if `cargo check` is clean after AUTO_FIX changes.

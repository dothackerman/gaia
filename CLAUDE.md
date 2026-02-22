# CLAUDE.md — Claude Code session instructions

Follow these rules for every session in this repository.

## Session start checklist

1. Read `AGENTS.md` in full before writing any code.
2. Read `docs/current-state.md` to understand the current build state before
   writing any code.

## Git / PR workflow

**Never push directly to `main`.** All changes go through a pull request:

1. Branch from `main`: `git checkout -b claude/<kebab-case-description>`
2. Implement and commit on the branch.
3. Push the branch and open a PR: `gh pr create`

**Branch naming:** `claude/<kebab-case-description>`
(e.g. `claude/implement-treasury-pallet`)

**PR titles:** short, imperative, sentence-case — follow existing PR titles
in the repository.

## Cargo command hierarchy

Always iterate in this order — never skip a level:

```
cargo check   →   cargo clippy   →   cargo test   →   cargo build
```

`cargo check` must pass before any other command is run.

## Code quality rules, ADR requirements, post-build analysis

See `AGENTS.md` §4–8. The same rules apply to Claude Code sessions.

## Runbooks

| Runbook | When to use |
|---|---|
| `.claude/agents/post-build.md` | After every successful build |
| `.claude/agents/security-upgrade.md` | When a CVE is identified in a dependency |

# GitHub Copilot — Global agent instructions

Follow these rules for every session in this repository.

## Session start checklist

1. Read `AGENTS.md` in full before writing any code.
2. Read `docs/current-state.md` to understand the current build state before
   writing any code.

## Cargo command hierarchy

Always iterate in this order — never skip a level:

```
cargo check   →   cargo clippy   →   cargo test   →   cargo build
```

`cargo check` must pass before any other command is run.

## Code quality rules

- Write tests for every public pallet function. Unit tests live in the same file
  under `#[cfg(test)]`; integration tests live in `tests/`.
- One responsibility per pallet. Do not add logic to a pallet that belongs to
  another pallet.
- Keep changes minimal and focused (KISS, YAGNI). Do not add abstractions or
  features that are not required by the current task.

## Decision records

Record every significant architectural or design decision as an ADR (Architecture
Decision Record) in `docs/decisions/`. Use the next sequential number and the
format established in `docs/decisions/001-standalone-chain.md`.

Significant decisions include (but are not limited to):

- Choosing or rejecting a Substrate pallet or feature
- Changing the token economics model
- Adding or removing a pallet
- Altering any of the invariants listed in `AGENTS.md`

## Post-Build Output Analysis

After every cargo command that produces output, classify all warnings before proceeding:

**Auto-fix immediately** (do not ask, just fix):
- unused parentheses
- unused variables
- unused imports
- clippy suggestions with a clear `help:` line

**Log to `docs/current-state.md`** (do not fix, just record):
- upstream dependency warnings (code you do not own)
- future Rust rejection warnings on external crates
- architecture or cycle detection output from the overseer

**Create ADR in `docs/decisions/`** (next available number):
- new build target recommendations (e.g. wasm32v1-none migration)
- any warning that implies a required future action on GAIA code

**Never silently ignore a warning.** Every warning is either fixed, logged, or recorded as a decision. If classification is ambiguous, log it to `current-state.md` and flag it to the operator.
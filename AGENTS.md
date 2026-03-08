# AGENTS.md — Operational context for AI coding agents

> **Read this file in full before writing any code.**
> Then read [`docs/current-state.md`](docs/current-state.md) to know what has
> already been built.

---

## 1. Project identity

| Key | Value |
|---|---|
| Name | GAIA |
| Type | Private, standalone Substrate solochain |
| Relay chain | None — fully self-sovereign |
| Consensus | Single-authority (development); no external validators |
| Purpose | On-chain treasury governed by equal-weight member voting |

## 2. Repository map

```
AGENTS.md                          # ← YOU ARE HERE — agent instructions
CLAUDE.md                          # Claude Code session instructions (auto-read)
README.md                          # Human-facing project overview
docs/
  current-state.md                 # Live build status — read before coding (READ-ONLY in parallel sessions)
  agent-state/                     # Per-session state files written by parallel agents (see §13)
  decisions/                       # ADRs (Architecture Decision Records)
    draft/                         # Draft ADRs from parallel sessions — merger promotes to numbered
    001-standalone-chain.md
    002-versioning-policy.md
    003-wasm32v1-none-target.md
    004-project-license-gpl-3.0-or-later.md
    005-suspension-unanimity.md
    006-treasury-sovereign-account.md
.github/
  copilot-instructions.md          # GitHub Copilot session rules (references this file)
  agents/
    post-build.md                  # Post-build warning-classification workflow
    security-upgrade.md            # Runbook for security dependency upgrades
.codex/
  instructions.md                  # Codex session rules (mirrors Copilot setup)
  agents/
    post-build.md                  # Post-build warning-classification workflow
    security-upgrade.md            # Security dependency-upgrade runbook
    merger.md                      # Merger agent protocol (Pattern A + B) — see §12
    ralph-loop-controller.md       # Codex Ralph loop controller protocol
.claude/
  agents/
    post-build.md                  # Post-build warning-classification workflow
    security-upgrade.md            # Security dependency-upgrade runbook
    merger.md                      # Merger agent protocol (mirror)
node/                              # Substrate node binary
runtime/                           # Runtime crate — wires pallets together
pallets/
  membership/                      # Pallet: member registry (implemented)
  treasury/                        # Pallet: community fund (implemented)
  proposals/                       # Pallet: spending proposals + voting (implemented)
```

## 3. Three-pallet architecture

### 3.1 Pallet responsibilities

| Pallet | Path | Single responsibility |
|---|---|---|
| **membership** | `pallets/membership/` | Maintains the authoritative set of active members. Exposes `is_active_member(AccountId) -> bool`. Handles registration and deactivation. |
| **treasury** | `pallets/treasury/` | Holds the community token balance. Accepts fee deposits. Executes approved disbursements. Enforces non-negative balance. |
| **proposals** | `pallets/proposals/` | Full lifecycle of a spending proposal: submission → voting → tally → approval/rejection → single execution. |

### 3.2 Pallet dependency graph

```
proposals ──uses──▸ membership   (member-validity check on every vote)
proposals ──uses──▸ treasury     (fund transfer on approved proposal)
treasury  ──uses──▸ (none)
membership──uses──▸ (none)
```

**Rule: pallets couple through traits, never concrete types.** Each downstream
pallet defines a trait (e.g., `MembershipChecker`, `TreasuryHandler`) that the
upstream pallet implements. The runtime wires the concrete types.

### 3.3 One responsibility per pallet

Do NOT add logic to a pallet that belongs to another pallet. If new
functionality does not fit any existing pallet, propose a new one via an ADR.

## 4. Invariants — NEVER violate

These are hard constraints checked in code **and** in tests.

| # | Invariant | Enforcement |
|---|---|---|
| I-1 | **Treasury balance ≥ 0.** Any disbursement that would underflow must return an error, not silently clamp. | Guard in treasury dispatchable; unit test asserting `InsufficientFunds` error. |
| I-2 | **Only active members vote.** `proposals` must call `membership::is_active_member` on every vote extrinsic and reject non-members. | Guard in proposals dispatchable; unit test with deactivated member. |
| I-3 | **A proposal executes at most once.** After transitioning to `Executed`, further execution attempts must return an error. | State check in proposals dispatchable; unit test for double-execute. |

## 5. Cargo command hierarchy

Execute in this exact order. **Never skip a level.**

```
cargo check   # 1. Type/syntax check. MUST pass before anything else.
cargo clippy  # 2. Lint. Fix ALL warnings before proceeding.
cargo test    # 3. Unit + integration tests. ALL must pass.
cargo build   # 4. Full compile. Only after 1–3 are green.
```

If `cargo check` fails → stop → fix → re-run `cargo check`. Do not run
`clippy`, `test`, or `build` until `check` passes.

## 6. Code conventions

### 6.1 Tests

- **Unit tests** live in the same file under `#[cfg(test)] mod tests { … }`.
- **Integration tests** live in `tests/` at the workspace root.
- Write at least one test for every public dispatchable and every invariant.
- Name tests descriptively: `fn vote_rejects_inactive_member()`, not `fn test1()`.

### 6.2 Errors

- Each pallet defines its own `#[pallet::error]` enum.
- Error variants are descriptive: `InsufficientFunds`, `NotActiveMember`,
  `ProposalAlreadyExecuted`.
- Never use `unwrap()` or `panic!()` in non-test code.

### 6.3 Style

- Follow standard `rustfmt` formatting.
- Keep changes minimal and focused (KISS, YAGNI).
- Do not add abstractions or features not required by the current task.

## 7. Architecture Decision Records (ADRs)

Record every significant architectural or design decision in
`docs/decisions/`. Use the next sequential number and the format established
in `docs/decisions/001-standalone-chain.md`.

Triggers that **require** an ADR:

- Choosing or rejecting a Substrate pallet or feature
- Adding or removing a pallet
- Changing the token economics model
- Altering any invariant in §4
- Changing the dependency pinning or versioning policy

## 8. Dependency policy (summary)

Full details: [`docs/decisions/002-versioning-policy.md`](docs/decisions/002-versioning-policy.md).

- FRAME/Substrate crate versions are **pinned explicitly** in `Cargo.toml`.
- No wildcard (`*`) or loose (`^`) ranges for Substrate crates.
- `spec_version` **must** be bumped before any runtime upgrade.
- `transaction_version` bumped only when extrinsic encoding changes.
- Security patches follow [`.github/agents/security-upgrade.md`](.github/agents/security-upgrade.md).

## 9. Domain model

The full problem-domain class diagram lives in
[`docs/domain-model.md`](docs/domain-model.md). Consult it when you need to
understand entity relationships, lifecycle states, or naming conventions.

## 10. Terminology

Use these terms consistently in code, comments, and documentation.

| Term | Meaning |
|---|---|
| Member | A registered on-chain participant. Not a token — a storage record. |
| Active member | A member whose status permits voting and proposal submission. |
| Community Token | The single fungible asset used within the network. |
| Treasury | The community-owned pool of Community Tokens. |
| Proposal | A formal spending request: purpose + amount + voting period. |
| Vote | A single for/against signal from an active member. Equal weight. |

## 11. Current build state

See [`docs/current-state.md`](docs/current-state.md) for live status of every
component. **Read it before writing any code.**

---

## 12. Multi-agent workflow

Two parallel workflow patterns are supported. See ADR-008 for the full
decision record. See `.codex/agents/merger.md` for the merger runbook.

### 12.1 Pattern A — Ralph loop (parallel Codex sessions)

Two Codex instances work in parallel, each in an isolated git worktree.
A third Codex session acts as merger when both are complete.

**Worktree lifecycle:**

```bash
# Create — run from the repo root, outside existing worktrees
git worktree add ../gaia.worktrees/codex-<task>-<YYYY-MM-DD> -b codex/<task>

# Work inside the worktree
cd ../gaia.worktrees/codex-<task>-<YYYY-MM-DD>

# Cleanup — only after the branch is merged
cd <repo-root>
git worktree remove ../gaia.worktrees/codex-<task>-<YYYY-MM-DD>
git branch -d codex/<task>
```

Naming: `codex-<task>-<YYYY-MM-DD>` (e.g. `codex-treasury-fee-2026-03-07`).

### 12.2 Pattern B — Orchestrator + sub-agents

A single orchestrator Codex session decomposes the task, spawns sub-agents,
and integrates their branches sequentially. The orchestrator owns the merge.

### 12.3 Parallelization boundaries

These are determined by the pallet dependency graph (§3.2):

| Work | Safe to parallelise? | Reason |
|---|---|---|
| `pallets/membership/` changes | ✅ Yes | No upstream pallet dependencies |
| `pallets/treasury/` changes | ✅ Yes | No upstream pallet dependencies |
| `pallets/proposals/` changes | ⚠️ Caution | Depends on membership + treasury traits |
| `runtime/` wiring | ❌ Serialise | Single file; high merge conflict risk |
| `docs/current-state.md` | ❌ Read-only | Parallel agents must not write this file |
| `docs/decisions/` numbered ADRs | ❌ Serialise | Number collisions — use draft/ instead |
| `tests/` integration tests | ⚠️ Caution | Cross-pallet tests may overlap |

**Rule:** if two parallel agents both touch `proposals` or `runtime/`, the
operator must explicitly task-decompose to avoid semantic conflicts.

---

## 13. Agent state in parallel sessions

`docs/current-state.md` is **read-only** for any agent running in a parallel
worktree session. Do not modify it during parallel work.

Instead, write your session progress to:

```
docs/agent-state/<branch-slug>.md
```

Required sections in your agent-state file:

```markdown
## What changed
<bullet list of functional changes>

## Build state
<result of cargo check / cargo test / cargo build>

## Open issues
<anything unresolved that the merger must handle>
```

The merger reads all `docs/agent-state/` files and updates
`docs/current-state.md` as part of integration. After a successful
merge the merger deletes the consumed agent-state files.

---

## 14. ADR protocol in parallel sessions

**Never claim a sequential ADR number directly in a parallel session.**
Number collisions between parallel branches corrupt the ADR log.

Instead:

1. Create your draft in `docs/decisions/draft/<branch-slug>-<title>.md`.
2. Use the full ADR format (see `docs/decisions/001-standalone-chain.md`).
3. Write `# ADR DRAFT — <title>` as the heading (no number yet).
4. The merger promotes drafts to numbered ADRs in sequence before opening
   the integration PR.

Single-agent sessions on `main` may claim the next sequential number
directly (no parallelism risk).

---

## 15. Git Discipline (Non-Negotiable)

- Git is backup, recovery, and audit trail. Use it continuously.
- Commit related changes together in small, reviewable slices.
- Before each commit: run the required command hierarchy (§5).
- Push each clean, meaningful commit promptly (avoid local commit pileups).
- Never end a task with relevant local-only commits not pushed.
- Never batch unrelated changes into one commit.


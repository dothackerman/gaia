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
  current-state.md                 # Live build status — read before coding
  decisions/                       # ADRs (Architecture Decision Records)
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
.claude/
  agents/
    post-build.md                  # Post-build warning-classification workflow
    security-upgrade.md            # Security dependency-upgrade runbook
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

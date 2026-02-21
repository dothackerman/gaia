# AGENTS.md — Operational context for AI coding agents

## Project purpose

Community Chain is a private, standalone Substrate solochain that provides a
member-based community with a shared on-chain treasury. Members pay annual fees
that flow into the treasury. Any active member may submit a spending proposal;
all active members vote with equal weight; an approved proposal triggers an
automatic treasury disbursement. There are no external validators, no relay
chain, and no shared-security dependency — the community is fully self-sovereign.

## Three-pallet architecture

| Pallet | Responsibility |
|---|---|
| `pallets/membership` | Maintains the authoritative set of active members. Exposes the predicate `is_active_member(AccountId) -> bool` consumed by other pallets. Handles member registration and deactivation. |
| `pallets/treasury` | Holds the community token balance. Accepts fee deposits. Executes approved disbursements. Enforces the invariant that the balance never goes negative. |
| `pallets/proposals` | Full lifecycle of a spending proposal: submission, open voting period, tally, approval/rejection, and single execution. Delegates member-validity checks to the membership pallet and fund transfers to the treasury pallet. |

## Cargo command hierarchy

Always iterate in this exact order. Never skip a level.

```
cargo check      # Fast syntax & type check. Must pass before anything else.
cargo clippy     # Lint. Fix all warnings before proceeding.
cargo test       # Run unit and integration tests.
cargo build      # Full compile. Only run when the above three are green.
```

**Rule:** `cargo check` must pass before any other command is run. If `cargo
check` fails, stop, fix the errors, and re-run `cargo check` before moving on.

## Invariants that must never be violated

1. **Treasury balance never goes negative.** Any disbursement that would reduce
   the treasury below zero must be rejected with an error, not silently clamped.
2. **Only active members can vote.** The proposals pallet must call
   `membership::is_active_member` on every vote extrinsic and return an error
   for non-members or deactivated members.
3. **A proposal can only be executed once.** Once a proposal transitions to the
   `Executed` state it must be immutable. Any subsequent execution attempt must
   return an error.

## Current build state

See [`docs/current-state.md`](docs/current-state.md) for a live, human-readable
status of every component. Read it before writing any code.

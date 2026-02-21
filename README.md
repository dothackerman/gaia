# Community Chain

A private, standalone Substrate-based blockchain built as the backbone for a member-based community.

## What is this?

Community Chain is a solochain (not a parachain) that gives a closed community a shared on-chain ledger to manage collective resources — no central authority, no shared security dependency, just the members themselves.

## Core domain concepts

**Member** — A registered participant of the community. Membership is an on-chain record, not a token. In this early prototype, initial members are hardcoded at genesis. Only active members may vote on proposals.

**Community Token** — The single fungible asset that represents value within the network. Used to fund the treasury and denominate proposal budgets.

**Treasury** — A community-owned pool of tokens funded by annual member fees. The treasury never disburses funds unless a proposal is explicitly approved. Its balance can never go negative.

**Proposal** — A formal spending request submitted by any active member. A proposal describes the purpose, the requested amount, and is subject to a collective vote.

**Vote** — Every active member has equal voting power. When a proposal reaches the required approval threshold the treasury releases the requested funds. Each proposal can only be executed once.

## Architecture overview

Three pallets carry the core logic:

| Pallet | Responsibility |
|---|---|
| `membership` | Tracks active members; gates participation in voting |
| `treasury` | Holds community funds; enforces balance invariants |
| `proposals` | Lifecycle of a spending proposal from submission to execution |

## Status

> ⚠️ **Early prototype.** No runtime code has been written yet. Node template not initialised. All pallets are stubs. See [`docs/current-state.md`](docs/current-state.md) for the live build status.

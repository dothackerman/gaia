# GAIA

A private, standalone Substrate blockchain for community self-governance.

## What is GAIA?

GAIA gives a closed community its own sovereign chain — no central authority,
no relay-chain dependency, just the members themselves. Members pay annual fees
into a shared treasury, propose how to spend those funds, and vote with equal
weight. When a proposal passes, the treasury disburses automatically.

It is a solochain, not a parachain. The community controls its own consensus,
upgrades, and governance without external dependencies.

## How it works

```
Member fees ──▸ Treasury ◂── approved proposals draw from here
                   ▲
                   │
 Proposals: submit → vote → tally → execute (once)
```

1. **Members** register on-chain. Only active members can submit proposals and
   vote.
2. **Treasury** collects fees and holds the community's funds. Its balance can
   never go negative.
3. **Proposals** let any active member request a spend. All members vote with
   equal weight. An approved proposal triggers a one-time treasury
   disbursement.

## Key concepts

| Term | Meaning |
|---|---|
| Member | An on-chain participant — a storage record, not a token |
| Community Token | The single fungible asset used for fees and proposals |
| Treasury | The community-owned pool of tokens |
| Proposal | A formal spending request subject to member vote |
| Vote | One member, one equal-weight signal (for or against) |

## Domain model

For a deeper look at the problem domain and requirements engineering, see
[`docs/domain-model.md`](docs/domain-model.md) — includes a full Mermaid class
diagram of the *Fachdomäne*.

## Project structure

| Directory | Purpose |
|---|---|
| `pallets/membership/` | Member registry — who is active |
| `pallets/treasury/` | Community funds — deposits and disbursements |
| `pallets/proposals/` | Proposal lifecycle — submit, vote, execute |
| `runtime/` | Wires pallets into a Substrate runtime |
| `node/` | Substrate node binary |
| `docs/` | Architecture decisions and build status |

## Status

> **Active scaffold.** `membership` is implemented and wired in runtime.
> `treasury` and `proposals` are scaffolded and runtime-wired, pending full implementation.
> See [`docs/current-state.md`](docs/current-state.md) for the latest detailed status.

## For AI agents

If you are an AI coding agent, read [`AGENTS.md`](AGENTS.md) before writing
any code. It contains invariants, conventions, and constraints that govern all
contributions to this repository.

Codex sessions should also load [`.codex/instructions.md`](.codex/instructions.md)
to mirror the same operating rules used by GitHub Copilot agent sessions.

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

## Tester CLI (local member UX)

The workspace now includes `gaia-tester-cli`, a human-focused local tester for
manual member flows. It is intended for one person to run through:
`submit -> vote -> tally -> execute` in one session.

### Command contract

The command surface is intentionally small:

- `persona` — list and preview seeded local personas (`Alice`, `Bob`, `Charlie`, etc.)
- `membership` — submit and vote member admission calls
- `proposal` — submit/vote/tally/execute proposal calls (`vote` uses `yes|no`)
- `treasury` — deposit fees into treasury
- `watch` — inspect proposal state and treasury balance
- `local` — local helper hints (`start`, `reset`, `refresh-metadata`)

### Fast local tester mode

For practical local manual testing, runtime voting period can be shortened with
feature `fast-local`:

```bash
cargo run -p gaia-node --features gaia-runtime/fast-local -- --dev --tmp --rpc-external --unsafe-rpc-external
```

- Default runtime behavior remains unchanged.
- `fast-local` is opt-in and intended only for local tester sessions.

### Getting started (clone to first interaction)

1. Clone and enter repo:

```bash
git clone <your-gaia-repo-url>
cd gaia
```

2. Build tester CLI:

```bash
cargo build -p gaia-tester-cli
```

3. Start local node (fast local tester mode):

```bash
cargo run -p gaia-node --features gaia-runtime/fast-local -- --dev --tmp --rpc-external --unsafe-rpc-external
```

The `--dev` preset endows seeded tester personas (`Alice` through `Ferdie`) so
membership/proposal/treasury calls can pay fees immediately.

4. In a second terminal, list personas:

```bash
cargo run -p gaia-tester-cli -- persona list
```

Expected first output:

```text
Available seeded personas:
- Alice
- Bob
- Charlie
- Dave
- Eve
- Ferdie
```

5. Preview first signer identity:

```bash
cargo run -p gaia-tester-cli -- persona preview alice
```

6. Perform first member action (example: propose Charlie):

```bash
cargo run -p gaia-tester-cli -- membership propose alice charlie
```

Next commands to discover:

```bash
cargo run -p gaia-tester-cli -- proposal --help
cargo run -p gaia-tester-cli -- watch --help
cargo run -p gaia-tester-cli -- local --help
```

### Metadata artifact refresh

The tester CLI uses a committed metadata artifact: `tester-cli/artifacts/gaia.scale`.

To refresh it after runtime changes:

1. Run local node with WS on `ws://127.0.0.1:9944` and HTTP RPC on `http://127.0.0.1:9933`.
2. Fetch and decode metadata directly into `tester-cli/artifacts/gaia.scale`:

```bash
curl -sS -H 'content-type: application/json' \
  -d '{"id":1,"jsonrpc":"2.0","method":"state_getMetadata","params":[]}' \
  http://127.0.0.1:9933 \
  | sed -n 's/.*"result":"0x\([^"]*\)".*/\1/p' \
  | xxd -r -p > tester-cli/artifacts/gaia.scale
```

3. Rebuild `gaia-tester-cli`.

## Project structure

| Directory | Purpose |
|---|---|
| `pallets/membership/` | Member registry — who is active |
| `pallets/treasury/` | Community funds — deposits and disbursements |
| `pallets/proposals/` | Proposal lifecycle — submit, vote, execute |
| `runtime/` | Wires pallets into a Substrate runtime |
| `node/` | Substrate node binary |
| `tester-cli/` | Human-focused Subxt tester CLI for local member UX |
| `docs/` | Architecture decisions and build status |

## Status

> **All three pallets implemented and tested.** `membership`, `treasury`, and `proposals`
> are fully implemented, runtime-wired, and covered by unit, integration, and runtime tests.
> See [`docs/current-state.md`](docs/current-state.md) for the latest detailed status,
> including current test counts.

## For AI agents

If you are an AI coding agent, read [`AGENTS.md`](AGENTS.md) before writing
any code. It contains invariants, conventions, and constraints that govern all
contributions to this repository.

Codex sessions should also load [`.codex/instructions.md`](.codex/instructions.md)
to mirror the same operating rules used by GitHub Copilot agent sessions.

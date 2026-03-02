# GAIA

A private, standalone Substrate blockchain for community self-governance.

## What is GAIA?

GAIA gives a closed community its own sovereign chain: members fund a shared
treasury, submit spending proposals, and vote with equal weight.

It is a solochain, not a parachain. The community controls its own consensus,
upgrades, and governance.

## Current state vs target model

### Current state (implemented now)

- Signed chain accounts and membership records are distinct concepts.
- Membership rights are vote-gated through membership proposals
  (`memberships propose` -> `memberships vote` -> `memberships finalize`).
- Membership proposals are ID-based and time-bounded (`vote_end` deadline).
- Only active members can submit proposals, vote, and finalize membership proposals.
- Any signed account can currently deposit funds into treasury.
- The local tester CLI uses seeded personas (`Alice` through `Ferdie`) for
  deterministic local testing.

### Target model (future goal)

- A signed transacting account should be equivalent to an admitted member
  account in the private-network model.
- New member accounts should only become usable after approval by existing
  members.
- Recurring fee policy (for example annual rhythm) should be protocol-enforced,
  not only social convention.

## How it works

```text
Member fees ──> Treasury <── approved proposals draw from here
                    ^
                    |
Proposals: submit -> vote -> finalize -> execute (once)
```

1. Members are admitted on-chain through peer voting.
2. Treasury holds community funds and enforces non-negative balance.
3. Proposals let active members request spends from treasury.

## Governance note (known limitation)

Current voting thresholds are intentionally simple and need hardening:

- Treasury proposals: approval is `yes > no` at finalize time (no quorum yet).
- Membership proposals: approval is `>= 80%` of a submit-time active-member
  snapshot (no quorum/turnout guard yet).
- Suspension by peers: unanimity of all other active members.

Future work should formalize quorum/turnout and more robust threshold policy.

## Domain model

See [`docs/domain-model.md`](docs/domain-model.md) for relationships and lifecycle terms.

## Tester CLI (local member UX)

The workspace includes `gaia-tester-cli`, a human-focused local tester for
manual governance flows.

### Command namespaces

- `personas` — seeded local identities (list/preview)
- `memberships` — membership proposal governance
- `proposals` — treasury spending proposal lifecycle
- `treasury` — treasury deposit actions
- `watch` — read-only state inspection
- `local` — local node/metadata helper hints

### Built-in help

```bash
cargo run -p gaia-tester-cli -- --help
cargo run -p gaia-tester-cli -- personas --help
cargo run -p gaia-tester-cli -- memberships --help
cargo run -p gaia-tester-cli -- proposals --help
cargo run -p gaia-tester-cli -- treasury --help
cargo run -p gaia-tester-cli -- watch --help
cargo run -p gaia-tester-cli -- local --help
```

### Watch list/detail UX

Read-only list/detail surfaces:

- `watch proposals [proposal_id]`
- `watch memberships [proposal_id]`
- `watch treasury`

List defaults:

- `--state active`
- `--order newest`

List options:

- `--state`:
  - proposals: `active|approved|rejected|executed|all`
  - memberships: `active|approved|rejected|all`
- `--order newest|oldest`
- `--pager` (force pager)
- `--no-pager` (disable pager)

Pager behavior:

- If stdout is a TTY, output goes through pager.
- If stdout is not a TTY (pipe/redirection), raw output is printed.
- Uses `$PAGER` when set; otherwise falls back to `less -FR`.

### Fast local tester mode

For practical local testing, run the node with shortened voting periods:

```bash
cargo run -p gaia-node --features gaia-runtime/fast-local -- --dev --tmp --rpc-external --unsafe-rpc-external
```

- `proposals` voting period: `20` blocks in fast-local, `100_800` blocks otherwise.
- `memberships` voting period: `20` blocks in fast-local, `100_800` blocks otherwise.

### Quick local flow

1. Build:

```bash
cargo build -p gaia-tester-cli
```

2. Start local node:

```bash
cargo run -p gaia-node --features gaia-runtime/fast-local -- --dev --tmp --rpc-external --unsafe-rpc-external
```

3. In a second terminal:

```bash
cargo run -p gaia-tester-cli -- personas list
cargo run -p gaia-tester-cli -- personas preview alice
```

4. Membership example:

```bash
cargo run -p gaia-tester-cli -- memberships propose alice dave
cargo run -p gaia-tester-cli -- watch memberships
cargo run -p gaia-tester-cli -- memberships vote alice 1 yes
cargo run -p gaia-tester-cli -- memberships vote bob 1 yes
cargo run -p gaia-tester-cli -- memberships vote charlie 1 yes
```

5. Treasury proposal example:

```bash
cargo run -p gaia-tester-cli -- treasury deposit alice 1000
cargo run -p gaia-tester-cli -- proposals submit alice "workshop" "fund-local-event" 10 240
cargo run -p gaia-tester-cli -- proposals vote bob 1 yes
cargo run -p gaia-tester-cli -- proposals vote charlie 1 yes
cargo run -p gaia-tester-cli -- watch proposals 1
# wait until current block > vote_end
cargo run -p gaia-tester-cli -- proposals finalize alice 1
cargo run -p gaia-tester-cli -- proposals execute alice 1
cargo run -p gaia-tester-cli -- watch treasury
```

### Metadata artifact refresh

The tester CLI uses committed metadata: `tester-cli/artifacts/gaia.scale`.

Refresh after runtime changes:

```bash
cargo run -p gaia-tester-cli --bin refresh_metadata -- ws://127.0.0.1:9944 tester-cli/artifacts/gaia.scale
```

Then rebuild:

```bash
cargo build -p gaia-tester-cli
```

## Project structure

| Directory | Purpose |
|---|---|
| `pallets/membership/` | Member registry and membership proposal governance |
| `pallets/treasury/` | Community funds: deposits and disbursements |
| `pallets/proposals/` | Treasury proposal lifecycle |
| `runtime/` | Runtime wiring and constants |
| `node/` | Substrate node binary |
| `tester-cli/` | Subxt-based local tester CLI |
| `docs/` | ADRs and current-state documentation |

## Status

All three GAIA pallets are implemented and runtime-wired. See
[`docs/current-state.md`](docs/current-state.md) for the detailed build and test state.

## For AI agents

If you are an AI coding agent, read [`AGENTS.md`](AGENTS.md) before writing
code. It defines invariants and contribution constraints.


# ADR 001 — Standalone solochain instead of parachain

## Context

The project is a private, member-based community network. The community needs
an on-chain treasury, membership records, and a proposal/voting system. The
primary question during initial architecture was whether to build as a Polkadot
or Kusama **parachain** (sharing security with the relay chain) or as a
**standalone solochain** (self-securing, no relay-chain dependency).

Parachains provide shared security — the relay chain's validator set finalises
parachain blocks, which means even a small parachain benefits from the economic
security of the entire network. The trade-off is that a parachain slot must be
acquired (via auction or crowdloan) and the runtime must conform to the
Polkadot/Kusama parachain interface (Cumulus).

## Decision

Build as a **standalone Substrate solochain** (no Cumulus, no relay chain).

## Consequences

**Positive**

- No parachain slot cost or auction process.
- Simpler runtime — no Cumulus boilerplate, no cross-chain messaging (XCM)
  surface area.
- Faster iteration: the node template compiles and runs without any relay-chain
  infrastructure.
- The community retains full governance over upgrades with no external
  dependency on relay-chain governance.

**Negative / accepted trade-offs**

- The network is self-securing. For a small private community this is a
  conscious and acceptable trade-off: the threat model does not include
  well-resourced external adversaries trying to reorg the chain.
- No native interoperability with the Polkadot ecosystem. If cross-chain
  features are required in the future, migrating to a parachain is a
  significant but tractable effort.

**Rejected alternative**

Shared security via a parachain slot was explicitly rejected as unnecessary
overhead for a private community network with a closed and trusted membership.

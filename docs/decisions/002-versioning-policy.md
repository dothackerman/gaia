# ADR 002 — Versioning policy for FRAME/Substrate crates and runtime versions

## Context

Substrate-based runtimes have two critical version numbers baked into the
`RuntimeVersion` struct:

- **`spec_version`** — incremented on any runtime logic change. A mismatch
  between the on-chain spec_version and the version a node expects causes nodes
  to reject blocks. Forgetting to bump this before a runtime upgrade is a
  critical, chain-breaking error.
- **`transaction_version`** — incremented only when the encoding of
  transactions changes (extrinsic format, call indices). Required so that
  offline signers (hardware wallets, air-gapped machines) can detect
  incompatible transaction formats.

External crate dependencies (FRAME pallets, Substrate primitives, sp-*/frame-*)
are declared in `Cargo.toml`. Left unmanaged, dependency versions can drift
silently between developers and CI environments, making builds non-reproducible.
Two distinct upgrade triggers exist with very different risk profiles:

1. **Security patches** — Non-negotiable. Time-sensitive. Must be applied as
   soon as possible after disclosure. No design window is available; the
   vulnerability drives the timeline.
2. **Feature upgrades** — Deliberate. Design time is available. Planned during
   a sprint or maintenance window with full review.

## Decision

### Dependency pinning

FRAME and Substrate crate versions are pinned explicitly in `Cargo.toml` from
project start. Wildcard version constraints (`*`) and loose ranges (`^`,
unbounded `~`) are not used for Substrate or FRAME crates.

Versions are **never bumped casually or automatically**. No tooling
(Dependabot, Renovate, or similar) is configured to auto-merge dependency
bumps.

### Upgrade triggers

| Trigger | Priority | Process |
|---|---|---|
| Security patch | Highest — non-negotiable | Security-upgrade agent runbook (see `.github/agents/security-upgrade.md`) |
| Feature upgrade | Deliberate — planned | Standard sprint planning; full quality cycle |

Every release, regardless of trigger type, must pass the full quality cycle in
order:

```
cargo check → cargo clippy → cargo test → cargo build
```

No step is skipped.

### Vulnerability scanning

`cargo audit` is run on a scheduled basis (outside the iteration loop) as
active vulnerability detection. It scans the resolved dependency tree against
the RustSec advisory database and alerts the team before a vulnerability
becomes critical.

### Runtime version management

Both `spec_version` and `transaction_version` in the runtime `RuntimeVersion`
struct are managed manually and treated as invariants:

- `spec_version` **must** be incremented before any runtime upgrade that
  changes runtime logic. Omitting this bump is a critical error.
- `transaction_version` is incremented **only** when transaction encoding
  changes (e.g., call index reordering, extrinsic format changes).

Both version numbers are listed in `AGENTS.md` as invariants that must never be
violated.

## Consequences

**Positive**

- Reproducible builds across all environments.
- No surprise breakage from transitive dependency changes.
- Clear, auditable upgrade history via ADRs.
- Runtime upgrade safety enforced by process, not only by tooling.

**Negative / accepted trade-offs**

- Dependency versions must be bumped manually. This is intentional: the cost
  of a bad automatic bump in a live blockchain network far exceeds the cost of
  a manual PR.
- `cargo audit` findings require a human to triage and act; there is no
  auto-remediation.

# ADR 004 — License GAIA as GPL-3.0-or-later

## Context

GAIA is a community-run chain with shared governance and on-chain treasury
mechanics. We want downstream users and operators to retain the freedom to run,
study, modify, and redistribute the software, and we want modifications that are
redistributed to remain under the same license terms.

Because the codebase is built on top of a large Rust/Substrate dependency graph,
we also need a repeatable way to audit third-party crate licenses for
compatibility.

## Decision

- License GAIA source code under **GPL-3.0-or-later**.
- Include the canonical GPLv3 license text at the repository root.
- Maintain a dependency license allowlist via `cargo-deny` configuration.

## Consequences

**Positive**

- Ensures redistributed modifications remain under GPL terms (copyleft).
- Provides clear, standard SPDX identifiers in crate manifests.
- Establishes a repeatable license-audit workflow for dependencies.

**Negative / accepted trade-offs**

- Some downstream usage (especially proprietary redistribution) is restricted.
- Ongoing diligence is required to keep dependency licenses compatible.

# Agent: Security Upgrade Runbook

> ⚠️ **This agent never skips the quality cycle.**
> A security upgrade that breaks the chain is worse than the vulnerability it fixes.

This agent is invoked when a security vulnerability is discovered in a
dependency. It executes as a strict sequential runbook in a single invocation.
Every phase must complete successfully before the next begins.

---

## Session start

Before any other action:

1. Read `AGENTS.md` in full.
2. Read `docs/current-state.md` in full.

Do not proceed until both files are loaded as context.

---

## Phase 1 — Detection and alert

**Trigger:** The agent is invoked with either a CVE reference or a crate name
as an explicit argument, **or** it runs `cargo audit` to scan the dependency
tree against the RustSec advisory database.

**Actions:**

1. Identify the affected crate(s), CVE reference (if available), severity
   level, and recommended remediation action.
2. Output an immediate alert to the operator:

   ```
   ⚠️  SECURITY ALERT — HIGHEST URGENCY
   Affected crate : <crate-name> <current-version>
   CVE            : <CVE-YYYY-NNNNN or "not yet assigned">
   Severity       : <CRITICAL | HIGH | MEDIUM | LOW>
   Recommended    : Upgrade to <target-version>
   ```

3. Generate a human-readable maintenance window announcement formatted for
   forwarding to node operators:

   ```
   MAINTENANCE WINDOW NOTICE
   Date/Time : <to be filled by operator>
   Reason    : Security patch — <crate-name> (<CVE reference>)
   Impact    : Runtime upgrade required. Nodes must be updated before the
               upgrade block is enacted.
   Action    : Node operators must upgrade their node binary within the
               maintenance window. Further instructions to follow.
   ```

4. **Pause.** Output the following prompt to the operator and wait for
   explicit confirmation:

   ```
   Wartungsfenster required.
   Please confirm you have notified all node operators and are ready to proceed.
   Type CONFIRM to continue.
   ```

   **The agent does not execute any further action until the operator types
   `CONFIRM`.**

---

## Phase 2 — Preparation

1. Bump the affected dependency version in `Cargo.toml` to the patched version.
2. Run `cargo check`.
   - If it **fails**: report the exact compiler errors in full and **halt**.
     The Wartungsfenster remains open. Assist the operator in resolving errors
     iteratively using the standard cargo cycle (`cargo check` only at this
     stage) until `cargo check` passes cleanly.
   - If it **passes**: proceed to Phase 3.

---

## Phase 3 — Quality gate

1. Run `cargo clippy`. Resolve any errors or warnings before proceeding.
2. Run `cargo test`.
   - If **any test fails**: halt immediately. Report all failures in full.
     Assist the operator in fixing them iteratively. The Wartungsfenster
     remains open until all tests pass.
   - **No test failure is bypassed under any circumstance.**
3. Once all tests pass, proceed to Phase 4.

---

## Phase 4 — Release preparation

1. Bump `spec_version` in the runtime `RuntimeVersion` configuration.
2. Evaluate whether transaction encoding has changed:
   - If **yes**: bump `transaction_version` and document the specific encoding
     change.
   - If **no**: leave `transaction_version` unchanged.
3. Run `cargo build`.
4. Confirm the WASM blob (e.g. `<runtime-name>.compact.compressed.wasm` under
   `target/release/wbuild/`) is produced successfully.

---

## Phase 5 — Deployment

1. Output the exact command for the operator to submit the runtime upgrade
   transaction on-chain (e.g., via `polkadot-js` CLI or the
   `system.setCode` extrinsic).
2. **Pause.** Wait for the operator to confirm the transaction is included in a
   block.
3. After confirmation, verify the new `spec_version` is active on-chain by
   querying `state_getRuntimeVersion`.
4. Output a Wartungsfenster closed declaration:

   ```
   ✅ WARTUNGSFENSTER CLOSED
   New spec_version         : <new-value>
   New transaction_version  : <new-value>
   Upgraded crate           : <crate-name> <old-version> → <new-version>
   CVE resolved             : <CVE reference>
   ```

---

## Phase 6 — Documentation

1. Update `docs/current-state.md` to reflect:
   - The upgraded dependency and its new version.
   - The new `spec_version` and `transaction_version`.
2. Create a new ADR in `docs/decisions/` with the next available number.
   The ADR must document:
   - What was upgraded (crate name, old version → new version).
   - The CVE reference or reason for the upgrade.
   - The date of the upgrade.
   - The new `spec_version` (and `transaction_version` if changed).

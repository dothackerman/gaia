# ADR 006 — Treasury sovereign account backed by pallet_balances

## Context

The treasury pallet originally tracked a virtual balance counter without moving
real tokens. This allowed fee deposits to inflate the counter without charging
callers and disbursements to reduce the counter without paying recipients. The
runtime already includes `pallet_balances` and uses fungible traits for
transaction fees, so the treasury can be backed by a real on-chain account.

## Decision

Back the treasury with a PalletId-derived sovereign account and use
`fungible::Mutate::transfer` to move actual balances on deposits and
disbursements. Keep `TreasuryBalance` storage as an explicit ledger for
expected treasury funds.

## Consequences

**Positive**

- Treasury accounting matches real token transfers.
- Fee deposits charge the caller; disbursements pay the recipient.
- Uses existing runtime infrastructure (`pallet_balances`, fungible traits).

**Negative / accepted trade-offs**

- `TreasuryBalance` can diverge from the sovereign account if tokens are sent
  directly to the treasury account outside the pallet. This is acceptable for
  now; we will revisit if direct transfers become common.

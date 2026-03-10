# Domain model

> For anyone who wants to go deeper into the requirements engineering behind
> GAIA. This diagram captures the full *Fachdomäne* (problem domain) — the
> real-world concepts the system models, their relationships, and the
> design choices that simplify the on-chain implementation.

## Class diagram

```mermaid
---
title: Community Network — Fachdomäne
---
classDiagram
    direction TB

    class Community {
        name
        memberFee
        feeRhythm
        votingThreshold
    }

    class Member {
        status: active | suspended
        joinedAt
        votingPower: always 1
    }

    class Treasury {
        balance
        currency: CommunityToken
    }

    class CommunityToken {
        <<value unit>>
        represents real-world money
        fungible
    }

    class Proposal {
        title
        description
        class: standard | governance | constitutional
        action
        submittedAt
        approvedAt?
        voteEnd
        status: active | approved | rejected | executed
    }

    class Vote {
        signal: yes | no
        castBy: Member
    }

    class MemberFeePayment {
        amount
        period
        paidBy: Member
    }

    Community "1" --> "many" Member : has
    Community "1" --> "1" Treasury : governs
    Member "many" --> "many" Proposal : votes on
    Member "1" --> "many" Proposal : authors
    Member "1" --> "many" Vote : casts
    Vote "many" --> "1" Proposal : decides
    Treasury "1" --> "many" Proposal : funds
    Treasury "1" --> "1" CommunityToken : denominated in
    MemberFeePayment "many" --> "1" Treasury : flows into
    Member "1" --> "many" MemberFeePayment : obligated to

    note for Member "Membership record IS proof of voting rights.\n No separate governance token needed."
    note for Proposal "A data record with lifecycle.\nNot a token."
    note for CommunityToken "Only true token in the system.\nAll other concepts are records."
```

## Reading guide

| Concept | Key insight |
|---|---|
| **Community** | The root aggregate — owns the member set, treasury, and governance parameters. |
| **Member** | A storage record, not a token. The record itself *is* the proof of voting rights. |
| **CommunityToken** | The only true token in the system. Everything else (members, proposals, votes) is a data record. |
| **Treasury** | Always denominated in CommunityToken. Balance invariant: never negative. |
| **Proposal** | A lifecycle entity (active → approved/rejected → executed) carrying a typed governance action. Never a token. |
| **Vote** | One signal per member per proposal. Equal weight — no quadratic or stake-weighted voting. |
| **MemberFeePayment** | The funding mechanism: member fees flow into the treasury, proposals flow out. |

## Design decisions reflected here

- **No governance token.** Voting power derives from active membership status,
  not from token holdings. This is a deliberate simplification that keeps the
  system egalitarian.
- **Single currency.** CommunityToken is the only fungible asset. There is no
  secondary staking or reward token.
- **Records over tokens.** Members, proposals, and votes are storage records
  with lifecycle states — not NFTs, not transferable assets.
- **Typed governance actions.** Proposals are no longer only treasury-withdrawal
  records; they can also govern thresholds, voting periods, execution delay,
  membership parameters, and runtime upgrades.
- **Implementation simplifications vs. domain.** The domain describes a
  `pending` member status and `draft`/`disputed` proposal states as aspirational
  concepts. The current implementation omits them: candidates awaiting admission
  move through explicit membership proposals, and proposals are submitted
  directly into the `Active` state with no draft or dispute phase.

use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_runtime::{AccountId, Balances, Proposals, Runtime, RuntimeOrigin, Treasury};
use sp_runtime::AccountId32;

// ---------------------------------------------------------------------------
// Full lifecycle
// ---------------------------------------------------------------------------

#[test]
fn full_proposal_lifecycle() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        let before_treasury =
            Balances::free_balance(gaia_treasury::Pallet::<Runtime>::account_id());
        let before_organizer = Balances::free_balance(alice());
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(
            Balances::free_balance(gaia_treasury::Pallet::<Runtime>::account_id()),
            before_treasury - 100
        );
        assert_eq!(Balances::free_balance(alice()), before_organizer + 100);
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Executed
        );
    });
}

// ---------------------------------------------------------------------------
// submit_proposal
// ---------------------------------------------------------------------------

#[test]
fn non_member_cannot_submit_proposal() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::submit_proposal(
                RuntimeOrigin::signed(dave()),
                bounded_title(b"t"),
                bounded_desc(b"d"),
                gaia_proposals::pallet::ProposalClass::Standard,
                gaia_proposals::pallet::GovernanceAction::DisburseToAccount {
                    recipient: dave(),
                    amount: 100,
                }
            ),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
    });
}

// ---------------------------------------------------------------------------
// vote_on_proposal
// ---------------------------------------------------------------------------

#[test]
fn double_vote_on_proposal_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(alice()), id, true),
            gaia_proposals::Error::<Runtime>::AlreadyVoted
        );
    });
}

#[test]
fn vote_after_window_closes_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        advance_past_voting_period();
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(alice()), id, true),
            gaia_proposals::Error::<Runtime>::VotingClosed
        );
    });
}

// ---------------------------------------------------------------------------
// tally_proposal
// ---------------------------------------------------------------------------

#[test]
fn proposal_rejected_when_no_majority() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            false
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Rejected
        );
    });
}

#[test]
fn standard_class_tie_vote_is_approved() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            false
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(charlie()),
            id
        ));
        // Standard default threshold is 1/2, so tie passes.
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

#[test]
fn tally_fails_while_voting_open() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_noop!(
            Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::VotingStillOpen
        );
    });
}

#[test]
fn tally_fails_for_already_tallied_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_noop!(
            Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalNotActive
        );
    });
}

// ---------------------------------------------------------------------------
// execute_proposal
// ---------------------------------------------------------------------------

#[test]
fn execute_allowed_for_non_organizer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_ok!(Proposals::execute_proposal(RuntimeOrigin::signed(bob()), id));
    });
}

#[test]
fn execute_fails_for_rejected_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            false
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalNotApproved
        );
    });
}

#[test]
fn execute_fails_when_treasury_insufficient() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_treasury::Error::<Runtime>::InsufficientFunds
        );
        // Status must remain Approved — not Executed
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

#[test]
fn double_execution_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        // I-3: second execution must fail
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalAlreadyExecuted
        );
    });
}

#[test]
fn execute_active_proposal_fails() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalNotApproved
        );
    });
}

#[test]
fn execute_proposal_respects_configured_execution_delay() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(alice()),
            bounded_title(b"Set delay"),
            bounded_desc(b"Govern delay"),
            gaia_proposals::pallet::ProposalClass::Governance,
            gaia_proposals::pallet::GovernanceAction::SetExecutionDelay { blocks: 3 }
        ));
        let set_delay_id = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        for voter in [alice(), bob()] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                set_delay_id,
                true
            ));
        }
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            set_delay_id
        ));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            set_delay_id
        ));
        assert_eq!(gaia_proposals::ExecutionDelay::<Runtime>::get(), 3);

        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let payout_id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            payout_id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            payout_id,
            true
        ));
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            payout_id
        ));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), payout_id),
            gaia_proposals::Error::<Runtime>::ExecutionTooEarly
        );
    });
}

#[test]
fn governance_class_proposal_end_to_end() {
    new_test_ext_with_members(&[
        (alice(), b"Alice"),
        (bob(), b"Bob"),
        (charlie(), b"Charlie"),
        (dave(), b"Dave"),
        (eve(), b"Eve"),
    ])
    .execute_with(|| {
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(alice()),
            bounded_title(b"Set voting period"),
            bounded_desc(b"Governance change"),
            gaia_proposals::pallet::ProposalClass::Governance,
            gaia_proposals::pallet::GovernanceAction::SetProposalVotingPeriod { blocks: 77 },
        ));
        let id = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        for voter in [alice(), bob(), charlie(), dave()] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                true
            ));
        }
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(eve()),
            id,
            false
        ));

        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(gaia_proposals::ProposalVotingPeriod::<Runtime>::get(), 77);
    });
}

#[test]
fn constitutional_class_threshold_change_end_to_end() {
    let m7 = synthetic_member(7);
    let m8 = synthetic_member(8);
    let m9 = synthetic_member(9);
    let m10 = synthetic_member(10);
    new_test_ext_with_members(&[
        (alice(), b"Alice"),
        (bob(), b"Bob"),
        (charlie(), b"Charlie"),
        (dave(), b"Dave"),
        (eve(), b"Eve"),
        (ferdie(), b"Ferdie"),
        (m7.clone(), b"M7"),
        (m8.clone(), b"M8"),
        (m9.clone(), b"M9"),
        (m10.clone(), b"M10"),
    ])
    .execute_with(|| {
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(alice()),
            bounded_title(b"Set constitutional threshold"),
            bounded_desc(b"Constitutional change"),
            gaia_proposals::pallet::ProposalClass::Constitutional,
            gaia_proposals::pallet::GovernanceAction::SetConstitutionalApprovalThreshold {
                numerator: 19,
                denominator: 20,
            },
        ));
        let id = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        for voter in [
            alice(),
            bob(),
            charlie(),
            dave(),
            eve(),
            ferdie(),
            m7.clone(),
            m8.clone(),
            m9.clone(),
            m10.clone(),
        ] {
            let yes = voter != m10;
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                yes
            ));
        }
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(gaia_proposals::ConstitutionalApprovalNumerator::<Runtime>::get(), 19);
        assert_eq!(
            gaia_proposals::ConstitutionalApprovalDenominator::<Runtime>::get(),
            20
        );

        // 8/10 should fail once threshold is 95%.
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(alice()),
            bounded_title(b"Second constitutional change"),
            bounded_desc(b"Should fail"),
            gaia_proposals::pallet::ProposalClass::Constitutional,
            gaia_proposals::pallet::GovernanceAction::SetConstitutionalApprovalThreshold {
                numerator: 9,
                denominator: 10,
            },
        ));
        let id2 = gaia_proposals::pallet::ProposalCount::<Runtime>::get();
        for voter in [
            alice(),
            bob(),
            charlie(),
            dave(),
            eve(),
            ferdie(),
            m7.clone(),
            m8.clone(),
            m9.clone(),
            m10.clone(),
        ] {
            let yes = voter != m10 && voter != m9;
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id2,
                yes
            ));
        }
        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id2));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id2)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Rejected
        );
    });
}

fn synthetic_member(seed: u8) -> AccountId {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    AccountId32::new(bytes).into()
}

// ---------------------------------------------------------------------------
// Edge cases: concurrent proposals
// ---------------------------------------------------------------------------

/// Two proposals submitted by different organisers proceed independently.
/// Approve one and reject the other — they must not interfere.
#[test]
fn concurrent_proposals_independent_voting() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 1_000));

        // Alice submits proposal 1
        let id1 = submit_default_proposal();
        // Bob submits proposal 2
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(bob()),
            bounded_title(b"Second"),
            bounded_desc(b"Another proposal"),
            gaia_proposals::pallet::ProposalClass::Standard,
            gaia_proposals::pallet::GovernanceAction::DisburseToAccount {
                recipient: bob(),
                amount: 200,
            }
        ));
        let id2 = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        // Vote to approve proposal 1
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id1,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id1,
            true
        ));

        // Vote to reject proposal 2
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id2,
            false
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id2,
            false
        ));

        advance_past_voting_period();

        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id1
        ));
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id2
        ));

        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id1)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id2)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Rejected
        );
    });
}

/// Two approved proposals compete for the same treasury funds.
/// The first execution succeeds; the second must fail with
/// `InsufficientFunds`, confirming I-1 under contention.
#[test]
fn concurrent_proposals_exhaust_treasury() {
    new_test_ext().execute_with(|| {
        // Fund treasury with exactly enough for ONE proposal (150 tokens).
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 150));

        // Proposal 1: requests 100
        let id1 = submit_default_proposal();
        // Proposal 2: requests 100
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(bob()),
            bounded_title(b"Second"),
            bounded_desc(b"Another"),
            gaia_proposals::pallet::ProposalClass::Standard,
            gaia_proposals::pallet::GovernanceAction::DisburseToAccount {
                recipient: bob(),
                amount: 100,
            }
        ));
        let id2 = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        // Approve both
        for id in [id1, id2] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(alice()),
                id,
                true
            ));
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(bob()),
                id,
                true
            ));
        }

        advance_past_voting_period();

        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id1
        ));
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id2
        ));

        // Execute first — succeeds (treasury: 150 → 50)
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id1
        ));

        // Execute second — fails (treasury: 50 < 100)
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(bob()), id2),
            gaia_treasury::Error::<Runtime>::InsufficientFunds
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases: zero-amount proposal
// ---------------------------------------------------------------------------

/// A proposal requesting 0 tokens goes through submission, voting, and
/// tally, but execution fails because the treasury rejects zero-amount
/// disbursements. This documents the current behaviour.
#[test]
fn zero_amount_proposal_execution_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(alice()),
            bounded_title(b"Free event"),
            bounded_desc(b"No cost"),
            gaia_proposals::pallet::ProposalClass::Standard,
            gaia_proposals::pallet::GovernanceAction::DisburseToAccount {
                recipient: alice(),
                amount: 0,
            }
        ));
        let id = gaia_proposals::pallet::ProposalCount::<Runtime>::get();

        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));

        advance_past_voting_period();

        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));

        // Execution fails — treasury rejects zero-amount disbursements.
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_treasury::Error::<Runtime>::ZeroAmount
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases: majority boundary
// ---------------------------------------------------------------------------

/// With 3 voters, 2 yes + 1 no is the minimum winning margin.
/// Confirms strict `yes > no` majority.
#[test]
fn proposal_approved_at_exact_majority() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();

        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(charlie()),
            id,
            false
        ));

        advance_past_voting_period();

        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases: tally by non-member
// ---------------------------------------------------------------------------

/// `tally_proposal` only requires a signed origin — no membership check.
/// A non-member (Dave) can trigger tally after the voting window closes.
#[test]
fn non_member_can_tally_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));

        advance_past_voting_period();

        // Dave is not a member, yet tally succeeds.
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(dave()),
            id
        ));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases: vote storage persists after execution
// ---------------------------------------------------------------------------

/// Proposal vote storage (`ProposalVotes`, `ProposalYesCount`,
/// `ProposalNoCount`) is NOT cleaned up after execution. This test
/// documents that behaviour so any future cleanup refactor trips this
/// regression guard.
#[test]
fn vote_storage_persists_after_execution() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();

        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            false
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(charlie()),
            id,
            true
        ));

        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));

        // Vote records still present.
        assert!(gaia_proposals::pallet::ProposalVotes::<Runtime>::contains_key(id, alice()));
        assert!(gaia_proposals::pallet::ProposalVotes::<Runtime>::contains_key(id, bob()));
        assert!(gaia_proposals::pallet::ProposalVotes::<Runtime>::contains_key(id, charlie()));
        assert_eq!(gaia_proposals::pallet::ProposalYesCount::<Runtime>::get(id), 2);
        assert_eq!(gaia_proposals::pallet::ProposalNoCount::<Runtime>::get(id), 1);
    });
}

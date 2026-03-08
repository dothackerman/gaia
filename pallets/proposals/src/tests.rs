use crate::mock::*;
use crate::pallet::{
    ConstitutionalApprovalDenominator,
    ConstitutionalApprovalNumerator,
    ExecutionDelay,
    GovernanceApprovalDenominator,
    GovernanceApprovalNumerator,
    ProposalNoCount,
    ProposalStatus,
    ProposalVotes,
    ProposalVotingPeriod,
    ProposalYesCount,
    // Aliased to avoid shadowing the pallet type alias `Proposals` from mock.
    Proposals as ProposalStorage,
    StandardApprovalDenominator,
    StandardApprovalNumerator,
};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn bounded_title(s: &[u8]) -> sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<128>> {
    sp_runtime::BoundedVec::try_from(s.to_vec()).expect("title within bounds")
}

fn bounded_desc(s: &[u8]) -> sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<1024>> {
    sp_runtime::BoundedVec::try_from(s.to_vec()).expect("description within bounds")
}

/// Submit a default proposal from ALICE and return its id.
fn submit_default(origin: u64) -> u32 {
    assert_ok!(Proposals::submit_proposal(
        RuntimeOrigin::signed(origin),
        bounded_title(b"Fund the festival"),
        bounded_desc(b"A community festival needs funding."),
        100u64,
        50u64,
    ));
    crate::pallet::ProposalCount::<Test>::get()
}

/// Advance chain to a block number past the voting window.
fn advance_past_voting() {
    // current block is 1 after genesis.
    let period = ProposalVotingPeriod::<Test>::get();
    System::set_block_number(period + 2);
}

// ---------------------------------------------------------------------------
// Governance parameter setters
// ---------------------------------------------------------------------------

#[test]
fn set_proposal_voting_period_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_proposal_voting_period(
            RuntimeOrigin::root(),
            42
        ));
        assert_eq!(ProposalVotingPeriod::<Test>::get(), 42);
        System::assert_last_event(Event::ProposalVotingPeriodSet { blocks: 42 }.into());
    });
}

#[test]
fn set_execution_delay_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_execution_delay(RuntimeOrigin::root(), 7));
        assert_eq!(ExecutionDelay::<Test>::get(), 7);
        System::assert_last_event(Event::ExecutionDelaySet { blocks: 7 }.into());
    });
}

#[test]
fn set_standard_threshold_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_standard_approval_threshold(
            RuntimeOrigin::root(),
            3,
            4
        ));
        assert_eq!(StandardApprovalNumerator::<Test>::get(), 3);
        assert_eq!(StandardApprovalDenominator::<Test>::get(), 4);

        assert_ok!(Proposals::set_governance_approval_threshold(
            RuntimeOrigin::root(),
            4,
            5
        ));
        assert_eq!(GovernanceApprovalNumerator::<Test>::get(), 4);
        assert_eq!(GovernanceApprovalDenominator::<Test>::get(), 5);

        assert_ok!(Proposals::set_constitutional_approval_threshold(
            RuntimeOrigin::root(),
            9,
            10
        ));
        assert_eq!(ConstitutionalApprovalNumerator::<Test>::get(), 9);
        assert_eq!(ConstitutionalApprovalDenominator::<Test>::get(), 10);
    });
}

#[test]
fn set_threshold_rejects_zero_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_standard_approval_threshold(RuntimeOrigin::root(), 1, 0),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn set_threshold_rejects_numerator_greater_than_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_standard_approval_threshold(RuntimeOrigin::root(), 3, 2),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn non_root_cannot_call_setters() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_proposal_voting_period(RuntimeOrigin::signed(ALICE), 55),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Proposals::set_execution_delay(RuntimeOrigin::signed(ALICE), 2),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Proposals::set_standard_approval_threshold(RuntimeOrigin::signed(ALICE), 1, 2),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Proposals::set_governance_approval_threshold(RuntimeOrigin::signed(ALICE), 4, 5),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Proposals::set_constitutional_approval_threshold(RuntimeOrigin::signed(ALICE), 9, 10),
            DispatchError::BadOrigin
        );
    });
}

// ---------------------------------------------------------------------------
// submit_proposal — happy path
// ---------------------------------------------------------------------------

#[test]
fn submit_proposal_succeeds_for_active_member() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_eq!(id, 1);
        assert!(ProposalStorage::<Test>::contains_key(1));
        System::assert_last_event(
            Event::ProposalSubmitted {
                proposal_id: 1,
                organizer: ALICE,
            }
            .into(),
        );
    });
}

// ---------------------------------------------------------------------------
// submit_proposal — failure paths
// ---------------------------------------------------------------------------

#[test]
fn submit_proposal_fails_for_non_member() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::submit_proposal(
                RuntimeOrigin::signed(DAVE),
                bounded_title(b"Festival"),
                bounded_desc(b"desc"),
                50u64,
                10u64,
            ),
            Error::<Test>::NotActiveMember
        );
    });
}

// ---------------------------------------------------------------------------
// vote_on_proposal — happy path
// ---------------------------------------------------------------------------

#[test]
fn vote_on_proposal_records_vote() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert!(ProposalVotes::<Test>::contains_key(id, ALICE));
        assert_eq!(ProposalYesCount::<Test>::get(id), 1);
        assert_eq!(ProposalNoCount::<Test>::get(id), 0);
        System::assert_last_event(
            Event::VoteCast {
                proposal_id: id,
                voter: ALICE,
                approve: true,
            }
            .into(),
        );
    });
}

#[test]
fn no_vote_increments_no_count() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            false
        ));
        assert_eq!(ProposalYesCount::<Test>::get(id), 0);
        assert_eq!(ProposalNoCount::<Test>::get(id), 1);
    });
}

// ---------------------------------------------------------------------------
// vote_on_proposal — failure paths
// ---------------------------------------------------------------------------

#[test]
fn vote_rejects_non_active_member() {
    // I-2: only active members may vote.
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(DAVE), id, true),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn vote_fails_for_double_vote() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(ALICE), id, true),
            Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn vote_fails_after_voting_period_closed() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        advance_past_voting();
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(ALICE), id, true),
            Error::<Test>::VotingClosed
        );
    });
}

// ---------------------------------------------------------------------------
// tally_proposal — happy paths
// ---------------------------------------------------------------------------

#[test]
fn tally_approves_with_yes_majority() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(CHARLIE),
            id,
            false
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        let proposal = ProposalStorage::<Test>::get(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved);
        System::assert_last_event(Event::ProposalApproved { proposal_id: id }.into());
    });
}

#[test]
fn tally_rejects_with_no_majority() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            false
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            false
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(CHARLIE),
            id,
            true
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        let proposal = ProposalStorage::<Test>::get(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Rejected);
        System::assert_last_event(Event::ProposalRejected { proposal_id: id }.into());
    });
}

// ---------------------------------------------------------------------------
// tally_proposal — failure paths
// ---------------------------------------------------------------------------

#[test]
fn tally_fails_while_voting_still_open() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        // Voting window is open — block 1, vote_end = 11.
        assert_noop!(
            Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::VotingStillOpen
        );
    });
}

// ---------------------------------------------------------------------------
// execute_proposal — happy path
// ---------------------------------------------------------------------------

#[test]
fn execute_proposal_succeeds_for_approved() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            true
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            id
        ));
        let proposal = ProposalStorage::<Test>::get(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Executed);
        System::assert_last_event(
            Event::ProposalExecuted {
                proposal_id: id,
                organizer: ALICE,
                amount: 100,
            }
            .into(),
        );
    });
}

// ---------------------------------------------------------------------------
// execute_proposal — failure paths
// ---------------------------------------------------------------------------

#[test]
fn execute_fails_for_non_organizer() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            true
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(BOB), id),
            Error::<Test>::NotOrganizer
        );
    });
}

#[test]
fn execute_fails_for_non_approved_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        // Proposal is still Active.
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::ProposalNotApproved
        );
    });
}

#[test]
fn execute_fails_for_already_executed_proposal() {
    // I-3: a proposal must execute at most once.
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            true
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(ALICE),
            id
        ));
        // Second execution must fail.
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::ProposalAlreadyExecuted
        );
    });
}

#[test]
fn execute_propagates_treasury_error() {
    new_test_ext().execute_with(|| {
        let id = submit_default(ALICE);
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(ALICE),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(BOB),
            id,
            true
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        MockTreasury::set_fail(true);
        assert!(Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id).is_err());
        // Status must remain Approved — not Executed.
        let proposal = ProposalStorage::<Test>::get(id).unwrap();
        assert_eq!(proposal.status, ProposalStatus::Approved);
    });
}

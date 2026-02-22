use crate::mock::*;
use crate::pallet::{
    ProposalNoCount,
    ProposalStatus,
    ProposalVotes,
    ProposalYesCount,
    // Aliased to avoid shadowing the pallet type alias `Proposals` from mock.
    Proposals as ProposalStorage,
};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};

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
    // VotingPeriod = 10 blocks; current block is 1 after genesis.
    System::set_block_number(12);
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

use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_runtime::{Balances, Proposals, Runtime, RuntimeOrigin, Treasury};

fn b(s: &[u8]) -> sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<128>> {
    sp_runtime::BoundedVec::try_from(s.to_vec()).unwrap()
}
fn d(s: &[u8]) -> sp_runtime::BoundedVec<u8, frame_support::traits::ConstU32<1024>> {
    sp_runtime::BoundedVec::try_from(s.to_vec()).unwrap()
}

fn submit_default() -> u32 {
    assert_ok!(Proposals::submit_proposal(
        RuntimeOrigin::signed(alice()),
        b(b"t"),
        d(b"d"),
        100,
        10
    ));
    gaia_proposals::pallet::ProposalCount::<Runtime>::get()
}

// ---------------------------------------------------------------------------
// Full lifecycle
// ---------------------------------------------------------------------------

#[test]
fn full_proposal_lifecycle() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default();
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
        advance_blocks(100_801);
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
                b(b"t"),
                d(b"d"),
                100,
                10
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
        let id = submit_default();
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
        let id = submit_default();
        advance_blocks(100_801);
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
        let id = submit_default();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            false
        ));
        advance_blocks(100_801);
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
fn tally_rejects_when_tie_vote() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
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
        advance_blocks(100_801);
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(charlie()),
            id
        ));
        // yes == no → Rejected (simple majority requires yes > no)
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Rejected
        );
    });
}

#[test]
fn tally_fails_while_voting_open() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
        assert_noop!(
            Proposals::tally_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::VotingStillOpen
        );
    });
}

#[test]
fn tally_fails_for_already_tallied_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));
        advance_blocks(100_801);
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
fn execute_restricted_to_organizer() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default();
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
        advance_blocks(100_801);
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(bob()), id),
            gaia_proposals::Error::<Runtime>::NotOrganizer
        );
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
    });
}

#[test]
fn execute_fails_for_rejected_proposal() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            false
        ));
        advance_blocks(100_801);
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
        let id = submit_default();
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
        advance_blocks(100_801);
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
        let id = submit_default();
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
        advance_blocks(100_801);
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
        let id = submit_default();
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalNotApproved
        );
    });
}

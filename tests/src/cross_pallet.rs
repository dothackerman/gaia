use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_runtime::{Membership, Proposals, Runtime, RuntimeOrigin, Treasury};

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
// I-2: Only active members vote
// ---------------------------------------------------------------------------

#[test]
fn only_active_members_can_vote_on_proposals() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
        // Non-member cannot vote
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(dave()), id, true),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
        // Suspended member cannot vote
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(bob())));
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(bob()), id, true),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
    });
}

// ---------------------------------------------------------------------------
// I-3: A proposal executes at most once
// ---------------------------------------------------------------------------

#[test]
fn proposal_executes_at_most_once() {
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
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_proposals::Error::<Runtime>::ProposalAlreadyExecuted
        );
    });
}

// ---------------------------------------------------------------------------
// Suspension ↔ proposals interaction
// ---------------------------------------------------------------------------

#[test]
fn suspended_member_cannot_submit_proposal() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Proposals::submit_proposal(RuntimeOrigin::signed(alice()), b(b"t"), d(b"d"), 100, 10),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
    });
}

#[test]
fn suspension_during_voting_period() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default();
        // Bob votes yes then suspends — vote already cast, still counts
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            true
        ));
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(bob())));
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
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
    });
}

// ---------------------------------------------------------------------------
// Newly admitted member interacts with proposals
// ---------------------------------------------------------------------------

#[test]
fn newly_admitted_member_can_vote_on_proposals() {
    new_test_ext().execute_with(|| {
        // Admit dave
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            dave(),
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(bob()),
            dave(),
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(charlie()),
            dave(),
            true
        ));

        // Dave is now a member — can vote on a proposal
        let id = submit_default();
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(dave()),
            id,
            true
        ));
    });
}

#[test]
fn newly_admitted_member_can_submit_proposal() {
    new_test_ext().execute_with(|| {
        // Admit dave
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            dave(),
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(bob()),
            dave(),
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(charlie()),
            dave(),
            true
        ));

        // Dave submits a proposal
        assert_ok!(Proposals::submit_proposal(
            RuntimeOrigin::signed(dave()),
            b(b"Dave's idea"),
            d(b"Dave proposes something"),
            50,
            20
        ));
    });
}

// ---------------------------------------------------------------------------
// I-1: Treasury balance ≥ 0 (cross-pallet via proposal execution)
// ---------------------------------------------------------------------------

#[test]
fn treasury_balance_never_goes_negative_via_proposal() {
    new_test_ext().execute_with(|| {
        // Fund treasury with less than the proposal amount
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 50));
        let id = submit_default(); // requests 100
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
        // Execution fails — treasury only has 50, proposal needs 100
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(alice()), id),
            gaia_treasury::Error::<Runtime>::InsufficientFunds
        );
        // Treasury balance is unchanged
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            50
        );
    });
}

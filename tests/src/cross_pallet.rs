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

#[test]
fn only_active_members_can_vote_on_proposals() {
    new_test_ext().execute_with(|| {
        let id = submit_default();
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(dave()), id, true),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(bob())));
        assert_noop!(
            Proposals::vote_on_proposal(RuntimeOrigin::signed(bob()), id, true),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
    });
}

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

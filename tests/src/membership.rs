use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_membership::pallet::{ActiveMemberCount, MemberStatus, Members};
use gaia_runtime::{Membership, Proposals, RuntimeOrigin};

fn bounded_name(
    name: &[u8],
) -> sp_runtime::BoundedVec<
    u8,
    frame_support::traits::ConstU32<{ gaia_membership::pallet::MAX_NAME_LEN }>,
> {
    sp_runtime::BoundedVec::try_from(name.to_vec()).expect("name fits")
}

#[test]
fn propose_and_admit_new_member() {
    new_test_ext().execute_with(|| {
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
        assert_eq!(
            Members::<gaia_runtime::Runtime>::get(dave())
                .unwrap()
                .status,
            MemberStatus::Active
        );
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 4);
    });
}

#[test]
fn self_suspension_decrements_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_eq!(
            Members::<gaia_runtime::Runtime>::get(alice())
                .unwrap()
                .status,
            MemberStatus::Suspended
        );
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 2);
        assert_noop!(
            Proposals::submit_proposal(
                RuntimeOrigin::signed(alice()),
                bounded_name(b"x"),
                bounded_name(b"y"),
                10,
                10
            ),
            gaia_proposals::Error::<gaia_runtime::Runtime>::NotActiveMember
        );
    });
}

#[test]
fn peer_vote_suspension_requires_unanimity() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(bob()),
            alice(),
            true
        ));
        assert_eq!(
            Members::<gaia_runtime::Runtime>::get(alice())
                .unwrap()
                .status,
            MemberStatus::Active
        );
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(charlie()),
            alice(),
            true
        ));
        assert_eq!(
            Members::<gaia_runtime::Runtime>::get(alice())
                .unwrap()
                .status,
            MemberStatus::Suspended
        );
    });
}

#[test]
fn suspended_member_cannot_vote_on_candidates() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(bob()),
            dave(),
            bounded_name(b"Dave")
        ));
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(alice()), dave(), true),
            gaia_membership::Error::<gaia_runtime::Runtime>::MemberSuspended
        );
    });
}

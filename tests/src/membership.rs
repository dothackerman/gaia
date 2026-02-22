use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_membership::pallet::{ActiveMemberCount, MemberStatus, Members};
use gaia_runtime::{Membership, RuntimeOrigin};

// ---------------------------------------------------------------------------
// Genesis
// ---------------------------------------------------------------------------

#[test]
fn genesis_seeds_three_active_members() {
    new_test_ext().execute_with(|| {
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 3);
        for who in [alice(), bob(), charlie()] {
            let record = Members::<gaia_runtime::Runtime>::get(&who).expect("member exists");
            assert_eq!(record.status, MemberStatus::Active);
        }
        assert!(!Members::<gaia_runtime::Runtime>::contains_key(dave()));
    });
}

// ---------------------------------------------------------------------------
// propose_member + vote_on_candidate
// ---------------------------------------------------------------------------

#[test]
fn propose_and_admit_new_member() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        // 3 active → 80% threshold → need 3 approvals
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
fn candidate_not_approved_below_threshold() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        // 2/3 approve → 66%, below 80%
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
            false
        ));
        assert!(!Members::<gaia_runtime::Runtime>::contains_key(dave()));
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 3);
    });
}

#[test]
fn non_member_cannot_propose_candidate() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(
                RuntimeOrigin::signed(dave()),
                eve(),
                bounded_name(b"Eve")
            ),
            gaia_membership::Error::<gaia_runtime::Runtime>::NotActiveMember
        );
    });
}

#[test]
fn propose_existing_member_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(
                RuntimeOrigin::signed(alice()),
                bob(),
                bounded_name(b"Bob")
            ),
            gaia_membership::Error::<gaia_runtime::Runtime>::AlreadyMember
        );
    });
}

#[test]
fn duplicate_candidate_proposal_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        assert_noop!(
            Membership::propose_member(
                RuntimeOrigin::signed(bob()),
                dave(),
                bounded_name(b"Dave")
            ),
            gaia_membership::Error::<gaia_runtime::Runtime>::CandidateAlreadyProposed
        );
    });
}

#[test]
fn double_vote_on_candidate_fails() {
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
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(alice()), dave(), true),
            gaia_membership::Error::<gaia_runtime::Runtime>::AlreadyVoted
        );
    });
}

// ---------------------------------------------------------------------------
// Suspension
// ---------------------------------------------------------------------------

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
    });
}

#[test]
fn double_self_suspension_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(alice())),
            gaia_membership::Error::<gaia_runtime::Runtime>::AlreadySuspended
        );
    });
}

#[test]
fn non_member_cannot_self_suspend() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(dave())),
            gaia_membership::Error::<gaia_runtime::Runtime>::NotActiveMember
        );
    });
}

#[test]
fn peer_vote_suspension_requires_unanimity() {
    new_test_ext().execute_with(|| {
        // One vote is not enough
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
        // Second vote meets unanimity (all others = 2)
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
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 2);
    });
}

#[test]
fn cannot_cast_suspension_vote_against_self() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::vote_suspend_member(RuntimeOrigin::signed(alice()), alice(), true),
            gaia_membership::Error::<gaia_runtime::Runtime>::CannotSuspendSelf
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

#[test]
fn suspended_member_cannot_propose_candidate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::propose_member(
                RuntimeOrigin::signed(alice()),
                dave(),
                bounded_name(b"Dave")
            ),
            gaia_membership::Error::<gaia_runtime::Runtime>::MemberSuspended
        );
    });
}

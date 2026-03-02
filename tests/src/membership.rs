use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_membership::pallet::{
    ActiveMemberCount, ActiveProposalByCandidate, MemberStatus, Members, MembershipProposalStatus,
    MembershipProposals,
};
use gaia_runtime::{Membership, Runtime, RuntimeOrigin};

// ---------------------------------------------------------------------------
// Genesis
// ---------------------------------------------------------------------------

#[test]
fn genesis_seeds_three_active_members() {
    new_test_ext().execute_with(|| {
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 3);
        for who in [alice(), bob(), charlie()] {
            let record = Members::<Runtime>::get(&who).expect("member exists");
            assert_eq!(record.status, MemberStatus::Active);
        }
        assert!(!Members::<Runtime>::contains_key(dave()));
    });
}

// ---------------------------------------------------------------------------
// Membership proposal lifecycle
// ---------------------------------------------------------------------------

#[test]
fn propose_and_admit_new_member() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");

        // 3 active → 80% threshold → need 3 approvals
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            proposal_id,
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(bob()),
            proposal_id,
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(charlie()),
            proposal_id,
            true
        ));

        assert_eq!(Members::<Runtime>::get(dave()).unwrap().status, MemberStatus::Active);
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 4);
        assert_eq!(
            MembershipProposals::<Runtime>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Approved
        );
        assert_eq!(ActiveProposalByCandidate::<Runtime>::get(dave()), None);
    });
}

#[test]
fn membership_proposal_rejected_after_deadline_when_below_threshold() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");

        // 2/3 yes, below threshold.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            proposal_id,
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(bob()),
            proposal_id,
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(charlie()),
            proposal_id,
            false
        ));

        advance_past_membership_voting_period();
        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(alice()),
            proposal_id
        ));

        assert!(!Members::<Runtime>::contains_key(dave()));
        assert_eq!(
            MembershipProposals::<Runtime>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Rejected
        );
        assert_eq!(ActiveProposalByCandidate::<Runtime>::get(dave()), None);
    });
}

#[test]
fn finalize_requires_active_member() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");
        advance_past_membership_voting_period();

        assert_noop!(
            Membership::finalize_proposal(RuntimeOrigin::signed(dave()), proposal_id),
            gaia_membership::Error::<Runtime>::NotActiveMember
        );
    });
}

#[test]
fn finalize_before_deadline_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");
        assert_noop!(
            Membership::finalize_proposal(RuntimeOrigin::signed(alice()), proposal_id),
            gaia_membership::Error::<Runtime>::VotingStillOpen
        );
    });
}

#[test]
fn threshold_uses_submit_time_snapshot() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");

        // Snapshot is 3. Two yes votes do not satisfy 80%.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            proposal_id,
            true
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(bob()),
            proposal_id,
            true
        ));

        // Live active member count drops to 2, but snapshot should still apply.
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(charlie())));
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 2);

        advance_past_membership_voting_period();
        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(alice()),
            proposal_id
        ));

        assert!(!Members::<Runtime>::contains_key(dave()));
        assert_eq!(
            MembershipProposals::<Runtime>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Rejected
        );
    });
}

#[test]
fn one_active_membership_proposal_per_candidate() {
    new_test_ext().execute_with(|| {
        let _id = submit_membership_proposal(alice(), dave(), b"Dave");

        assert_noop!(
            Membership::propose_member(
                RuntimeOrigin::signed(bob()),
                dave(),
                bounded_name(b"Dave")
            ),
            gaia_membership::Error::<Runtime>::CandidateAlreadyProposed
        );
    });
}

#[test]
fn can_repropose_after_rejection() {
    new_test_ext().execute_with(|| {
        let first = submit_membership_proposal(alice(), dave(), b"Dave");
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            first,
            false
        ));

        advance_past_membership_voting_period();
        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(bob()),
            first
        ));

        let second = submit_membership_proposal(alice(), dave(), b"Dave-v2");
        assert!(second > first);
    });
}

#[test]
fn non_member_cannot_propose_candidate() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(dave()), eve(), bounded_name(b"Eve")),
            gaia_membership::Error::<Runtime>::NotActiveMember
        );
    });
}

#[test]
fn propose_existing_member_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(alice()), bob(), bounded_name(b"Bob")),
            gaia_membership::Error::<Runtime>::AlreadyMember
        );
    });
}

#[test]
fn double_vote_on_candidate_fails() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            proposal_id,
            true
        ));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(alice()), proposal_id, true),
            gaia_membership::Error::<Runtime>::AlreadyVoted
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
            Members::<Runtime>::get(alice()).unwrap().status,
            MemberStatus::Suspended
        );
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 2);
    });
}

#[test]
fn double_self_suspension_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(alice())),
            gaia_membership::Error::<Runtime>::AlreadySuspended
        );
    });
}

#[test]
fn non_member_cannot_self_suspend() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(dave())),
            gaia_membership::Error::<Runtime>::NotActiveMember
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
            Members::<Runtime>::get(alice()).unwrap().status,
            MemberStatus::Active
        );

        // Second vote meets unanimity (all others = 2)
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(charlie()),
            alice(),
            true
        ));
        assert_eq!(
            Members::<Runtime>::get(alice()).unwrap().status,
            MemberStatus::Suspended
        );
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 2);
    });
}

#[test]
fn cannot_cast_suspension_vote_against_self() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::vote_suspend_member(RuntimeOrigin::signed(alice()), alice(), true),
            gaia_membership::Error::<Runtime>::CannotSuspendSelf
        );
    });
}

#[test]
fn suspended_member_cannot_vote_on_membership_proposal() {
    new_test_ext().execute_with(|| {
        let proposal_id = submit_membership_proposal(bob(), dave(), b"Dave");
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(alice()), proposal_id, true),
            gaia_membership::Error::<Runtime>::MemberSuspended
        );
    });
}

#[test]
fn suspended_member_cannot_propose_candidate() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(alice()), dave(), bounded_name(b"Dave")),
            gaia_membership::Error::<Runtime>::MemberSuspended
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

#[test]
fn single_member_genesis_admits_candidate() {
    new_test_ext_with_members(&[(alice(), b"Alice")]).execute_with(|| {
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 1);

        let proposal_id = submit_membership_proposal(alice(), dave(), b"Dave");
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(alice()),
            proposal_id,
            true
        ));

        assert_eq!(Members::<Runtime>::get(dave()).unwrap().status, MemberStatus::Active);
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 2);
    });
}

#[test]
fn threshold_boundary_with_five_members() {
    new_test_ext().execute_with(|| {
        // Admit Dave and Eve to reach five active members.
        let dave_proposal = admit_candidate(dave(), b"Dave");
        assert_eq!(
            MembershipProposals::<Runtime>::get(dave_proposal)
                .unwrap()
                .status,
            MembershipProposalStatus::Approved
        );
        let eve_proposal = submit_membership_proposal(alice(), eve(), b"Eve");
        for voter in [alice(), bob(), charlie(), dave()] {
            assert_ok!(Membership::vote_on_candidate(
                RuntimeOrigin::signed(voter),
                eve_proposal,
                true
            ));
        }
        assert_eq!(
            MembershipProposals::<Runtime>::get(eve_proposal)
                .unwrap()
                .status,
            MembershipProposalStatus::Approved
        );
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 5);

        // 3 yes votes out of 5 should not approve.
        let ferdie_proposal = submit_membership_proposal(alice(), ferdie(), b"Ferdie");
        for voter in [alice(), bob(), charlie()] {
            assert_ok!(Membership::vote_on_candidate(
                RuntimeOrigin::signed(voter),
                ferdie_proposal,
                true
            ));
        }
        assert!(!Members::<Runtime>::contains_key(ferdie()));

        // 4th yes vote reaches threshold.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(dave()),
            ferdie_proposal,
            true
        ));
        assert_eq!(Members::<Runtime>::get(ferdie()).unwrap().status, MemberStatus::Active);
        assert_eq!(ActiveMemberCount::<Runtime>::get(), 6);
    });
}

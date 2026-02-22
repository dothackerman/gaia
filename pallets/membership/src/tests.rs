use crate::mock::*;
use crate::pallet::{
    ActiveMemberCount, CandidateApprovalCount, CandidateVotes, Candidates, MemberStatus, Members,
    SuspensionApprovalCount, SuspensionReason, SuspensionVotes,
};
use crate::{Error, Event, MembershipChecker};
use frame_support::{assert_noop, assert_ok};

// ---------------------------------------------------------------------------
// Genesis sanity
// ---------------------------------------------------------------------------

#[test]
fn genesis_seeds_three_active_members() {
    new_test_ext().execute_with(|| {
        assert_eq!(ActiveMemberCount::<Test>::get(), 3);
        assert!(Members::<Test>::contains_key(ALICE));
        assert!(Members::<Test>::contains_key(BOB));
        assert!(Members::<Test>::contains_key(CHARLIE));
        assert!(!Members::<Test>::contains_key(DAVE));
    });
}

#[test]
fn genesis_members_are_active() {
    new_test_ext().execute_with(|| {
        let record = Members::<Test>::get(ALICE).unwrap();
        assert_eq!(record.status, MemberStatus::Active);
        assert_eq!(record.name.as_slice(), b"Alice");
    });
}

// ---------------------------------------------------------------------------
// MembershipChecker trait
// ---------------------------------------------------------------------------

#[test]
fn is_active_member_returns_true_for_active() {
    new_test_ext().execute_with(|| {
        assert!(<crate::Pallet<Test> as MembershipChecker<u64>>::is_active_member(&ALICE));
    });
}

#[test]
fn is_active_member_returns_false_for_non_member() {
    new_test_ext().execute_with(|| {
        assert!(!<crate::Pallet<Test> as MembershipChecker<u64>>::is_active_member(&DAVE));
    });
}

// ---------------------------------------------------------------------------
// propose_member — happy path
// ---------------------------------------------------------------------------

#[test]
fn propose_member_succeeds_for_active_member() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert!(Candidates::<Test>::contains_key(EVE));
        System::assert_last_event(
            Event::CandidateProposed {
                candidate: EVE,
                proposed_by: ALICE,
            }
            .into(),
        );
    });
}

// ---------------------------------------------------------------------------
// propose_member — failure paths
// ---------------------------------------------------------------------------

#[test]
fn propose_member_fails_for_non_member() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(DAVE), EVE, bounded_name(b"Eve"),),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn propose_member_fails_for_existing_member() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(ALICE), BOB, bounded_name(b"Bob"),),
            Error::<Test>::AlreadyMember
        );
    });
}

#[test]
fn propose_member_fails_for_duplicate_proposal() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(BOB), EVE, bounded_name(b"Eve"),),
            Error::<Test>::CandidateAlreadyProposed
        );
    });
}

// ---------------------------------------------------------------------------
// vote_on_candidate — happy path
// ---------------------------------------------------------------------------

#[test]
fn vote_on_candidate_records_vote() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            EVE,
            true,
        ));
        assert!(CandidateVotes::<Test>::contains_key(EVE, ALICE));
        assert_eq!(CandidateApprovalCount::<Test>::get(EVE), 1);
        System::assert_last_event(
            Event::VoteCast {
                candidate: EVE,
                voter: ALICE,
                approve: true,
            }
            .into(),
        );
    });
}

#[test]
fn rejection_vote_does_not_increment_approval_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            EVE,
            false,
        ));
        assert_eq!(CandidateApprovalCount::<Test>::get(EVE), 0);
    });
}

// ---------------------------------------------------------------------------
// vote_on_candidate — failure paths
// ---------------------------------------------------------------------------

#[test]
fn vote_fails_for_non_member() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(DAVE), EVE, true),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn vote_fails_for_nonexistent_candidate() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), EVE, true),
            Error::<Test>::CandidateNotFound
        );
    });
}

#[test]
fn vote_fails_for_double_vote() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            EVE,
            true,
        ));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), EVE, true),
            Error::<Test>::AlreadyVoted
        );
    });
}

// ---------------------------------------------------------------------------
// Approval threshold (80 %)
// ---------------------------------------------------------------------------

#[test]
fn candidate_approved_at_80_percent_threshold() {
    // 3 active members → 80 % = ceil(2.4) = 3 needed
    // With integer math: approval * 5 >= active * 4  →  3 * 5 = 15 >= 3 * 4 = 12 ✓
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        // Vote 1
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            EVE,
            true,
        ));
        assert!(!Members::<Test>::contains_key(EVE));

        // Vote 2
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            EVE,
            true,
        ));
        assert!(!Members::<Test>::contains_key(EVE));

        // Vote 3 — threshold met
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(CHARLIE),
            EVE,
            true,
        ));
        // Now Eve should be a member
        assert!(Members::<Test>::contains_key(EVE));
        assert_eq!(ActiveMemberCount::<Test>::get(), 4);
        // Candidate storage cleaned up
        assert!(!Candidates::<Test>::contains_key(EVE));
        assert_eq!(CandidateApprovalCount::<Test>::get(EVE), 0);
    });
}

#[test]
fn candidate_not_approved_below_threshold() {
    // 3 active members: 2 approve, 1 rejects → 2 * 5 = 10 < 3 * 4 = 12 → not approved
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            EVE,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            EVE,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(CHARLIE),
            EVE,
            false,
        ));
        // Should NOT be approved — 2/3 = 66 %, below 80 %
        assert!(!Members::<Test>::contains_key(EVE));
        assert!(Candidates::<Test>::contains_key(EVE));
    });
}

// ---------------------------------------------------------------------------
// Suspended member restrictions
// ---------------------------------------------------------------------------

#[test]
fn suspended_member_cannot_propose() {
    new_test_ext().execute_with(|| {
        // Manually suspend Alice for testing purposes.
        Members::<Test>::mutate(ALICE, |maybe| {
            if let Some(ref mut record) = maybe {
                record.status = MemberStatus::Suspended;
            }
        });
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(ALICE), EVE, bounded_name(b"Eve"),),
            Error::<Test>::MemberSuspended
        );
    });
}

#[test]
fn suspended_member_cannot_vote() {
    new_test_ext().execute_with(|| {
        // First, propose EVE while Alice is still active.
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(BOB),
            EVE,
            bounded_name(b"Eve"),
        ));
        // Suspend Alice.
        Members::<Test>::mutate(ALICE, |maybe| {
            if let Some(ref mut record) = maybe {
                record.status = MemberStatus::Suspended;
            }
        });
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), EVE, true),
            Error::<Test>::MemberSuspended
        );
    });
}

// ---------------------------------------------------------------------------
// Suspension dispatchables
// ---------------------------------------------------------------------------

#[test]
fn suspend_self_marks_member_suspended_and_decrements_count() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(ALICE)));
        assert_eq!(ActiveMemberCount::<Test>::get(), 2);
        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Suspended
        );
        System::assert_last_event(
            Event::MemberSuspended {
                member: ALICE,
                reason: SuspensionReason::SelfInitiated,
            }
            .into(),
        );
    });
}

#[test]
fn suspend_self_fails_for_non_member() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(DAVE)),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn suspend_self_twice_returns_already_suspended() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(ALICE)));
        assert_noop!(
            Membership::suspend_self(RuntimeOrigin::signed(ALICE)),
            Error::<Test>::AlreadySuspended
        );
    });
}

#[test]
fn vote_suspend_member_requires_unanimous_other_members() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(BOB),
            ALICE,
            true
        ));
        assert_eq!(ActiveMemberCount::<Test>::get(), 3);
        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Active
        );
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(CHARLIE),
            ALICE,
            true
        ));
        assert_eq!(ActiveMemberCount::<Test>::get(), 2);
        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Suspended
        );
        assert!(!SuspensionVotes::<Test>::contains_key(ALICE, BOB));
        assert_eq!(SuspensionApprovalCount::<Test>::get(ALICE), 0);
        System::assert_last_event(
            Event::MemberSuspended {
                member: ALICE,
                reason: SuspensionReason::PeerVote,
            }
            .into(),
        );
    });
}

#[test]
fn vote_suspend_member_rejects_self_target() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::vote_suspend_member(RuntimeOrigin::signed(ALICE), ALICE, true),
            Error::<Test>::CannotSuspendSelf
        );
    });
}

#[test]
fn self_suspension_clears_pending_votes_against_other_targets() {
    // Regression: if Charlie self-suspends while a suspension vote against
    // Alice is in progress, the voter pool shrinks and stale votes must be
    // invalidated to prevent an incorrect threshold match.
    new_test_ext().execute_with(|| {
        // 3 active members: ALICE, BOB, CHARLIE.
        // BOB votes to suspend ALICE (requires unanimity of others = 2).
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(BOB),
            ALICE,
            true,
        ));
        assert_eq!(SuspensionApprovalCount::<Test>::get(ALICE), 1);

        // CHARLIE self-suspends → active count drops to 2.
        // All pending suspension votes must be cleared.
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(CHARLIE)));
        assert_eq!(ActiveMemberCount::<Test>::get(), 2);
        assert_eq!(SuspensionApprovalCount::<Test>::get(ALICE), 0);
        assert!(!SuspensionVotes::<Test>::contains_key(ALICE, BOB));

        // ALICE must still be active — the stale vote must not trigger her
        // suspension.
        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Active
        );
    });
}

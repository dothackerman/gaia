use crate::mock::*;
use crate::pallet::{
    ActiveMemberCount, ActiveProposalByCandidate, MemberRecord, MemberStatus, Members,
    MembershipProposalCount,
    MembershipApprovalDenominator, MembershipApprovalNumerator, MembershipProposalNoCount,
    MembershipProposalStatus, MembershipProposalVotes, MembershipProposalYesCount,
    MembershipProposals, MembershipVotingPeriod, SuspensionApprovalCount, SuspensionDenominator,
    SuspensionNumerator, SuspensionReason, SuspensionVotes,
};
use crate::{Error, Event, MembershipChecker};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::OnInitialize;
use sp_runtime::DispatchError;

fn advance_past_membership_voting_period() {
    let period = MembershipVotingPeriod::<Test>::get();
    for _ in 0..=period {
        let next = System::block_number() + 1;
        System::set_block_number(next);
        System::on_initialize(next);
    }
}

// ---------------------------------------------------------------------------
// Governance parameter setters
// ---------------------------------------------------------------------------

#[test]
fn set_membership_voting_period_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::set_membership_voting_period(
            RuntimeOrigin::root(),
            42
        ));
        assert_eq!(MembershipVotingPeriod::<Test>::get(), 42);
        System::assert_last_event(Event::MembershipVotingPeriodSet { blocks: 42 }.into());
    });
}

#[test]
fn set_membership_approval_threshold_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::set_membership_approval_threshold(
            RuntimeOrigin::root(),
            3,
            4,
        ));
        assert_eq!(MembershipApprovalNumerator::<Test>::get(), 3);
        assert_eq!(MembershipApprovalDenominator::<Test>::get(), 4);
        System::assert_last_event(
            Event::MembershipApprovalThresholdSet {
                numerator: 3,
                denominator: 4,
            }
            .into(),
        );
    });
}

#[test]
fn set_suspension_threshold_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::set_suspension_threshold(
            RuntimeOrigin::root(),
            2,
            3,
        ));
        assert_eq!(SuspensionNumerator::<Test>::get(), 2);
        assert_eq!(SuspensionDenominator::<Test>::get(), 3);
        System::assert_last_event(
            Event::SuspensionThresholdSet {
                numerator: 2,
                denominator: 3,
            }
            .into(),
        );
    });
}

#[test]
fn set_threshold_rejects_zero_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::set_membership_approval_threshold(RuntimeOrigin::root(), 1, 0),
            Error::<Test>::InvalidThreshold
        );
        assert_noop!(
            Membership::set_suspension_threshold(RuntimeOrigin::root(), 1, 0),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn set_threshold_rejects_numerator_greater_than_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::set_membership_approval_threshold(RuntimeOrigin::root(), 5, 4),
            Error::<Test>::InvalidThreshold
        );
        assert_noop!(
            Membership::set_suspension_threshold(RuntimeOrigin::root(), 2, 1),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn membership_approval_uses_stored_threshold() {
    new_test_ext().execute_with(|| {
        let dave_record = MemberRecord::<Test> {
            name: bounded_name(b"Dave"),
            status: MemberStatus::Active,
            joined_at: System::block_number(),
        };
        let extra_record = MemberRecord::<Test> {
            name: bounded_name(b"Extra"),
            status: MemberStatus::Active,
            joined_at: System::block_number(),
        };
        Members::<Test>::insert(DAVE, dave_record);
        Members::<Test>::insert(6u64, extra_record);
        ActiveMemberCount::<Test>::put(5);

        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            7u64,
            bounded_name(b"Candidate"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(CHARLIE),
            proposal_id,
            true,
        ));
        assert!(Members::<Test>::get(7u64).is_none());

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(6u64),
            proposal_id,
            true,
        ));
        assert_eq!(Members::<Test>::get(7u64).unwrap().status, MemberStatus::Active);
    });
}

#[test]
fn suspension_threshold_default_requires_unanimity() {
    new_test_ext().execute_with(|| {
        assert_eq!(SuspensionNumerator::<Test>::get(), 1);
        assert_eq!(SuspensionDenominator::<Test>::get(), 1);

        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(BOB),
            ALICE,
            true,
        ));
        assert_eq!(Members::<Test>::get(ALICE).unwrap().status, MemberStatus::Active);

        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(CHARLIE),
            ALICE,
            true,
        ));
        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Suspended
        );
    });
}

#[test]
fn non_root_cannot_call_setters() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::set_membership_voting_period(RuntimeOrigin::signed(ALICE), 9),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Membership::set_membership_approval_threshold(RuntimeOrigin::signed(ALICE), 4, 5),
            DispatchError::BadOrigin
        );
        assert_noop!(
            Membership::set_suspension_threshold(RuntimeOrigin::signed(ALICE), 1, 1),
            DispatchError::BadOrigin
        );
    });
}

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
// propose_member
// ---------------------------------------------------------------------------

#[test]
fn propose_member_creates_active_proposal_with_snapshot_and_deadline() {
    new_test_ext().execute_with(|| {
        let now = System::block_number();
        let period = MembershipVotingPeriod::<Test>::get();

        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));

        let proposal_id = MembershipProposalCount::<Test>::get();
        let proposal = MembershipProposals::<Test>::get(proposal_id).expect("proposal exists");

        assert_eq!(proposal.candidate, EVE);
        assert_eq!(proposal.proposed_by, ALICE);
        assert_eq!(proposal.proposed_at, now);
        assert_eq!(proposal.vote_end, now + period);
        assert_eq!(proposal.active_member_snapshot, 3);
        assert_eq!(proposal.status, MembershipProposalStatus::Active);
        assert_eq!(ActiveProposalByCandidate::<Test>::get(EVE), Some(proposal_id));

        System::assert_last_event(
            Event::MemberProposalSubmitted {
                proposal_id,
                candidate: EVE,
                proposed_by: ALICE,
                vote_end: now + period,
            }
            .into(),
        );
    });
}

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
fn propose_member_fails_for_duplicate_active_proposal() {
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
// vote_on_candidate
// ---------------------------------------------------------------------------

#[test]
fn vote_on_candidate_records_yes_vote() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));

        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));

        assert!(MembershipProposalVotes::<Test>::contains_key(proposal_id, ALICE));
        assert_eq!(MembershipProposalYesCount::<Test>::get(proposal_id), 1);
        assert_eq!(MembershipProposalNoCount::<Test>::get(proposal_id), 0);

        System::assert_last_event(
            Event::MemberProposalVoteCast {
                proposal_id,
                candidate: EVE,
                voter: ALICE,
                approve: true,
            }
            .into(),
        );
    });
}

#[test]
fn vote_on_candidate_records_no_vote() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));

        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            false,
        ));

        assert_eq!(MembershipProposalYesCount::<Test>::get(proposal_id), 0);
        assert_eq!(MembershipProposalNoCount::<Test>::get(proposal_id), 1);
    });
}

#[test]
fn vote_fails_for_non_member() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(DAVE), proposal_id, true),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn vote_fails_for_unknown_proposal() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), 999, true),
            Error::<Test>::ProposalNotFound
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
        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));

        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), proposal_id, true),
            Error::<Test>::AlreadyVoted
        );
    });
}

#[test]
fn vote_fails_after_voting_window_closed() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        advance_past_membership_voting_period();

        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), proposal_id, true),
            Error::<Test>::VotingClosed
        );
    });
}

#[test]
fn vote_approves_candidate_early_at_snapshot_threshold() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        // Snapshot = 3 active members. Threshold requires 3 yes votes.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            true,
        ));
        assert!(Members::<Test>::get(EVE).is_none());

        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(CHARLIE),
            proposal_id,
            true,
        ));

        assert_eq!(Members::<Test>::get(EVE).unwrap().status, MemberStatus::Active);
        assert_eq!(ActiveMemberCount::<Test>::get(), 4);
        assert_eq!(ActiveProposalByCandidate::<Test>::get(EVE), None);
        assert_eq!(
            MembershipProposals::<Test>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Approved
        );

        System::assert_last_event(
            Event::MemberProposalApproved {
                proposal_id,
                member: EVE,
            }
            .into(),
        );
    });
}

// ---------------------------------------------------------------------------
// finalize_proposal
// ---------------------------------------------------------------------------

#[test]
fn finalize_rejects_when_threshold_not_met_by_deadline() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        // 2/3 yes is below 80% threshold.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(CHARLIE),
            proposal_id,
            false,
        ));

        advance_past_membership_voting_period();

        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
        ));

        assert!(Members::<Test>::get(EVE).is_none());
        assert_eq!(ActiveProposalByCandidate::<Test>::get(EVE), None);
        assert_eq!(
            MembershipProposals::<Test>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Rejected
        );

        System::assert_last_event(
            Event::MemberProposalRejected {
                proposal_id,
                candidate: EVE,
            }
            .into(),
        );
    });
}

#[test]
fn finalize_fails_before_voting_window_ends() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_noop!(
            Membership::finalize_proposal(RuntimeOrigin::signed(ALICE), proposal_id),
            Error::<Test>::VotingStillOpen
        );
    });
}

#[test]
fn finalize_requires_active_member_origin() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        advance_past_membership_voting_period();

        assert_noop!(
            Membership::finalize_proposal(RuntimeOrigin::signed(DAVE), proposal_id),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn finalize_uses_submit_time_snapshot_threshold() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        // Two yes votes with snapshot=3 is insufficient.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
            true,
        ));
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(BOB),
            proposal_id,
            true,
        ));

        // Shrink live active member count to 2.
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(CHARLIE)));
        assert_eq!(ActiveMemberCount::<Test>::get(), 2);

        advance_past_membership_voting_period();

        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(ALICE),
            proposal_id,
        ));

        // Still rejected because threshold uses snapshot=3.
        assert!(Members::<Test>::get(EVE).is_none());
        assert_eq!(
            MembershipProposals::<Test>::get(proposal_id)
                .unwrap()
                .status,
            MembershipProposalStatus::Rejected
        );
    });
}

#[test]
fn can_repropose_candidate_after_rejection() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve"),
        ));
        let first = MembershipProposalCount::<Test>::get();

        // Reject first proposal.
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(ALICE),
            first,
            false,
        ));
        advance_past_membership_voting_period();
        assert_ok!(Membership::finalize_proposal(
            RuntimeOrigin::signed(BOB),
            first,
        ));

        assert_eq!(
            MembershipProposals::<Test>::get(first).unwrap().status,
            MembershipProposalStatus::Rejected
        );

        // Candidate can be proposed again once prior active proposal is resolved.
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(ALICE),
            EVE,
            bounded_name(b"Eve v2"),
        ));
        let second = MembershipProposalCount::<Test>::get();
        assert!(second > first);
        assert_eq!(ActiveProposalByCandidate::<Test>::get(EVE), Some(second));
    });
}

// ---------------------------------------------------------------------------
// Suspended member restrictions
// ---------------------------------------------------------------------------

#[test]
fn suspended_member_cannot_propose() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(ALICE)));
        assert_noop!(
            Membership::propose_member(RuntimeOrigin::signed(ALICE), EVE, bounded_name(b"Eve"),),
            Error::<Test>::MemberSuspended
        );
    });
}

#[test]
fn suspended_member_cannot_vote() {
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(BOB),
            EVE,
            bounded_name(b"Eve"),
        ));
        let proposal_id = MembershipProposalCount::<Test>::get();

        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(ALICE)));
        assert_noop!(
            Membership::vote_on_candidate(RuntimeOrigin::signed(ALICE), proposal_id, true),
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
    new_test_ext().execute_with(|| {
        assert_ok!(Membership::vote_suspend_member(
            RuntimeOrigin::signed(BOB),
            ALICE,
            true,
        ));
        assert_eq!(SuspensionApprovalCount::<Test>::get(ALICE), 1);

        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(CHARLIE)));
        assert_eq!(ActiveMemberCount::<Test>::get(), 2);
        assert_eq!(SuspensionApprovalCount::<Test>::get(ALICE), 0);
        assert!(!SuspensionVotes::<Test>::contains_key(ALICE, BOB));

        assert_eq!(
            Members::<Test>::get(ALICE).unwrap().status,
            MemberStatus::Active
        );
    });
}

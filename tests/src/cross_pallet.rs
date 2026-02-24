use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_runtime::{Membership, Proposals, Runtime, RuntimeOrigin, Treasury};

// ---------------------------------------------------------------------------
// I-2: Only active members vote
// ---------------------------------------------------------------------------

#[test]
fn only_active_members_can_vote_on_proposals() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();
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
        let id = submit_default_proposal();
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
        advance_past_voting_period();
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
            Proposals::submit_proposal(RuntimeOrigin::signed(alice()), bounded_title(b"t"), bounded_desc(b"d"), 100, 10),
            gaia_proposals::Error::<Runtime>::NotActiveMember
        );
    });
}

#[test]
fn suspension_during_voting_period() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal();
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
        advance_past_voting_period();
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
        let id = submit_default_proposal();
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
            bounded_title(b"Dave's idea"),
            bounded_desc(b"Dave proposes something"),
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
        let id = submit_default_proposal(); // requests 100
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
        advance_past_voting_period();
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

// ---------------------------------------------------------------------------
// Suspended organizer ↔ proposal execution
// ---------------------------------------------------------------------------

/// `execute_proposal` checks `NotOrganizer` but does NOT check
/// `is_active_member`. A suspended organizer can still execute their
/// approved proposal. This test documents that behaviour.
#[test]
fn suspended_organizer_can_execute_approved_proposal() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));
        let id = submit_default_proposal(); // Alice is organizer

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

        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(bob()),
            id
        ));

        // Alice self-suspends AFTER approval
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));

        // Execution still succeeds — no active-member check in execute_proposal
        assert_ok!(Proposals::execute_proposal(
            RuntimeOrigin::signed(alice()),
            id
        ));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Executed
        );
    });
}

// ---------------------------------------------------------------------------
// Tally after all voters suspend
// ---------------------------------------------------------------------------

/// All three members vote, then all self-suspend. A non-member (Dave)
/// tallies successfully. Votes cast before suspension still count, and
/// tally does not require membership.
#[test]
fn tally_succeeds_after_all_voters_suspended() {
    new_test_ext().execute_with(|| {
        let id = submit_default_proposal();

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
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(charlie()),
            id,
            false
        ));

        // All members self-suspend
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(alice())));
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(bob())));
        assert_ok!(Membership::suspend_self(RuntimeOrigin::signed(charlie())));
        assert_eq!(
            gaia_membership::pallet::ActiveMemberCount::<Runtime>::get(),
            0
        );

        advance_past_voting_period();

        // Non-member tallies — succeeds (2 yes > 1 no)
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(dave()),
            id
        ));
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

// ---------------------------------------------------------------------------
// Newly admitted member + suspension interaction
// ---------------------------------------------------------------------------

/// A freshly admitted member votes on a proposal, then the proposer
/// (also a member who voted) self-suspends. The newly admitted member's
/// vote still counts toward tally.
#[test]
fn newly_admitted_member_vote_counts_after_proposer_suspends() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 500));

        // Admit Dave
        assert_ok!(Membership::propose_member(
            RuntimeOrigin::signed(alice()),
            dave(),
            bounded_name(b"Dave")
        ));
        for voter in [alice(), bob(), charlie()] {
            assert_ok!(Membership::vote_on_candidate(
                RuntimeOrigin::signed(voter),
                dave(),
                true
            ));
        }

        let id = submit_default_proposal(); // Alice submits

        // Dave and Alice vote yes
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(dave()),
            id,
            true
        ));
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(alice()),
            id,
            true
        ));

        // Bob votes no, Charlie abstains (doesn't vote)
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(bob()),
            id,
            false
        ));

        advance_past_voting_period();
        assert_ok!(Proposals::tally_proposal(
            RuntimeOrigin::signed(charlie()),
            id
        ));

        // 2 yes > 1 no → Approved
        assert_eq!(
            gaia_proposals::pallet::Proposals::<Runtime>::get(id)
                .unwrap()
                .status,
            gaia_proposals::pallet::ProposalStatus::Approved
        );
    });
}

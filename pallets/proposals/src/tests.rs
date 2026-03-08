use crate::mock::*;
use crate::pallet::{
    ConstitutionalApprovalDenominator,
    ConstitutionalApprovalNumerator,
    ExecutionDelay,
    GovernanceApprovalDenominator,
    GovernanceApprovalNumerator,
    ProposalNoCount,
    ProposalStatus,
    ProposalVotes,
    ProposalVotingPeriod,
    ProposalYesCount,
    PendingRuntimeCode,
    // Aliased to avoid shadowing the pallet type alias `Proposals` from mock.
    Proposals as ProposalStorage,
    GovernanceAction,
    ProposalClass,
    StandardApprovalDenominator,
    StandardApprovalNumerator,
};
use crate::{Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_support::sp_runtime::traits::AccountIdConversion;
use frame_support::traits::{GetStorageVersion, StorageVersion};

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
        ProposalClass::Standard,
        GovernanceAction::DisburseToAccount {
            recipient: origin,
            amount: 100u64,
        },
    ));
    crate::pallet::ProposalCount::<Test>::get()
}

fn submit_with_class(origin: u64, class: ProposalClass) -> u32 {
    let action = match class {
        ProposalClass::Standard => GovernanceAction::DisburseToAccount {
            recipient: origin,
            amount: 100u64,
        },
        ProposalClass::Governance => GovernanceAction::SetProposalVotingPeriod { blocks: 25u64 },
        ProposalClass::Constitutional => GovernanceAction::SetConstitutionalApprovalThreshold {
            numerator: 9,
            denominator: 10,
        },
    };
    assert_ok!(Proposals::submit_proposal(
        RuntimeOrigin::signed(origin),
        bounded_title(b"Governance proposal"),
        bounded_desc(b"parameter update"),
        class,
        action,
    ));
    crate::pallet::ProposalCount::<Test>::get()
}

fn submit_upgrade_proposal(origin: u64, code_hash: [u8; 32]) -> u32 {
    assert_ok!(Proposals::submit_proposal(
        RuntimeOrigin::signed(origin),
        bounded_title(b"Upgrade runtime"),
        bounded_desc(b"Upgrade runtime blob"),
        ProposalClass::Constitutional,
        GovernanceAction::UpgradeRuntime { code_hash },
    ));
    crate::pallet::ProposalCount::<Test>::get()
}

/// Advance chain to a block number past the voting window.
fn advance_past_voting() {
    let period = ProposalVotingPeriod::<Test>::get();
    let now = System::block_number();
    System::set_block_number(now.saturating_add(period).saturating_add(1));
}

fn governance_origin() -> RuntimeOrigin {
    RuntimeOrigin::signed(GovernancePalletId::get().into_account_truncating())
}

// ---------------------------------------------------------------------------
// Governance parameter setters
// ---------------------------------------------------------------------------

#[test]
fn set_proposal_voting_period_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_proposal_voting_period(
            governance_origin(),
            42
        ));
        assert_eq!(ProposalVotingPeriod::<Test>::get(), 42);
        System::assert_last_event(Event::ProposalVotingPeriodSet { blocks: 42 }.into());
    });
}

#[test]
fn set_execution_delay_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_execution_delay(governance_origin(), 7));
        assert_eq!(ExecutionDelay::<Test>::get(), 7);
        System::assert_last_event(Event::ExecutionDelaySet { blocks: 7 }.into());
    });
}

#[test]
fn set_standard_threshold_updates_storage() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_standard_approval_threshold(
            governance_origin(),
            3,
            4
        ));
        assert_eq!(StandardApprovalNumerator::<Test>::get(), 3);
        assert_eq!(StandardApprovalDenominator::<Test>::get(), 4);

        assert_ok!(Proposals::set_governance_approval_threshold(
            governance_origin(),
            4,
            5
        ));
        assert_eq!(GovernanceApprovalNumerator::<Test>::get(), 4);
        assert_eq!(GovernanceApprovalDenominator::<Test>::get(), 5);

        assert_ok!(Proposals::set_constitutional_approval_threshold(
            governance_origin(),
            9,
            10
        ));
        assert_eq!(ConstitutionalApprovalNumerator::<Test>::get(), 9);
        assert_eq!(ConstitutionalApprovalDenominator::<Test>::get(), 10);
    });
}

#[test]
fn set_threshold_rejects_zero_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_standard_approval_threshold(governance_origin(), 1, 0),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn set_threshold_rejects_numerator_greater_than_denominator() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_standard_approval_threshold(governance_origin(), 3, 2),
            Error::<Test>::InvalidThreshold
        );
    });
}

#[test]
fn non_governance_origin_cannot_call_setters() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::set_proposal_voting_period(RuntimeOrigin::signed(ALICE), 55),
            Error::<Test>::NotGovernanceOrigin
        );
        assert_noop!(
            Proposals::set_execution_delay(RuntimeOrigin::signed(ALICE), 2),
            Error::<Test>::NotGovernanceOrigin
        );
        assert_noop!(
            Proposals::set_standard_approval_threshold(RuntimeOrigin::signed(ALICE), 1, 2),
            Error::<Test>::NotGovernanceOrigin
        );
        assert_noop!(
            Proposals::set_governance_approval_threshold(RuntimeOrigin::signed(ALICE), 4, 5),
            Error::<Test>::NotGovernanceOrigin
        );
        assert_noop!(
            Proposals::set_constitutional_approval_threshold(RuntimeOrigin::signed(ALICE), 9, 10),
            Error::<Test>::NotGovernanceOrigin
        );
    });
}

#[test]
fn migration_backfills_missing_governance_parameter_keys() {
    new_test_ext().execute_with(|| {
        ProposalVotingPeriod::<Test>::kill();
        ExecutionDelay::<Test>::kill();
        StandardApprovalNumerator::<Test>::kill();
        StandardApprovalDenominator::<Test>::kill();
        GovernanceApprovalNumerator::<Test>::kill();
        GovernanceApprovalDenominator::<Test>::kill();
        ConstitutionalApprovalNumerator::<Test>::kill();
        ConstitutionalApprovalDenominator::<Test>::kill();

        StorageVersion::new(0).put::<Proposals>();
        let _ = <Proposals as frame_support::traits::Hooks<u64>>::on_runtime_upgrade();

        assert_eq!(ProposalVotingPeriod::<Test>::get(), 100_800);
        assert_eq!(ExecutionDelay::<Test>::get(), 0);
        assert_eq!(StandardApprovalNumerator::<Test>::get(), 1);
        assert_eq!(StandardApprovalDenominator::<Test>::get(), 2);
        assert_eq!(GovernanceApprovalNumerator::<Test>::get(), 4);
        assert_eq!(GovernanceApprovalDenominator::<Test>::get(), 5);
        assert_eq!(ConstitutionalApprovalNumerator::<Test>::get(), 9);
        assert_eq!(ConstitutionalApprovalDenominator::<Test>::get(), 10);
        assert_eq!(
            <Proposals as GetStorageVersion>::on_chain_storage_version(),
            StorageVersion::new(1)
        );
    });
}

#[test]
fn migration_preserves_existing_parameter_values() {
    new_test_ext().execute_with(|| {
        ProposalVotingPeriod::<Test>::put(42);
        ExecutionDelay::<Test>::put(3);
        StandardApprovalNumerator::<Test>::put(3);
        StandardApprovalDenominator::<Test>::put(4);
        GovernanceApprovalNumerator::<Test>::put(2);
        GovernanceApprovalDenominator::<Test>::put(3);
        ConstitutionalApprovalNumerator::<Test>::put(8);
        ConstitutionalApprovalDenominator::<Test>::put(9);

        StorageVersion::new(0).put::<Proposals>();
        let _ = <Proposals as frame_support::traits::Hooks<u64>>::on_runtime_upgrade();

        assert_eq!(ProposalVotingPeriod::<Test>::get(), 42);
        assert_eq!(ExecutionDelay::<Test>::get(), 3);
        assert_eq!(StandardApprovalNumerator::<Test>::get(), 3);
        assert_eq!(StandardApprovalDenominator::<Test>::get(), 4);
        assert_eq!(GovernanceApprovalNumerator::<Test>::get(), 2);
        assert_eq!(GovernanceApprovalDenominator::<Test>::get(), 3);
        assert_eq!(ConstitutionalApprovalNumerator::<Test>::get(), 8);
        assert_eq!(ConstitutionalApprovalDenominator::<Test>::get(), 9);
    });
}

#[test]
fn migration_is_idempotent_after_storage_version_update() {
    new_test_ext().execute_with(|| {
        StorageVersion::new(0).put::<Proposals>();
        let _ = <Proposals as frame_support::traits::Hooks<u64>>::on_runtime_upgrade();

        ProposalVotingPeriod::<Test>::put(77);
        let _ = <Proposals as frame_support::traits::Hooks<u64>>::on_runtime_upgrade();

        assert_eq!(ProposalVotingPeriod::<Test>::get(), 77);
    });
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
                ProposalClass::Standard,
                GovernanceAction::DisburseToAccount {
                    recipient: DAVE,
                    amount: 50u64,
                },
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

#[test]
fn governance_class_requires_80_percent() {
    new_test_ext().execute_with(|| {
        // 4 yes / 1 no => 80% (approve)
        MockMembership::add(5);
        MockMembership::add(6);
        let id = submit_with_class(ALICE, ProposalClass::Governance);
        for voter in [ALICE, BOB, CHARLIE, 5] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                true
            ));
        }
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(6),
            id,
            false
        ));
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_eq!(ProposalStorage::<Test>::get(id).unwrap().status, ProposalStatus::Approved);

        // 3 yes / 2 no => 60% (reject)
        let id2 = submit_with_class(ALICE, ProposalClass::Governance);
        for voter in [ALICE, BOB, CHARLIE] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id2,
                true
            ));
        }
        for voter in [5, 6] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id2,
                false
            ));
        }
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id2));
        assert_eq!(
            ProposalStorage::<Test>::get(id2).unwrap().status,
            ProposalStatus::Rejected
        );
    });
}

#[test]
fn constitutional_class_requires_90_percent() {
    new_test_ext().execute_with(|| {
        for who in 5..=12 {
            MockMembership::add(who);
        }
        MockMembership::add(11);
        let id = submit_proposal_with_votes(9, 1, ProposalClass::Constitutional);
        assert_eq!(ProposalStorage::<Test>::get(id).unwrap().status, ProposalStatus::Approved);

        let id2 = submit_proposal_with_votes(8, 2, ProposalClass::Constitutional);
        assert_eq!(
            ProposalStorage::<Test>::get(id2).unwrap().status,
            ProposalStatus::Rejected
        );
    });
}

fn submit_proposal_with_votes(yes: usize, no: usize, class: ProposalClass) -> u32 {
    let id = submit_with_class(ALICE, class);
    let mut voters: Vec<u64> = vec![ALICE, BOB, CHARLIE, 5, 6, 7, 8, 9, 10, 11, 12];
    while voters.len() < yes + no {
        let next = (voters.len() as u64) + 20;
        MockMembership::add(next);
        voters.push(next);
    }

    for voter in voters.iter().take(yes) {
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(*voter),
            id,
            true
        ));
    }
    for voter in voters.iter().skip(yes).take(no) {
        assert_ok!(Proposals::vote_on_proposal(
            RuntimeOrigin::signed(*voter),
            id,
            false
        ));
    }
    advance_past_voting();
    assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
    id
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
            }
            .into(),
        );
    });
}

// ---------------------------------------------------------------------------
// execute_proposal — failure paths
// ---------------------------------------------------------------------------

#[test]
fn execute_allows_non_organizer() {
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
        assert_ok!(Proposals::execute_proposal(RuntimeOrigin::signed(BOB), id));
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

#[test]
fn execute_proposal_fails_before_delay_expires() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_execution_delay(governance_origin(), 10));
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
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::ExecutionTooEarly
        );
    });
}

#[test]
fn execute_proposal_succeeds_after_delay_expires() {
    new_test_ext().execute_with(|| {
        assert_ok!(Proposals::set_execution_delay(governance_origin(), 10));
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

        let approved_at = ProposalStorage::<Test>::get(id).unwrap().approved_at.unwrap();
        System::set_block_number(approved_at + 10);
        assert_ok!(Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id));
    });
}

#[test]
fn execute_proposal_fails_when_approved_at_missing() {
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

        ProposalStorage::<Test>::mutate(id, |proposal| {
            if let Some(record) = proposal {
                record.approved_at = None;
            }
        });

        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::ProposalNotYetApproved
        );
    });
}

#[test]
fn upload_runtime_code_stores_blob_and_emits_hash() {
    new_test_ext().execute_with(|| {
        let code = vec![1u8, 2, 3, 4];
        let hash = sp_io::hashing::blake2_256(&code);
        assert_ok!(Proposals::upload_runtime_code(
            RuntimeOrigin::signed(ALICE),
            code
        ));
        assert!(PendingRuntimeCode::<Test>::get().is_some());
        System::assert_last_event(
            Event::RuntimeCodeUploaded {
                uploader: ALICE,
                code_hash: hash,
            }
            .into(),
        );
    });
}

#[test]
fn upload_runtime_code_rejects_non_member() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Proposals::upload_runtime_code(RuntimeOrigin::signed(DAVE), vec![1u8, 2, 3]),
            Error::<Test>::NotActiveMember
        );
    });
}

#[test]
fn execute_runtime_upgrade_fails_without_pending_code() {
    new_test_ext().execute_with(|| {
        let code_hash = [7u8; 32];
        let id = submit_upgrade_proposal(ALICE, code_hash);
        for voter in [ALICE, BOB, CHARLIE] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                true
            ));
        }
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::NoPendingRuntimeCode
        );
    });
}

#[test]
fn execute_runtime_upgrade_fails_with_wrong_hash() {
    new_test_ext().execute_with(|| {
        let code = vec![1u8, 2, 3, 4];
        assert_ok!(Proposals::upload_runtime_code(
            RuntimeOrigin::signed(ALICE),
            code
        ));
        let wrong_hash = [9u8; 32];
        let id = submit_upgrade_proposal(ALICE, wrong_hash);
        for voter in [ALICE, BOB, CHARLIE] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                true
            ));
        }
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_noop!(
            Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id),
            Error::<Test>::RuntimeCodeHashMismatch
        );
    });
}

#[test]
fn execute_runtime_upgrade_requires_constitutional_class() {
    new_test_ext().execute_with(|| {
        let code_hash = [1u8; 32];
        assert_noop!(
            Proposals::submit_proposal(
                RuntimeOrigin::signed(ALICE),
                bounded_title(b"Bad class"),
                bounded_desc(b"Invalid class"),
                ProposalClass::Governance,
                GovernanceAction::UpgradeRuntime { code_hash },
            ),
            Error::<Test>::ProposalClassMismatch
        );
    });
}

#[test]
fn execute_runtime_upgrade_clears_pending_code_on_success() {
    new_test_ext().execute_with(|| {
        let code = vec![1u8, 2, 3, 4];
        let code_hash = sp_io::hashing::blake2_256(&code);
        assert_ok!(Proposals::upload_runtime_code(
            RuntimeOrigin::signed(ALICE),
            code
        ));
        let id = submit_upgrade_proposal(ALICE, code_hash);
        for voter in [ALICE, BOB, CHARLIE] {
            assert_ok!(Proposals::vote_on_proposal(
                RuntimeOrigin::signed(voter),
                id,
                true
            ));
        }
        advance_past_voting();
        assert_ok!(Proposals::tally_proposal(RuntimeOrigin::signed(ALICE), id));
        assert_ok!(Proposals::execute_proposal(RuntimeOrigin::signed(ALICE), id));
        assert!(PendingRuntimeCode::<Test>::get().is_none());
    });
}

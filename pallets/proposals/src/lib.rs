//! # Proposals Pallet
//!
//! Manages the full lifecycle of a spending proposal on the GAIA network:
//! **submission → voting → tally → approval/rejection → single execution**.
//!
//! ## Overview
//!
//! - Any active member may submit a proposal specifying a title, description,
//!   class, and typed governance action.
//! - Active members vote yes or no during the voting window
//!   (`submitted_at` … `submitted_at + ProposalVotingPeriod`).
//! - After the window closes, anyone may call `tally_proposal` to compute the
//!   result and transition to `Approved` or `Rejected`.
//! - An Approved proposal may be executed exactly once; execution dispatches
//!   the proposal's `GovernanceAction`.
//!
//! ## Invariants enforced
//!
//! - **I-2** Only active members may vote (`MembershipChecker` checked on every
//!   vote extrinsic).
//! - **I-3** A proposal executes at most once (status checked before execution;
//!   set to `Executed` on success).

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::dispatch::DispatchResult;
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Interface owned by this pallet for member eligibility checks.
///
/// Implemented by the membership pallet and wired in the runtime.
pub trait MembershipChecker<AccountId> {
    fn is_active_member(account: &AccountId) -> bool;
}

/// Interface owned by this pallet for treasury disbursements.
///
/// Implemented by the treasury pallet and wired in the runtime.
pub trait TreasuryHandler<AccountId, Balance> {
    fn disburse(to: &AccountId, amount: Balance) -> DispatchResult;
}

/// Interface owned by this pallet for membership-governance parameter updates.
pub trait MembershipGovernance<Origin, BlockNumber> {
    fn set_voting_period(origin: Origin, blocks: BlockNumber) -> DispatchResult;
    fn set_approval_threshold(origin: Origin, numerator: u32, denominator: u32) -> DispatchResult;
    fn set_suspension_threshold(origin: Origin, numerator: u32, denominator: u32)
        -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::traits::AccountIdConversion;
    use frame_support::traits::StorageVersion;
    use frame_support::sp_runtime::traits::SaturatedConversion;
    use frame_support::sp_runtime::Saturating;
    use frame_support::PalletId;
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;

    /// Maximum length of a proposal title in bytes.
    pub const MAX_TITLE_LEN: u32 = 128;

    /// Maximum length of a proposal description in bytes.
    pub const MAX_DESC_LEN: u32 = 1024;

    /// On-chain identifier for a proposal.
    pub type ProposalId = u32;

    /// Lifecycle state of a proposal.
    #[derive(
        Clone,
        Encode,
        Decode,
        DecodeWithMemTracking,
        Eq,
        PartialEq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum ProposalStatus {
        /// Voting window is open.
        Active,
        /// Tally passed (yes > no); awaiting execution.
        Approved,
        /// Tally failed (yes ≤ no).
        Rejected,
        /// Disbursement completed — terminal state.
        Executed,
    }

    /// Proposal approval class.
    #[derive(
        Clone,
        Encode,
        Decode,
        DecodeWithMemTracking,
        Eq,
        PartialEq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum ProposalClass {
        Standard,
        Governance,
        Constitutional,
    }

    /// Typed governance action payload executed when a proposal is approved.
    #[derive(
        Clone,
        Encode,
        Decode,
        DecodeWithMemTracking,
        Eq,
        PartialEq,
        RuntimeDebug,
        TypeInfo,
        MaxEncodedLen,
    )]
    pub enum GovernanceAction<AccountId, Balance, BlockNumber> {
        DisburseToAccount {
            recipient: AccountId,
            amount: Balance,
        },
        SetProposalVotingPeriod {
            blocks: BlockNumber,
        },
        SetExecutionDelay {
            blocks: BlockNumber,
        },
        SetStandardApprovalThreshold {
            numerator: u32,
            denominator: u32,
        },
        SetGovernanceApprovalThreshold {
            numerator: u32,
            denominator: u32,
        },
        SetConstitutionalApprovalThreshold {
            numerator: u32,
            denominator: u32,
        },
        SetMembershipVotingPeriod {
            blocks: BlockNumber,
        },
        SetMembershipApprovalThreshold {
            numerator: u32,
            denominator: u32,
        },
        SetSuspensionThreshold {
            numerator: u32,
            denominator: u32,
        },
    }

    /// On-chain record for a spending proposal.
    #[derive(
        Clone,
        Encode,
        Decode,
        DecodeWithMemTracking,
        Eq,
        PartialEq,
        TypeInfo,
        MaxEncodedLen,
    )]
    #[scale_info(skip_type_params(T))]
    pub struct ProposalRecord<T: Config> {
        pub title: BoundedVec<u8, ConstU32<MAX_TITLE_LEN>>,
        pub description: BoundedVec<u8, ConstU32<MAX_DESC_LEN>>,
        pub class: ProposalClass,
        pub action: GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>,
        pub approved_at: Option<BlockNumberFor<T>>,
        pub status: ProposalStatus,
        pub submitted_at: BlockNumberFor<T>,
        /// Block number after which tally may be called.
        pub vote_end: BlockNumberFor<T>,
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The balance type used for proposal amounts.
        type Balance: Parameter + Copy + Default + MaxEncodedLen;

        /// Cross-pallet member eligibility check.
        type Membership: super::MembershipChecker<Self::AccountId>;

        /// Cross-pallet treasury disbursement.
        type Treasury: super::TreasuryHandler<Self::AccountId, Self::Balance>;

        /// Cross-pallet membership-governance setter bridge.
        type MembershipGovernance: super::MembershipGovernance<OriginFor<Self>, BlockNumberFor<Self>>;

        /// PalletId-derived sovereign account used as GovernanceOrigin.
        #[pallet::constant]
        type GovernancePalletId: Get<PalletId>;
    }

    // ---------------------------------------------------------------------------
    // Storage
    // ---------------------------------------------------------------------------

    /// Monotonically increasing counter used to generate ProposalIds.
    #[pallet::storage]
    pub type ProposalCount<T: Config> = StorageValue<_, ProposalId, ValueQuery>;

    /// All proposals keyed by their id.
    #[pallet::storage]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalId, ProposalRecord<T>, OptionQuery>;

    /// Votes cast per proposal and voter. `(proposal_id, voter) → approve`.
    #[pallet::storage]
    pub type ProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ProposalId,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    /// Number of yes votes received by each proposal.
    #[pallet::storage]
    pub type ProposalYesCount<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalId, u32, ValueQuery>;

    /// Number of no votes received by each proposal.
    #[pallet::storage]
    pub type ProposalNoCount<T: Config> =
        StorageMap<_, Blake2_128Concat, ProposalId, u32, ValueQuery>;

    /// Number of blocks a proposal's voting window stays open.
    #[pallet::storage]
    pub type ProposalVotingPeriod<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Delay between proposal approval and execution eligibility.
    #[pallet::storage]
    pub type ExecutionDelay<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Numerator for standard proposal approval threshold.
    #[pallet::storage]
    pub type StandardApprovalNumerator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Denominator for standard proposal approval threshold.
    #[pallet::storage]
    pub type StandardApprovalDenominator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Numerator for governance proposal approval threshold.
    #[pallet::storage]
    pub type GovernanceApprovalNumerator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Denominator for governance proposal approval threshold.
    #[pallet::storage]
    pub type GovernanceApprovalDenominator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Numerator for constitutional proposal approval threshold.
    #[pallet::storage]
    pub type ConstitutionalApprovalNumerator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Denominator for constitutional proposal approval threshold.
    #[pallet::storage]
    pub type ConstitutionalApprovalDenominator<T: Config> = StorageValue<_, u32, ValueQuery>;

    // ---------------------------------------------------------------------------
    // Genesis
    // ---------------------------------------------------------------------------

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub proposal_voting_period: BlockNumberFor<T>,
        pub execution_delay: BlockNumberFor<T>,
        pub standard_approval_numerator: u32,
        pub standard_approval_denominator: u32,
        pub governance_approval_numerator: u32,
        pub governance_approval_denominator: u32,
        pub constitutional_approval_numerator: u32,
        pub constitutional_approval_denominator: u32,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                proposal_voting_period: default_proposal_voting_period::<T>(),
                execution_delay: 0u32.saturated_into(),
                standard_approval_numerator: 1,
                standard_approval_denominator: 2,
                governance_approval_numerator: 4,
                governance_approval_denominator: 5,
                constitutional_approval_numerator: 9,
                constitutional_approval_denominator: 10,
            }
        }
    }

    #[cfg(feature = "fast-local")]
    fn default_proposal_voting_period<T: Config>() -> BlockNumberFor<T> {
        20u32.saturated_into()
    }

    #[cfg(not(feature = "fast-local"))]
    fn default_proposal_voting_period<T: Config>() -> BlockNumberFor<T> {
        100_800u32.saturated_into()
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            ProposalVotingPeriod::<T>::put(self.proposal_voting_period);
            ExecutionDelay::<T>::put(self.execution_delay);
            StandardApprovalNumerator::<T>::put(self.standard_approval_numerator);
            StandardApprovalDenominator::<T>::put(self.standard_approval_denominator);
            GovernanceApprovalNumerator::<T>::put(self.governance_approval_numerator);
            GovernanceApprovalDenominator::<T>::put(self.governance_approval_denominator);
            ConstitutionalApprovalNumerator::<T>::put(self.constitutional_approval_numerator);
            ConstitutionalApprovalDenominator::<T>::put(self.constitutional_approval_denominator);
        }
    }

    // ---------------------------------------------------------------------------
    // Runtime upgrades
    // ---------------------------------------------------------------------------

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_runtime_upgrade() -> Weight {
            let on_chain = <Pallet<T> as frame_support::traits::GetStorageVersion>::on_chain_storage_version();
            if on_chain >= STORAGE_VERSION {
                return Weight::zero();
            }

            // Account for on_chain_storage_version() storage read above.
            let mut reads = 1u64;
            let mut writes = 0u64;

            reads = reads.saturating_add(1);
            if !ProposalVotingPeriod::<T>::exists() {
                ProposalVotingPeriod::<T>::put(default_proposal_voting_period::<T>());
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !ExecutionDelay::<T>::exists() {
                ExecutionDelay::<T>::put(BlockNumberFor::<T>::default());
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !StandardApprovalNumerator::<T>::exists() {
                StandardApprovalNumerator::<T>::put(1);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !StandardApprovalDenominator::<T>::exists() {
                StandardApprovalDenominator::<T>::put(2);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !GovernanceApprovalNumerator::<T>::exists() {
                GovernanceApprovalNumerator::<T>::put(4);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !GovernanceApprovalDenominator::<T>::exists() {
                GovernanceApprovalDenominator::<T>::put(5);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !ConstitutionalApprovalNumerator::<T>::exists() {
                ConstitutionalApprovalNumerator::<T>::put(9);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !ConstitutionalApprovalDenominator::<T>::exists() {
                ConstitutionalApprovalDenominator::<T>::put(10);
                writes = writes.saturating_add(1);
            }

            STORAGE_VERSION.put::<Pallet<T>>();
            writes = writes.saturating_add(1);

            T::DbWeight::get().reads_writes(reads, writes)
        }
    }

    // ---------------------------------------------------------------------------
    // Events
    // ---------------------------------------------------------------------------

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new proposal was submitted.
        ProposalSubmitted {
            proposal_id: ProposalId,
            organizer: T::AccountId,
        },
        /// An active member cast a vote on a proposal.
        VoteCast {
            proposal_id: ProposalId,
            voter: T::AccountId,
            approve: bool,
        },
        /// Tally completed with a yes majority — proposal is approved.
        ProposalApproved { proposal_id: ProposalId },
        /// Tally completed without a yes majority — proposal is rejected.
        ProposalRejected { proposal_id: ProposalId },
        /// An approved proposal was executed and funds disbursed.
        ProposalExecuted { proposal_id: ProposalId },
        /// Proposal voting period parameter was updated.
        ProposalVotingPeriodSet { blocks: BlockNumberFor<T> },
        /// Proposal execution delay parameter was updated.
        ExecutionDelaySet { blocks: BlockNumberFor<T> },
        /// Standard proposal approval threshold was updated.
        StandardThresholdSet { numerator: u32, denominator: u32 },
        /// Governance proposal approval threshold was updated.
        GovernanceThresholdSet { numerator: u32, denominator: u32 },
        /// Constitutional proposal approval threshold was updated.
        ConstitutionalThresholdSet { numerator: u32, denominator: u32 },
    }

    // ---------------------------------------------------------------------------
    // Errors
    // ---------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
        /// The caller is not a registered active member.
        NotActiveMember,
        /// No proposal exists with the given id.
        ProposalNotFound,
        /// The voter has already cast a vote on this proposal.
        AlreadyVoted,
        /// The voting window has closed; votes are no longer accepted.
        VotingClosed,
        /// The voting window has not yet closed; tally is not available.
        VotingStillOpen,
        /// The proposal is not in the Active state (already tallied or executed).
        ProposalNotActive,
        /// The proposal is not in the Approved state and cannot be executed.
        ProposalNotApproved,
        /// The proposal has already been executed. (I-3)
        ProposalAlreadyExecuted,
        /// The caller is not the organizer of the proposal.
        NotOrganizer,
        /// The supplied title exceeds the maximum allowed length.
        TitleTooLong,
        /// The supplied description exceeds the maximum allowed length.
        DescriptionTooLong,
        /// Invalid threshold fraction.
        InvalidThreshold,
        /// Proposal class does not match action requirements.
        ProposalClassMismatch,
        /// Setter was called by an origin other than the governance account.
        NotGovernanceOrigin,
    }

    // ---------------------------------------------------------------------------
    // Dispatchables
    // ---------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a new spending proposal.
        ///
        /// The caller must be an active member. Creates a proposal in the
        /// `Active` state with a voting window of `ProposalVotingPeriod` blocks.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 3))]
        pub fn submit_proposal(
            origin: OriginFor<T>,
            title: BoundedVec<u8, ConstU32<MAX_TITLE_LEN>>,
            description: BoundedVec<u8, ConstU32<MAX_DESC_LEN>>,
            class: ProposalClass,
            action: GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>,
        ) -> DispatchResult {
            let organizer = ensure_signed(origin)?;
            ensure!(
                T::Membership::is_active_member(&organizer),
                Error::<T>::NotActiveMember
            );
            ensure!(
                class == Self::required_class_for_action(&action),
                Error::<T>::ProposalClassMismatch
            );

            let now = frame_system::Pallet::<T>::block_number();
            let vote_end = now.saturating_add(ProposalVotingPeriod::<T>::get());

            let id = ProposalCount::<T>::mutate(|c| {
                *c = c.saturating_add(1);
                *c
            });

            let record = ProposalRecord::<T> {
                title,
                description,
                class,
                action,
                approved_at: None,
                status: ProposalStatus::Active,
                submitted_at: now,
                vote_end,
            };
            Proposals::<T>::insert(id, record);

            Self::deposit_event(Event::ProposalSubmitted {
                proposal_id: id,
                organizer,
            });

            Ok(())
        }

        /// Cast a yes or no vote on an active proposal.
        ///
        /// The caller must be an active member (I-2). Voting is only allowed
        /// while the voting window is open (`current_block ≤ vote_end`).
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(4, 3))]
        pub fn vote_on_proposal(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            approve: bool,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;
            // I-2: only active members may vote.
            ensure!(
                T::Membership::is_active_member(&voter),
                Error::<T>::NotActiveMember
            );

            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now <= proposal.vote_end, Error::<T>::VotingClosed);

            ensure!(
                !ProposalVotes::<T>::contains_key(proposal_id, &voter),
                Error::<T>::AlreadyVoted
            );

            ProposalVotes::<T>::insert(proposal_id, &voter, approve);

            if approve {
                ProposalYesCount::<T>::mutate(proposal_id, |c| *c = c.saturating_add(1));
            } else {
                ProposalNoCount::<T>::mutate(proposal_id, |c| *c = c.saturating_add(1));
            }

            Self::deposit_event(Event::VoteCast {
                proposal_id,
                voter,
                approve,
            });

            Ok(())
        }

        /// Tally the votes on a proposal after its voting window has closed.
        ///
        /// Anyone may call this. The proposal must be Active and the voting
        /// window must have ended. Simple majority (yes > no) → Approved.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(3, 1))]
        pub fn tally_proposal(origin: OriginFor<T>, proposal_id: ProposalId) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == ProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now > proposal.vote_end, Error::<T>::VotingStillOpen);

            let yes = ProposalYesCount::<T>::get(proposal_id);
            let no = ProposalNoCount::<T>::get(proposal_id);
            let total = yes.saturating_add(no);
            let (num, den) = match proposal.class {
                ProposalClass::Standard => (
                    StandardApprovalNumerator::<T>::get(),
                    StandardApprovalDenominator::<T>::get(),
                ),
                ProposalClass::Governance => (
                    GovernanceApprovalNumerator::<T>::get(),
                    GovernanceApprovalDenominator::<T>::get(),
                ),
                ProposalClass::Constitutional => (
                    ConstitutionalApprovalNumerator::<T>::get(),
                    ConstitutionalApprovalDenominator::<T>::get(),
                ),
            };

            if yes.saturating_mul(den) >= total.saturating_mul(num) {
                proposal.status = ProposalStatus::Approved;
                proposal.approved_at = Some(now);
                Proposals::<T>::insert(proposal_id, proposal);
                Self::deposit_event(Event::ProposalApproved { proposal_id });
            } else {
                proposal.status = ProposalStatus::Rejected;
                Proposals::<T>::insert(proposal_id, proposal);
                Self::deposit_event(Event::ProposalRejected { proposal_id });
            }

            Ok(())
        }

        /// Execute an approved proposal by disbursing funds from the treasury.
        ///
        /// The proposal must be in the `Approved` state. After a successful
        /// disbursement the status transitions to `Executed`. A second call
        /// returns `ProposalNotApproved` (I-3).
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 1))]
        pub fn execute_proposal(origin: OriginFor<T>, proposal_id: ProposalId) -> DispatchResult {
            ensure_signed(origin)?;

            let mut proposal =
                Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            // I-3: reject if already executed.
            if proposal.status == ProposalStatus::Executed {
                return Err(Error::<T>::ProposalAlreadyExecuted.into());
            }

            ensure!(
                proposal.status == ProposalStatus::Approved,
                Error::<T>::ProposalNotApproved
            );

            let governance_account = T::GovernancePalletId::get().into_account_truncating();
            let governance_origin = RawOrigin::Signed(governance_account).into();
            Self::dispatch_governance_action(governance_origin, &proposal.action)?;

            proposal.status = ProposalStatus::Executed;
            Proposals::<T>::insert(proposal_id, &proposal);

            Self::deposit_event(Event::ProposalExecuted { proposal_id });

            Ok(())
        }

        /// Update the proposal voting period.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn set_proposal_voting_period(
            origin: OriginFor<T>,
            blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            ProposalVotingPeriod::<T>::put(blocks);
            Self::deposit_event(Event::ProposalVotingPeriodSet { blocks });
            Ok(())
        }

        /// Update the proposal execution delay.
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn set_execution_delay(
            origin: OriginFor<T>,
            blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            ExecutionDelay::<T>::put(blocks);
            Self::deposit_event(Event::ExecutionDelaySet { blocks });
            Ok(())
        }

        /// Update the standard proposal approval threshold.
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
        pub fn set_standard_approval_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            Self::ensure_valid_threshold(numerator, denominator)?;
            StandardApprovalNumerator::<T>::put(numerator);
            StandardApprovalDenominator::<T>::put(denominator);
            Self::deposit_event(Event::StandardThresholdSet {
                numerator,
                denominator,
            });
            Ok(())
        }

        /// Update the governance proposal approval threshold.
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
        pub fn set_governance_approval_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            Self::ensure_valid_threshold(numerator, denominator)?;
            GovernanceApprovalNumerator::<T>::put(numerator);
            GovernanceApprovalDenominator::<T>::put(denominator);
            Self::deposit_event(Event::GovernanceThresholdSet {
                numerator,
                denominator,
            });
            Ok(())
        }

        /// Update the constitutional proposal approval threshold.
        #[pallet::call_index(8)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
        pub fn set_constitutional_approval_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            Self::ensure_valid_threshold(numerator, denominator)?;
            ConstitutionalApprovalNumerator::<T>::put(numerator);
            ConstitutionalApprovalDenominator::<T>::put(denominator);
            Self::deposit_event(Event::ConstitutionalThresholdSet {
                numerator,
                denominator,
            });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn required_class_for_action(
            action: &GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>,
        ) -> ProposalClass {
            match action {
                GovernanceAction::DisburseToAccount { .. } => ProposalClass::Standard,
                GovernanceAction::SetProposalVotingPeriod { .. }
                | GovernanceAction::SetExecutionDelay { .. }
                | GovernanceAction::SetMembershipVotingPeriod { .. } => ProposalClass::Governance,
                GovernanceAction::SetStandardApprovalThreshold { .. }
                | GovernanceAction::SetGovernanceApprovalThreshold { .. }
                | GovernanceAction::SetConstitutionalApprovalThreshold { .. }
                | GovernanceAction::SetMembershipApprovalThreshold { .. }
                | GovernanceAction::SetSuspensionThreshold { .. } => {
                    ProposalClass::Constitutional
                }
            }
        }

        fn ensure_governance_origin(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let expected = T::GovernancePalletId::get().into_account_truncating();
            ensure!(caller == expected, Error::<T>::NotGovernanceOrigin);
            Ok(())
        }

        fn dispatch_governance_action(
            governance_origin: OriginFor<T>,
            action: &GovernanceAction<T::AccountId, T::Balance, BlockNumberFor<T>>,
        ) -> DispatchResult {
            match action {
                GovernanceAction::DisburseToAccount { recipient, amount } => {
                    T::Treasury::disburse(recipient, *amount)
                }
                GovernanceAction::SetProposalVotingPeriod { blocks } => {
                    Self::set_proposal_voting_period(governance_origin, *blocks)
                }
                GovernanceAction::SetExecutionDelay { blocks } => {
                    Self::set_execution_delay(governance_origin, *blocks)
                }
                GovernanceAction::SetStandardApprovalThreshold {
                    numerator,
                    denominator,
                } => Self::set_standard_approval_threshold(
                    governance_origin,
                    *numerator,
                    *denominator,
                ),
                GovernanceAction::SetGovernanceApprovalThreshold {
                    numerator,
                    denominator,
                } => Self::set_governance_approval_threshold(
                    governance_origin,
                    *numerator,
                    *denominator,
                ),
                GovernanceAction::SetConstitutionalApprovalThreshold {
                    numerator,
                    denominator,
                } => Self::set_constitutional_approval_threshold(
                    governance_origin,
                    *numerator,
                    *denominator,
                ),
                GovernanceAction::SetMembershipVotingPeriod { blocks } => {
                    T::MembershipGovernance::set_voting_period(governance_origin, *blocks)
                }
                GovernanceAction::SetMembershipApprovalThreshold {
                    numerator,
                    denominator,
                } => T::MembershipGovernance::set_approval_threshold(
                    governance_origin,
                    *numerator,
                    *denominator,
                ),
                GovernanceAction::SetSuspensionThreshold {
                    numerator,
                    denominator,
                } => T::MembershipGovernance::set_suspension_threshold(
                    governance_origin,
                    *numerator,
                    *denominator,
                ),
            }
        }

        fn ensure_valid_threshold(numerator: u32, denominator: u32) -> DispatchResult {
            ensure!(denominator != 0, Error::<T>::InvalidThreshold);
            ensure!(numerator <= denominator, Error::<T>::InvalidThreshold);
            Ok(())
        }
    }
}

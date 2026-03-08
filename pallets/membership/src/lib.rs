//! # Membership Pallet
//!
//! Maintains the authoritative set of active members for the GAIA network.
//! Exposes `is_active_member(AccountId) -> bool` for use by other pallets.
//!
//! ## Overview
//!
//! - A **member record** holds an account address, a name (max 128 bytes),
//!   a status (active or suspended), and a join timestamp (block number).
//! - Membership is requested through **membership proposals** with explicit
//!   lifecycle states: `Active -> Approved/Rejected`.
//! - Active members propose candidates and vote on proposals.
//! - The approval threshold is storage-backed and checked against a submit-time
//!   active-member snapshot. Genesis default is 80% (`4/5`).
//! - Each proposal has a voting deadline (`vote_end`) and cannot remain active
//!   forever; active members finalize after the deadline.
//! - Suspended members cannot propose, vote, or finalize membership proposals.
//!
//! ### Suspension (implemented)
//!
//! Suspension mechanics are active in the current pallet implementation. Two
//! paths are supported:
//!
//! - **Self-initiated**: a member voluntarily suspends themselves.
//! - **Unanimous peer vote**: all other active members vote to suspend
//!   (see ADR `docs/decisions/005-suspension-unanimity.md`).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Trait that other pallets use to check membership status.
///
/// The runtime wires a concrete implementation; downstream pallets
/// depend only on this trait, never on the membership pallet directly.
pub trait MembershipChecker<AccountId> {
    fn is_active_member(account: &AccountId) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::sp_runtime::traits::AccountIdConversion;
    use frame_support::traits::StorageVersion;
    use frame_support::sp_runtime::{traits::SaturatedConversion, Saturating};
    use frame_support::PalletId;
    use frame_system::pallet_prelude::*;

    /// Maximum length of a member name in bytes.
    pub const MAX_NAME_LEN: u32 = 128;

    /// On-chain identifier for membership proposals.
    pub type MembershipProposalId = u32;

    /// Status of a member.
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum MemberStatus {
        Active,
        Suspended,
    }

    /// Lifecycle state of a membership proposal.
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum MembershipProposalStatus {
        Active,
        Approved,
        Rejected,
    }

    /// Reason why a member was suspended.
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
    pub enum SuspensionReason {
        SelfInitiated,
        PeerVote,
    }

    /// On-chain record for a registered member.
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct MemberRecord<T: Config> {
        pub name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
        pub status: MemberStatus,
        pub joined_at: BlockNumberFor<T>,
    }

    /// On-chain record for a membership proposal.
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct MembershipProposalRecord<T: Config> {
        pub candidate: T::AccountId,
        pub name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
        pub proposed_by: T::AccountId,
        pub proposed_at: BlockNumberFor<T>,
        /// Block number after which `finalize_proposal` is allowed.
        pub vote_end: BlockNumberFor<T>,
        /// Number of active members snapshotted at submission time.
        pub active_member_snapshot: u32,
        pub status: MembershipProposalStatus,
    }

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// PalletId-derived sovereign account used as GovernanceOrigin.
        #[pallet::constant]
        type GovernancePalletId: Get<PalletId>;
    }

    // ---------------------------------------------------------------------------
    // Storage
    // ---------------------------------------------------------------------------

    /// Map of registered members keyed by account id.
    #[pallet::storage]
    pub type Members<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, MemberRecord<T>, OptionQuery>;

    /// Number of currently active members.
    #[pallet::storage]
    pub type ActiveMemberCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Monotonically increasing counter used to generate MembershipProposalIds.
    #[pallet::storage]
    pub type MembershipProposalCount<T: Config> = StorageValue<_, MembershipProposalId, ValueQuery>;

    /// All membership proposals keyed by proposal id.
    #[pallet::storage]
    pub type MembershipProposals<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        MembershipProposalId,
        MembershipProposalRecord<T>,
        OptionQuery,
    >;

    /// Active membership proposal keyed by candidate account.
    #[pallet::storage]
    pub type ActiveProposalByCandidate<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, MembershipProposalId, OptionQuery>;

    /// Votes cast per membership proposal and voter. `(proposal_id, voter) -> approve`.
    #[pallet::storage]
    pub type MembershipProposalVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        MembershipProposalId,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    /// Number of yes votes received by each membership proposal.
    #[pallet::storage]
    pub type MembershipProposalYesCount<T: Config> =
        StorageMap<_, Blake2_128Concat, MembershipProposalId, u32, ValueQuery>;

    /// Number of no votes received by each membership proposal.
    #[pallet::storage]
    pub type MembershipProposalNoCount<T: Config> =
        StorageMap<_, Blake2_128Concat, MembershipProposalId, u32, ValueQuery>;

    /// Suspension votes cast per target member and voter.
    #[pallet::storage]
    pub type SuspensionVotes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    /// Number of approval votes received by each suspension target.
    #[pallet::storage]
    pub type SuspensionApprovalCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

    /// Number of blocks a membership proposal stays open for voting.
    #[pallet::storage]
    pub type MembershipVotingPeriod<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// Numerator for membership proposal approval threshold.
    #[pallet::storage]
    pub type MembershipApprovalNumerator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Denominator for membership proposal approval threshold.
    #[pallet::storage]
    pub type MembershipApprovalDenominator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Numerator for suspension threshold.
    #[pallet::storage]
    pub type SuspensionNumerator<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Denominator for suspension threshold.
    #[pallet::storage]
    pub type SuspensionDenominator<T: Config> = StorageValue<_, u32, ValueQuery>;

    // ---------------------------------------------------------------------------
    // Genesis
    // ---------------------------------------------------------------------------

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T: Config> {
        /// Initial members seeded at genesis: `(account, name_bytes)`.
        pub initial_members: sp_runtime::BoundedVec<
            (T::AccountId, BoundedVec<u8, ConstU32<MAX_NAME_LEN>>),
            ConstU32<100>,
        >,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let mut count = 0u32;
            for (account, name) in &self.initial_members {
                let record = MemberRecord::<T> {
                    name: name.clone(),
                    status: MemberStatus::Active,
                    joined_at: BlockNumberFor::<T>::default(),
                };
                Members::<T>::insert(account, record);
                count = count.saturating_add(1);
            }
            ActiveMemberCount::<T>::put(count);
            MembershipVotingPeriod::<T>::put(default_membership_voting_period::<T>());
            MembershipApprovalNumerator::<T>::put(4);
            MembershipApprovalDenominator::<T>::put(5);
            SuspensionNumerator::<T>::put(1);
            SuspensionDenominator::<T>::put(1);
        }
    }

    #[cfg(feature = "fast-local")]
    fn default_membership_voting_period<T: Config>() -> BlockNumberFor<T> {
        20u32.saturated_into()
    }

    #[cfg(not(feature = "fast-local"))]
    fn default_membership_voting_period<T: Config>() -> BlockNumberFor<T> {
        100_800u32.saturated_into()
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
            if !MembershipVotingPeriod::<T>::exists() {
                MembershipVotingPeriod::<T>::put(default_membership_voting_period::<T>());
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !MembershipApprovalNumerator::<T>::exists() {
                MembershipApprovalNumerator::<T>::put(4);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !MembershipApprovalDenominator::<T>::exists() {
                MembershipApprovalDenominator::<T>::put(5);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !SuspensionNumerator::<T>::exists() {
                SuspensionNumerator::<T>::put(1);
                writes = writes.saturating_add(1);
            }

            reads = reads.saturating_add(1);
            if !SuspensionDenominator::<T>::exists() {
                SuspensionDenominator::<T>::put(1);
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
        /// A membership proposal was created.
        MemberProposalSubmitted {
            proposal_id: MembershipProposalId,
            candidate: T::AccountId,
            proposed_by: T::AccountId,
            vote_end: BlockNumberFor<T>,
        },
        /// An active member voted on a membership proposal.
        MemberProposalVoteCast {
            proposal_id: MembershipProposalId,
            candidate: T::AccountId,
            voter: T::AccountId,
            approve: bool,
        },
        /// A membership proposal was approved and candidate became active member.
        MemberProposalApproved {
            proposal_id: MembershipProposalId,
            member: T::AccountId,
        },
        /// A membership proposal was rejected after its voting window.
        MemberProposalRejected {
            proposal_id: MembershipProposalId,
            candidate: T::AccountId,
        },
        /// A suspension vote was cast for a target member.
        SuspensionVoteCast {
            target: T::AccountId,
            voter: T::AccountId,
            approve: bool,
        },
        /// A member has been suspended.
        MemberSuspended {
            member: T::AccountId,
            reason: SuspensionReason,
        },
        /// Membership voting period was updated.
        MembershipVotingPeriodSet { blocks: BlockNumberFor<T> },
        /// Membership approval threshold was updated.
        MembershipApprovalThresholdSet { numerator: u32, denominator: u32 },
        /// Suspension threshold was updated.
        SuspensionThresholdSet { numerator: u32, denominator: u32 },
    }

    // ---------------------------------------------------------------------------
    // Errors
    // ---------------------------------------------------------------------------

    #[pallet::error]
    pub enum Error<T> {
        /// The caller is not a registered active member.
        NotActiveMember,
        /// The candidate is already a registered member.
        AlreadyMember,
        /// An active proposal for this candidate already exists.
        CandidateAlreadyProposed,
        /// No proposal exists with the given id.
        ProposalNotFound,
        /// The supplied proposal is not active.
        ProposalNotActive,
        /// The voter has already cast a vote for this proposal.
        AlreadyVoted,
        /// The voting window has closed; votes are no longer accepted.
        VotingClosed,
        /// The voting window has not yet closed; finalize is not available.
        VotingStillOpen,
        /// The supplied name exceeds the maximum allowed length.
        NameTooLong,
        /// The caller is suspended and cannot perform this action.
        MemberSuspended,
        /// The target member is already suspended.
        AlreadySuspended,
        /// A member cannot cast a peer suspension vote against themselves.
        CannotSuspendSelf,
        /// Invalid threshold fraction.
        InvalidThreshold,
        /// Setter was called by an origin other than the governance account.
        NotGovernanceOrigin,
    }

    // ---------------------------------------------------------------------------
    // Dispatchables
    // ---------------------------------------------------------------------------

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Propose a new candidate for membership.
        ///
        /// The caller must be an active member. The candidate must not already
        /// be a member or have another active membership proposal.
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(3, 4))]
        pub fn propose_member(
            origin: OriginFor<T>,
            candidate: T::AccountId,
            name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            Self::ensure_active_member(&proposer)?;

            ensure!(
                !Members::<T>::contains_key(&candidate),
                Error::<T>::AlreadyMember
            );
            ensure!(
                !ActiveProposalByCandidate::<T>::contains_key(&candidate),
                Error::<T>::CandidateAlreadyProposed
            );

            let now = frame_system::Pallet::<T>::block_number();
            let vote_end = now.saturating_add(MembershipVotingPeriod::<T>::get());
            let active_member_snapshot = ActiveMemberCount::<T>::get();

            let proposal_id = MembershipProposalCount::<T>::mutate(|count| {
                *count = count.saturating_add(1);
                *count
            });

            let record = MembershipProposalRecord::<T> {
                candidate: candidate.clone(),
                name,
                proposed_by: proposer.clone(),
                proposed_at: now,
                vote_end,
                active_member_snapshot,
                status: MembershipProposalStatus::Active,
            };

            MembershipProposals::<T>::insert(proposal_id, record);
            ActiveProposalByCandidate::<T>::insert(&candidate, proposal_id);

            Self::deposit_event(Event::MemberProposalSubmitted {
                proposal_id,
                candidate,
                proposed_by: proposer,
                vote_end,
            });

            Ok(())
        }

        /// Vote to approve or reject an active membership proposal.
        ///
        /// The caller must be an active member and must not have already voted
        /// on this proposal. If the approval threshold is reached, the proposal
        /// is approved immediately and candidate is admitted.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(7, 6))]
        pub fn vote_on_candidate(
            origin: OriginFor<T>,
            proposal_id: MembershipProposalId,
            approve: bool,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;
            Self::ensure_active_member(&voter)?;

            let proposal =
                MembershipProposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == MembershipProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now <= proposal.vote_end, Error::<T>::VotingClosed);

            ensure!(
                !MembershipProposalVotes::<T>::contains_key(proposal_id, &voter),
                Error::<T>::AlreadyVoted
            );

            MembershipProposalVotes::<T>::insert(proposal_id, &voter, approve);

            if approve {
                MembershipProposalYesCount::<T>::mutate(proposal_id, |count| {
                    *count = count.saturating_add(1)
                });
            } else {
                MembershipProposalNoCount::<T>::mutate(proposal_id, |count| {
                    *count = count.saturating_add(1)
                });
            }

            Self::deposit_event(Event::MemberProposalVoteCast {
                proposal_id,
                candidate: proposal.candidate.clone(),
                voter,
                approve,
            });

            if approve
                && Self::meets_membership_threshold(
                    proposal_id,
                    proposal.active_member_snapshot,
                )
            {
                Self::approve_membership_proposal(proposal_id, proposal)?;
            }

            Ok(())
        }

        /// Finalize an active membership proposal after its voting window.
        ///
        /// The caller must be an active member. Active proposals that met the
        /// threshold are approved; otherwise they are rejected.
        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(4, 3))]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: MembershipProposalId,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            Self::ensure_active_member(&caller)?;

            let mut proposal =
                MembershipProposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == MembershipProposalStatus::Active,
                Error::<T>::ProposalNotActive
            );

            let now = frame_system::Pallet::<T>::block_number();
            ensure!(now > proposal.vote_end, Error::<T>::VotingStillOpen);

            if Self::meets_membership_threshold(proposal_id, proposal.active_member_snapshot) {
                return Self::approve_membership_proposal(proposal_id, proposal);
            }

            proposal.status = MembershipProposalStatus::Rejected;
            ActiveProposalByCandidate::<T>::remove(&proposal.candidate);
            MembershipProposals::<T>::insert(proposal_id, proposal.clone());

            Self::deposit_event(Event::MemberProposalRejected {
                proposal_id,
                candidate: proposal.candidate,
            });

            Ok(())
        }

        /// Suspend the caller's own member account.
        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 4))]
        pub fn suspend_self(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            Self::suspend_member(&caller, SuspensionReason::SelfInitiated)
        }

        /// Vote to suspend an active member.
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(7, 7))]
        pub fn vote_suspend_member(
            origin: OriginFor<T>,
            target: T::AccountId,
            approve: bool,
        ) -> DispatchResult {
            let voter = ensure_signed(origin)?;
            Self::ensure_active_member(&voter)?;
            ensure!(voter != target, Error::<T>::CannotSuspendSelf);

            // Check the target is an active member. Use an explicit lookup so
            // that an already-suspended target returns the accurate error.
            let target_record =
                Members::<T>::get(&target).ok_or(Error::<T>::NotActiveMember)?;
            ensure!(
                target_record.status == MemberStatus::Active,
                Error::<T>::AlreadySuspended
            );

            ensure!(
                !SuspensionVotes::<T>::contains_key(&target, &voter),
                Error::<T>::AlreadyVoted
            );

            SuspensionVotes::<T>::insert(&target, &voter, approve);
            Self::deposit_event(Event::SuspensionVoteCast {
                target: target.clone(),
                voter,
                approve,
            });

            if approve {
                let approvals = SuspensionApprovalCount::<T>::mutate(&target, |count| {
                    *count = count.saturating_add(1);
                    *count
                });

                let n = SuspensionNumerator::<T>::get();
                let d = SuspensionDenominator::<T>::get();
                let others = ActiveMemberCount::<T>::get().saturating_sub(1);

                if approvals.saturating_mul(d) >= others.saturating_mul(n) {
                    Self::suspend_member(&target, SuspensionReason::PeerVote)?;
                }
            }

            Ok(())
        }

        /// Update the membership proposal voting period.
        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
        pub fn set_membership_voting_period(
            origin: OriginFor<T>,
            blocks: BlockNumberFor<T>,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            MembershipVotingPeriod::<T>::put(blocks);
            Self::deposit_event(Event::MembershipVotingPeriodSet { blocks });
            Ok(())
        }

        /// Update the membership proposal approval threshold.
        #[pallet::call_index(6)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
        pub fn set_membership_approval_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            Self::ensure_valid_threshold(numerator, denominator)?;
            MembershipApprovalNumerator::<T>::put(numerator);
            MembershipApprovalDenominator::<T>::put(denominator);
            Self::deposit_event(Event::MembershipApprovalThresholdSet {
                numerator,
                denominator,
            });
            Ok(())
        }

        /// Update the suspension threshold.
        #[pallet::call_index(7)]
        #[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(2))]
        pub fn set_suspension_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::ensure_governance_origin(origin)?;
            Self::ensure_valid_threshold(numerator, denominator)?;
            SuspensionNumerator::<T>::put(numerator);
            SuspensionDenominator::<T>::put(denominator);
            Self::deposit_event(Event::SuspensionThresholdSet {
                numerator,
                denominator,
            });
            Ok(())
        }
    }

    // ---------------------------------------------------------------------------
    // Internal helpers
    // ---------------------------------------------------------------------------

    impl<T: Config> Pallet<T> {
        /// Returns `Ok(())` if `who` is an active member, otherwise an error.
        fn ensure_active_member(who: &T::AccountId) -> DispatchResult {
            let record = Members::<T>::get(who).ok_or(Error::<T>::NotActiveMember)?;
            ensure!(
                record.status == MemberStatus::Active,
                Error::<T>::MemberSuspended
            );
            Ok(())
        }

        fn meets_membership_threshold(
            proposal_id: MembershipProposalId,
            active_member_snapshot: u32,
        ) -> bool {
            let yes_votes = MembershipProposalYesCount::<T>::get(proposal_id);
            let n = MembershipApprovalNumerator::<T>::get();
            let d = MembershipApprovalDenominator::<T>::get();
            yes_votes.saturating_mul(d) >= active_member_snapshot.saturating_mul(n)
        }

        fn ensure_valid_threshold(numerator: u32, denominator: u32) -> DispatchResult {
            ensure!(denominator != 0, Error::<T>::InvalidThreshold);
            ensure!(numerator <= denominator, Error::<T>::InvalidThreshold);
            Ok(())
        }

        fn ensure_governance_origin(origin: OriginFor<T>) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            let expected = T::GovernancePalletId::get().into_account_truncating();
            ensure!(caller == expected, Error::<T>::NotGovernanceOrigin);
            Ok(())
        }

        fn approve_membership_proposal(
            proposal_id: MembershipProposalId,
            mut proposal: MembershipProposalRecord<T>,
        ) -> DispatchResult {
            let now = frame_system::Pallet::<T>::block_number();

            let member_record = MemberRecord::<T> {
                name: proposal.name.clone(),
                status: MemberStatus::Active,
                joined_at: now,
            };

            Members::<T>::insert(&proposal.candidate, member_record);
            ActiveMemberCount::<T>::mutate(|count| *count = count.saturating_add(1));

            proposal.status = MembershipProposalStatus::Approved;
            ActiveProposalByCandidate::<T>::remove(&proposal.candidate);
            MembershipProposals::<T>::insert(proposal_id, proposal.clone());

            Self::deposit_event(Event::MemberProposalApproved {
                proposal_id,
                member: proposal.candidate,
            });

            Ok(())
        }

        /// Suspend an active member and clean up suspension vote storage.
        ///
        /// Clears **all** pending suspension votes and counts — not just those
        /// targeting this member — because any change in the active-member pool
        /// invalidates prior unanimity calculations (see ADR-005).
        fn suspend_member(member: &T::AccountId, reason: SuspensionReason) -> DispatchResult {
            Members::<T>::try_mutate(member, |record| -> DispatchResult {
                let record = record.as_mut().ok_or(Error::<T>::NotActiveMember)?;
                ensure!(
                    record.status == MemberStatus::Active,
                    Error::<T>::AlreadySuspended
                );
                record.status = MemberStatus::Suspended;
                Ok(())
            })?;

            ActiveMemberCount::<T>::mutate(|count| *count = count.saturating_sub(1));

            // Invalidate every pending suspension vote: the voter pool has
            // changed so the unanimity threshold is no longer meaningful.
            let _ = SuspensionVotes::<T>::clear(u32::MAX, None);
            let _ = SuspensionApprovalCount::<T>::clear(u32::MAX, None);

            Self::deposit_event(Event::MemberSuspended {
                member: member.clone(),
                reason,
            });

            Ok(())
        }
    }

    // ---------------------------------------------------------------------------
    // MembershipChecker implementation
    // ---------------------------------------------------------------------------

    impl<T: Config> MembershipChecker<T::AccountId> for Pallet<T> {
        fn is_active_member(account: &T::AccountId) -> bool {
            Members::<T>::get(account)
                .map(|record| record.status == MemberStatus::Active)
                .unwrap_or(false)
        }
    }

    impl<T: Config> gaia_proposals::MembershipGovernance<OriginFor<T>, BlockNumberFor<T>>
        for Pallet<T>
    {
        fn set_voting_period(origin: OriginFor<T>, blocks: BlockNumberFor<T>) -> DispatchResult {
            Self::set_membership_voting_period(origin, blocks)
        }

        fn set_approval_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::set_membership_approval_threshold(origin, numerator, denominator)
        }

        fn set_suspension_threshold(
            origin: OriginFor<T>,
            numerator: u32,
            denominator: u32,
        ) -> DispatchResult {
            Self::set_suspension_threshold(origin, numerator, denominator)
        }
    }
}

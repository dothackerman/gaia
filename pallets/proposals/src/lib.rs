//! # Proposals Pallet
//!
//! Manages the full lifecycle of a spending proposal on the GAIA network:
//! **submission → voting → tally → approval/rejection → single execution**.
//!
//! ## Overview
//!
//! - Any active member may submit a proposal specifying a title, description,
//!   requested amount, and the target event block.
//! - Active members vote yes or no during the voting window
//!   (`submitted_at` … `submitted_at + VotingPeriod`).
//! - After the window closes, anyone may call `tally_proposal` to compute the
//!   result: simple majority (yes > no) → Approved, otherwise Rejected.
//! - An Approved proposal may be executed exactly once; execution transfers
//!   funds via `TreasuryHandler::disburse`.
//!
//! ## Invariants enforced
//!
//! - **I-2** Only active members may vote (`MembershipChecker` checked on every
//!   vote extrinsic).
//! - **I-3** A proposal executes at most once (status checked before execution;
//!   set to `Executed` on success).

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::dispatch::DispatchResult;

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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::Saturating;
	use frame_system::pallet_prelude::*;

	/// Maximum length of a proposal title in bytes.
	pub const MAX_TITLE_LEN: u32 = 128;

	/// Maximum length of a proposal description in bytes.
	pub const MAX_DESC_LEN: u32 = 1024;

	/// On-chain identifier for a proposal.
	pub type ProposalId = u32;

	/// Lifecycle state of a proposal.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
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

	/// On-chain record for a spending proposal.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct ProposalRecord<T: Config> {
		pub title: BoundedVec<u8, ConstU32<MAX_TITLE_LEN>>,
		pub description: BoundedVec<u8, ConstU32<MAX_DESC_LEN>>,
		pub amount: T::Balance,
		pub organizer: T::AccountId,
		pub event_block: BlockNumberFor<T>,
		pub status: ProposalStatus,
		pub submitted_at: BlockNumberFor<T>,
		/// Block number after which tally may be called.
		pub vote_end: BlockNumberFor<T>,
	}

	#[pallet::pallet]
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

		/// Number of blocks a proposal's voting window stays open.
		#[pallet::constant]
		type VotingPeriod: Get<BlockNumberFor<Self>>;
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
		ProposalApproved {
			proposal_id: ProposalId,
		},
		/// Tally completed without a yes majority — proposal is rejected.
		ProposalRejected {
			proposal_id: ProposalId,
		},
		/// An approved proposal was executed and funds disbursed.
		ProposalExecuted {
			proposal_id: ProposalId,
			organizer: T::AccountId,
			amount: T::Balance,
		},
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
		/// The supplied title exceeds the maximum allowed length.
		TitleTooLong,
		/// The supplied description exceeds the maximum allowed length.
		DescriptionTooLong,
	}

	// ---------------------------------------------------------------------------
	// Dispatchables
	// ---------------------------------------------------------------------------

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submit a new spending proposal.
		///
		/// The caller must be an active member. Creates a proposal in the
		/// `Active` state with a voting window of `VotingPeriod` blocks.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(2, 3))]
		pub fn submit_proposal(
			origin: OriginFor<T>,
			title: BoundedVec<u8, ConstU32<MAX_TITLE_LEN>>,
			description: BoundedVec<u8, ConstU32<MAX_DESC_LEN>>,
			amount: T::Balance,
			event_block: BlockNumberFor<T>,
		) -> DispatchResult {
			let organizer = ensure_signed(origin)?;
			ensure!(
				T::Membership::is_active_member(&organizer),
				Error::<T>::NotActiveMember
			);

			let now = frame_system::Pallet::<T>::block_number();
			let vote_end = now.saturating_add(T::VotingPeriod::get());

			let id = ProposalCount::<T>::mutate(|c| {
				*c = c.saturating_add(1);
				*c
			});

			let record = ProposalRecord::<T> {
				title,
				description,
				amount,
				organizer: organizer.clone(),
				event_block,
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

			let proposal =
				Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

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
		pub fn tally_proposal(
			origin: OriginFor<T>,
			proposal_id: ProposalId,
		) -> DispatchResult {
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

			if yes > no {
				proposal.status = ProposalStatus::Approved;
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
		pub fn execute_proposal(
			origin: OriginFor<T>,
			proposal_id: ProposalId,
		) -> DispatchResult {
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

			// Disburse first; if it fails the status stays Approved.
			T::Treasury::disburse(&proposal.organizer, proposal.amount)?;

			proposal.status = ProposalStatus::Executed;
			Proposals::<T>::insert(proposal_id, &proposal);

			Self::deposit_event(Event::ProposalExecuted {
				proposal_id,
				organizer: proposal.organizer,
				amount: proposal.amount,
			});

			Ok(())
		}
	}
}

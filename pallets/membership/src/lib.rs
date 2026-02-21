//! # Membership Pallet
//!
//! Maintains the authoritative set of active members for the GAIA network.
//! Exposes `is_active_member(AccountId) -> bool` for use by other pallets.
//!
//! ## Overview
//!
//! - A **member record** holds an account address, a name (max 128 bytes),
//!   a status (active or suspended), and a join timestamp (block number).
//! - Membership is granted through **peer approval**: active members propose
//!   candidates and vote to approve them.
//! - An **80 % majority** of active members is required for approval.
//! - Suspended members cannot propose or vote.
//!
//! ### Suspension (out of scope)
//!
//! Suspension mechanics are deferred to a future iteration. Two paths exist:
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
	use frame_system::pallet_prelude::*;

	/// Maximum length of a member name in bytes.
	pub const MAX_NAME_LEN: u32 = 128;

	/// Status of a member.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum MemberStatus {
		Active,
		Suspended,
	}

	/// On-chain record for a registered member.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct MemberRecord<T: Config> {
		pub name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
		pub status: MemberStatus,
		pub joined_at: BlockNumberFor<T>,
	}

	/// Pending candidate awaiting approval votes.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct CandidateRecord<T: Config> {
		pub name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
		pub proposed_by: T::AccountId,
		pub proposed_at: BlockNumberFor<T>,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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

	/// Map of pending candidates keyed by candidate account id.
	#[pallet::storage]
	pub type Candidates<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, CandidateRecord<T>, OptionQuery>;

	/// Votes cast for a candidate. `(candidate, voter) -> approved`.
	#[pallet::storage]
	pub type CandidateVotes<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		bool,
		OptionQuery,
	>;

	/// Number of approval votes received by each candidate.
	#[pallet::storage]
	pub type CandidateApprovalCount<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

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
		}
	}

	// ---------------------------------------------------------------------------
	// Events
	// ---------------------------------------------------------------------------

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A candidate has been proposed for membership.
		CandidateProposed {
			candidate: T::AccountId,
			proposed_by: T::AccountId,
		},
		/// An active member has voted on a candidate.
		VoteCast {
			candidate: T::AccountId,
			voter: T::AccountId,
			approve: bool,
		},
		/// A candidate has been approved and is now an active member.
		MemberApproved {
			member: T::AccountId,
		},
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
		/// A proposal for this candidate already exists.
		CandidateAlreadyProposed,
		/// No pending proposal exists for this candidate.
		CandidateNotFound,
		/// The voter has already cast a vote for this candidate.
		AlreadyVoted,
		/// The supplied name exceeds the maximum allowed length.
		NameTooLong,
		/// The caller is suspended and cannot perform this action.
		MemberSuspended,
	}

	// ---------------------------------------------------------------------------
	// Dispatchables
	// ---------------------------------------------------------------------------

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Propose a new candidate for membership.
		///
		/// The caller must be an active member. The candidate must not already
		/// be a member or have a pending proposal.
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(3, 2))]
		pub fn propose_member(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			name: BoundedVec<u8, ConstU32<MAX_NAME_LEN>>,
		) -> DispatchResult {
			let proposer = ensure_signed(origin)?;
			Self::ensure_active_member(&proposer)?;

			ensure!(!Members::<T>::contains_key(&candidate), Error::<T>::AlreadyMember);
			ensure!(
				!Candidates::<T>::contains_key(&candidate),
				Error::<T>::CandidateAlreadyProposed
			);

			let now = frame_system::Pallet::<T>::block_number();
			let record = CandidateRecord::<T> {
				name,
				proposed_by: proposer.clone(),
				proposed_at: now,
			};
			Candidates::<T>::insert(&candidate, record);

			Self::deposit_event(Event::CandidateProposed {
				candidate,
				proposed_by: proposer,
			});

			Ok(())
		}

		/// Vote to approve or reject a pending candidate.
		///
		/// The caller must be an active member and must not have already voted
		/// on this candidate. When the approval threshold (80 % of active
		/// members) is reached the candidate is automatically admitted.
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().reads_writes(4, 4))]
		pub fn vote_on_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			approve: bool,
		) -> DispatchResult {
			let voter = ensure_signed(origin)?;
			Self::ensure_active_member(&voter)?;

			ensure!(Candidates::<T>::contains_key(&candidate), Error::<T>::CandidateNotFound);
			ensure!(
				!CandidateVotes::<T>::contains_key(&candidate, &voter),
				Error::<T>::AlreadyVoted
			);

			CandidateVotes::<T>::insert(&candidate, &voter, approve);

			Self::deposit_event(Event::VoteCast {
				candidate: candidate.clone(),
				voter,
				approve,
			});

			if approve {
				let new_count =
					CandidateApprovalCount::<T>::mutate(&candidate, |c| {
						*c = c.saturating_add(1);
						*c
					});

				let active = ActiveMemberCount::<T>::get();
				// 80 % threshold: new_count * 5 >= active * 4
				if new_count.saturating_mul(5) >= active.saturating_mul(4) {
					Self::admit_candidate(&candidate)?;
				}
			}

			Ok(())
		}

		// TODO: Implement `suspend_member` dispatchable.
		//
		// Two suspension paths must be supported:
		//
		// 1. **Self-initiated** — a member voluntarily suspends their own
		//    account. This should be callable only by the member themselves.
		//
		// 2. **Unanimous peer vote** — all other active members vote to suspend
		//    a member. See ADR `docs/decisions/005-suspension-unanimity.md`
		//    for the rationale behind requiring unanimity rather than a simple
		//    majority.
		//
		// Until this is implemented, `MemberStatus::Suspended` is defined but
		// unused in production code paths.
	}

	// ---------------------------------------------------------------------------
	// Internal helpers
	// ---------------------------------------------------------------------------

	impl<T: Config> Pallet<T> {
		/// Returns `Ok(())` if `who` is an active member, otherwise an error.
		fn ensure_active_member(who: &T::AccountId) -> DispatchResult {
			let record = Members::<T>::get(who).ok_or(Error::<T>::NotActiveMember)?;
			ensure!(record.status == MemberStatus::Active, Error::<T>::MemberSuspended);
			Ok(())
		}

		/// Admit a candidate as an active member and clean up candidate storage.
		fn admit_candidate(candidate: &T::AccountId) -> DispatchResult {
			let cand = Candidates::<T>::take(candidate).ok_or(Error::<T>::CandidateNotFound)?;

			let now = frame_system::Pallet::<T>::block_number();
			let record = MemberRecord::<T> {
				name: cand.name,
				status: MemberStatus::Active,
				joined_at: now,
			};
			Members::<T>::insert(candidate, record);
			ActiveMemberCount::<T>::mutate(|c| *c = c.saturating_add(1));

			// Clean up votes for this candidate. The result is intentionally
			// ignored: the candidate key is already removed, so any stale vote
			// entries are unreachable and harmless.
			let _ = CandidateVotes::<T>::clear_prefix(candidate, u32::MAX, None);
			CandidateApprovalCount::<T>::remove(candidate);

			Self::deposit_event(Event::MemberApproved { member: candidate.clone() });

			Ok(())
		}
	}

	// ---------------------------------------------------------------------------
	// MembershipChecker implementation
	// ---------------------------------------------------------------------------

	impl<T: Config> MembershipChecker<T::AccountId> for Pallet<T> {
		fn is_active_member(account: &T::AccountId) -> bool {
			Members::<T>::get(account)
				.map(|r| r.status == MemberStatus::Active)
				.unwrap_or(false)
		}
	}
}

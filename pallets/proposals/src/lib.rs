#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_support::dispatch::DispatchResult;

/// Interface owned by the downstream `proposals` pallet for member eligibility checks.
pub trait MembershipChecker<AccountId> {
	fn is_active_member(account: &AccountId) -> bool;
}

/// Interface owned by the downstream `proposals` pallet for treasury disbursements.
pub trait TreasuryHandler<AccountId, Balance> {
	fn disburse(to: &AccountId, amount: Balance) -> DispatchResult;
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Balance: Parameter + Copy + Default;
		type Membership: super::MembershipChecker<Self::AccountId>;
		type Treasury: super::TreasuryHandler<Self::AccountId, Self::Balance>;
	}

	#[pallet::event]
	pub enum Event<T: Config> {
		_Reserved(PhantomData<T>),
	}

	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

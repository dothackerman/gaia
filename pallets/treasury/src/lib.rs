#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

use frame_support::dispatch::DispatchResult;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
    }

    #[pallet::storage]
    pub type TreasuryBalance<T: Config> = StorageValue<_, T::Balance, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        FeeDeposited {
            from: T::AccountId,
            amount: T::Balance,
            new_balance: T::Balance,
        },
        Disbursed {
            to: T::AccountId,
            amount: T::Balance,
            new_balance: T::Balance,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ZeroAmount,
        InsufficientFunds,
        BalanceOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn deposit_fee(origin: OriginFor<T>, amount: T::Balance) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_deposit(&who, amount)
        }

        #[pallet::call_index(1)]
        #[pallet::weight(T::DbWeight::get().reads_writes(1, 1))]
        pub fn disburse(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: T::Balance,
        ) -> DispatchResult {
            ensure_root(origin)?;
            Self::do_disburse(&to, amount)
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_deposit(from: &T::AccountId, amount: T::Balance) -> DispatchResult {
            ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);

            let new_balance = TreasuryBalance::<T>::get()
                .checked_add(&amount)
                .ok_or(Error::<T>::BalanceOverflow)?;
            TreasuryBalance::<T>::put(new_balance);
            Self::deposit_event(Event::FeeDeposited {
                from: from.clone(),
                amount,
                new_balance,
            });
            Ok(())
        }

        pub fn do_disburse(to: &T::AccountId, amount: T::Balance) -> DispatchResult {
            ensure!(!amount.is_zero(), Error::<T>::ZeroAmount);

            let new_balance = TreasuryBalance::<T>::get()
                .checked_sub(&amount)
                .ok_or(Error::<T>::InsufficientFunds)?;
            TreasuryBalance::<T>::put(new_balance);
            Self::deposit_event(Event::Disbursed {
                to: to.clone(),
                amount,
                new_balance,
            });
            Ok(())
        }
    }
}

impl<T: pallet::Config> gaia_proposals::TreasuryHandler<T::AccountId, T::Balance> for Pallet<T> {
    fn disburse(to: &T::AccountId, amount: T::Balance) -> DispatchResult {
        Self::do_disburse(to, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_noop, assert_ok, construct_runtime, parameter_types};
    use sp_core::H256;
    use sp_runtime::{traits::IdentityLookup, BuildStorage};

    type AccountId = u64;
    type Balance = u128;

    const ALICE: AccountId = 1;
    const BOB: AccountId = 2;

    construct_runtime!(
        pub enum Test {
            System: frame_system,
            Treasury: pallet,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
    }

    impl frame_system::Config for Test {
        type BaseCallFilter = frame_support::traits::Everything;
        type BlockWeights = ();
        type BlockLength = ();
        type DbWeight = ();
        type RuntimeOrigin = RuntimeOrigin;
        type RuntimeCall = RuntimeCall;
        type RuntimeTask = RuntimeTask;
        type Nonce = u64;
        type Hash = H256;
        type Hashing = sp_runtime::traits::BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Block = frame_system::mocking::MockBlock<Self>;
        type RuntimeEvent = RuntimeEvent;
        type BlockHashCount = BlockHashCount;
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = ();
        type OnSetCode = ();
        type MaxConsumers = frame_support::traits::ConstU32<16>;
        type ExtensionsWeightInfo = ();
        type SingleBlockMigrations = ();
        type MultiBlockMigrator = ();
        type PreInherents = ();
        type PostInherents = ();
        type PostTransactions = ();
    }

    impl pallet::Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type Balance = Balance;
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis builds");
        storage.into()
    }

    #[test]
    fn deposit_fee_increases_treasury_balance() {
        new_test_ext().execute_with(|| {
            assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 50));
            assert_eq!(TreasuryBalance::<Test>::get(), 50);
        });
    }

    #[test]
    fn deposit_fee_rejects_zero_amount() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 0),
                Error::<Test>::ZeroAmount
            );
        });
    }

    #[test]
    fn disburse_reduces_balance_when_funded() {
        new_test_ext().execute_with(|| {
            assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 100));
            assert_ok!(Treasury::disburse(RuntimeOrigin::root(), BOB, 40));
            assert_eq!(TreasuryBalance::<Test>::get(), 60);
        });
    }

    #[test]
    fn disburse_rejects_when_insufficient_funds() {
        new_test_ext().execute_with(|| {
            assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 20));
            assert_noop!(
                Treasury::disburse(RuntimeOrigin::root(), BOB, 50),
                Error::<Test>::InsufficientFunds
            );
            assert_eq!(TreasuryBalance::<Test>::get(), 20);
        });
    }

    #[test]
    fn proposals_treasury_handler_disburses_once_funded() {
        new_test_ext().execute_with(|| {
            assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 80));
            assert_ok!(<Treasury as gaia_proposals::TreasuryHandler<
                AccountId,
                Balance,
            >>::disburse(&BOB, 30));
            assert_eq!(TreasuryBalance::<Test>::get(), 50);
        });
    }

    #[test]
    fn disburse_requires_root_origin() {
        new_test_ext().execute_with(|| {
            assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(ALICE), 20));
            assert_noop!(
                Treasury::disburse(RuntimeOrigin::signed(ALICE), BOB, 10),
                sp_runtime::DispatchError::BadOrigin
            );
        });
    }
}

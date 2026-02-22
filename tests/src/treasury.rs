use crate::common::*;
use frame_support::{assert_noop, assert_ok};
use gaia_runtime::{Balances, Runtime, RuntimeOrigin, Treasury};

#[test]
fn treasury_funded_at_genesis() {
    new_test_ext().execute_with(|| {
        assert!(Balances::free_balance(gaia_treasury::Pallet::<Runtime>::account_id()) > 0);
    });
}

#[test]
fn deposit_fee_transfers_real_tokens() {
    new_test_ext().execute_with(|| {
        let treasury = gaia_treasury::Pallet::<Runtime>::account_id();
        let before_user = Balances::free_balance(alice());
        let before_treasury = Balances::free_balance(treasury.clone());
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 100));
        assert_eq!(Balances::free_balance(alice()), before_user - 100);
        assert_eq!(Balances::free_balance(treasury), before_treasury + 100);
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            100
        );
    });
}

#[test]
fn disburse_requires_root() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 100));
        assert_noop!(
            Treasury::disburse(RuntimeOrigin::signed(alice()), bob(), 10),
            sp_runtime::DispatchError::BadOrigin
        );
        assert_ok!(Treasury::disburse(RuntimeOrigin::root(), bob(), 10));
    });
}

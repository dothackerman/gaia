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

#[test]
fn deposit_zero_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 0),
            gaia_treasury::Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn disburse_zero_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 100));
        assert_noop!(
            Treasury::disburse(RuntimeOrigin::root(), bob(), 0),
            gaia_treasury::Error::<Runtime>::ZeroAmount
        );
    });
}

#[test]
fn disburse_insufficient_funds_fails() {
    new_test_ext().execute_with(|| {
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 50));
        assert_noop!(
            Treasury::disburse(RuntimeOrigin::root(), bob(), 100),
            gaia_treasury::Error::<Runtime>::InsufficientFunds
        );
        // Balance unchanged
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            50
        );
    });
}

#[test]
fn disburse_transfers_correct_amounts() {
    new_test_ext().execute_with(|| {
        let treasury = gaia_treasury::Pallet::<Runtime>::account_id();
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 200));
        let bob_before = Balances::free_balance(bob());
        let treasury_before = Balances::free_balance(treasury.clone());
        assert_ok!(Treasury::disburse(RuntimeOrigin::root(), bob(), 80));
        assert_eq!(Balances::free_balance(bob()), bob_before + 80);
        assert_eq!(Balances::free_balance(treasury), treasury_before - 80);
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            120
        );
    });
}

// ---------------------------------------------------------------------------
// Edge cases
// ---------------------------------------------------------------------------

/// `deposit_fee` only requires a signed origin — no membership check.
/// A non-member (Dave) can deposit tokens into the treasury.
#[test]
fn non_member_can_deposit_fee() {
    new_test_ext().execute_with(|| {
        let treasury = gaia_treasury::Pallet::<Runtime>::account_id();
        let treasury_before = Balances::free_balance(treasury.clone());
        let dave_before = Balances::free_balance(dave());
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(dave()), 42));
        assert_eq!(Balances::free_balance(dave()), dave_before - 42);
        assert_eq!(Balances::free_balance(treasury), treasury_before + 42);
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            42
        );
    });
}

/// Multiple deposits from different accounts accumulate correctly in
/// the treasury's sovereign account AND the explicit ledger.
#[test]
fn multiple_deposits_accumulate() {
    new_test_ext().execute_with(|| {
        let treasury = gaia_treasury::Pallet::<Runtime>::account_id();
        let treasury_before = Balances::free_balance(treasury.clone());

        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(alice()), 100));
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(bob()), 200));
        assert_ok!(Treasury::deposit_fee(RuntimeOrigin::signed(charlie()), 50));
        assert_eq!(
            gaia_treasury::pallet::TreasuryBalance::<Runtime>::get(),
            350
        );
        assert_eq!(Balances::free_balance(treasury), treasury_before + 350);
    });
}

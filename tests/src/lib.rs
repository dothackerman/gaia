mod common;
mod cross_pallet;
mod membership;
mod proposals;
mod treasury;

use crate::common::*;
use gaia_membership::pallet::ActiveMemberCount;
use gaia_membership::MembershipChecker;

#[test]
fn genesis_seeds_initial_members() {
    new_test_ext().execute_with(|| {
        assert!(
            <gaia_membership::Pallet<gaia_runtime::Runtime> as MembershipChecker<
                gaia_runtime::AccountId,
            >>::is_active_member(&alice())
        );
        assert!(
            <gaia_membership::Pallet<gaia_runtime::Runtime> as MembershipChecker<
                gaia_runtime::AccountId,
            >>::is_active_member(&bob())
        );
        assert!(
            <gaia_membership::Pallet<gaia_runtime::Runtime> as MembershipChecker<
                gaia_runtime::AccountId,
            >>::is_active_member(&charlie())
        );
        assert_eq!(ActiveMemberCount::<gaia_runtime::Runtime>::get(), 3);
    });
}

use super::{AccountId, Balance, Runtime, RuntimeEvent};
use frame_support::{parameter_types, PalletId};
use gaia_membership::MembershipChecker as MembershipSource;

pub struct MembershipAdapter;

impl gaia_proposals::MembershipChecker<AccountId> for MembershipAdapter {
    fn is_active_member(account: &AccountId) -> bool {
        <gaia_membership::Pallet<Runtime> as MembershipSource<AccountId>>::is_active_member(account)
    }
}

impl gaia_proposals::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Membership = MembershipAdapter;
    type Treasury = gaia_treasury::Pallet<Runtime>;
    type MembershipGovernance = gaia_membership::Pallet<Runtime>;
    type GovernancePalletId = GovernancePalletId;
}

parameter_types! {
    pub const GovernancePalletId: PalletId = PalletId(*b"ga/govn0");
}

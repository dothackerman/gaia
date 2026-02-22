use super::{AccountId, Balance, Runtime, RuntimeEvent};
use frame_support::dispatch::DispatchResult;
use gaia_membership::MembershipChecker as MembershipSource;

pub struct MembershipAdapter;
pub struct TreasuryAdapter;

impl gaia_proposals::MembershipChecker<AccountId> for MembershipAdapter {
	fn is_active_member(account: &AccountId) -> bool {
		<gaia_membership::Pallet<Runtime> as MembershipSource<AccountId>>::is_active_member(account)
	}
}

impl gaia_proposals::TreasuryHandler<AccountId, Balance> for TreasuryAdapter {
	fn disburse(_to: &AccountId, _amount: Balance) -> DispatchResult {
		// Fail closed until gaia-treasury exposes the real disbursement path.
		Err(sp_runtime::DispatchError::Other("Treasury handler not implemented"))
	}
}

impl gaia_proposals::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = Balance;
	type Membership = MembershipAdapter;
	type Treasury = TreasuryAdapter;
}

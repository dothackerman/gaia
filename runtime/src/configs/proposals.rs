use super::{AccountId, Balance, Runtime, RuntimeEvent};
use frame_support::traits::ConstU32;
use gaia_membership::MembershipChecker as MembershipSource;

pub struct MembershipAdapter;

impl gaia_proposals::MembershipChecker<AccountId> for MembershipAdapter {
    fn is_active_member(account: &AccountId) -> bool {
        <gaia_membership::Pallet<Runtime> as MembershipSource<AccountId>>::is_active_member(account)
    }
}

#[cfg(feature = "fast-local")]
type RuntimeVotingPeriod = ConstU32<20>;

#[cfg(not(feature = "fast-local"))]
type RuntimeVotingPeriod = ConstU32<100_800>;

impl gaia_proposals::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type Membership = MembershipAdapter;
    type Treasury = gaia_treasury::Pallet<Runtime>;
    // Normal mode: 7 days × 14 400 blocks/day (6s block time) = 100 800 blocks.
    // Fast local mode (feature `fast-local`): 20 blocks.
    type VotingPeriod = RuntimeVotingPeriod;
}

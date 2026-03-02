use super::{Runtime, RuntimeEvent};
use frame_support::traits::ConstU32;

#[cfg(feature = "fast-local")]
type RuntimeMembershipVotingPeriod = ConstU32<20>;

#[cfg(not(feature = "fast-local"))]
type RuntimeMembershipVotingPeriod = ConstU32<100_800>;

impl gaia_membership::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    // Normal mode: 7 days × 14 400 blocks/day (6s block time) = 100 800 blocks.
    // Fast local mode (feature `fast-local`): 20 blocks.
    type VotingPeriod = RuntimeMembershipVotingPeriod;
}

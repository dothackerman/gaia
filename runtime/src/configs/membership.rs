use super::{Runtime, RuntimeEvent};
use super::proposals::GovernancePalletId;

impl gaia_membership::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type GovernancePalletId = GovernancePalletId;
}

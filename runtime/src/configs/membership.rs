use super::{Runtime, RuntimeEvent};

impl gaia_membership::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

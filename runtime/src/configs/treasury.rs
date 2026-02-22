use super::{Runtime, RuntimeEvent};

impl gaia_treasury::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

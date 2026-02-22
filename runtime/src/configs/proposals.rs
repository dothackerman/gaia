use super::{Runtime, RuntimeEvent};

impl gaia_proposals::pallet::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
}

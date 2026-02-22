use super::{Balance, Runtime, RuntimeEvent};

impl gaia_treasury::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
}

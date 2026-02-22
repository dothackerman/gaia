use super::{Balance, Balances, Runtime, RuntimeEvent};
use frame_support::{parameter_types, PalletId};

parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"ga/trsy0");
}

impl gaia_treasury::pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type NativeBalance = Balances;
    type PalletId = TreasuryPalletId;
}

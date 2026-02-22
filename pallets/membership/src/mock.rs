use crate as gaia_membership;
use crate::pallet::MAX_NAME_LEN;
use frame_support::derive_impl;
use sp_runtime::{BoundedVec, BuildStorage};

type Block = frame_system::mocking::MockBlock<Test>;

#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask,
		RuntimeViewFunction
	)]
	pub struct Test;

	#[runtime::pallet_index(0)]
	pub type System = frame_system::Pallet<Test>;

	#[runtime::pallet_index(1)]
	pub type Membership = gaia_membership::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

impl gaia_membership::Config for Test {
	type RuntimeEvent = RuntimeEvent;
}

/// Genesis accounts used across all tests.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
/// Non-member account used in failure-path tests.
pub const DAVE: u64 = 4;
/// Candidate account.
pub const EVE: u64 = 5;

pub fn bounded_name(name: &[u8]) -> BoundedVec<u8, frame_support::traits::ConstU32<MAX_NAME_LEN>> {
	BoundedVec::try_from(name.to_vec()).expect("name within bounds")
}

/// Build genesis storage with three hardcoded active members: Alice, Bob, Charlie.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	let initial_members: BoundedVec<_, frame_support::traits::ConstU32<100>> = BoundedVec::try_from(vec![
		(ALICE, bounded_name(b"Alice")),
		(BOB, bounded_name(b"Bob")),
		(CHARLIE, bounded_name(b"Charlie")),
	])
	.expect("within bounds");

	gaia_membership::GenesisConfig::<Test> { initial_members }
		.assimilate_storage(&mut storage)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

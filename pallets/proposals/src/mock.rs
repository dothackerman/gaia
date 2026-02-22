use crate as gaia_proposals;
use frame_support::{derive_impl, dispatch::DispatchResult};
use sp_runtime::BuildStorage;
use std::cell::RefCell;
use std::collections::BTreeSet;

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
	pub type Proposals = gaia_proposals::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
}

// ---------------------------------------------------------------------------
// Mock membership — thread_local set of active account ids.
// ---------------------------------------------------------------------------

thread_local! {
	static ACTIVE_MEMBERS: RefCell<BTreeSet<u64>> = RefCell::new(BTreeSet::new());
}

pub struct MockMembership;

impl MockMembership {
	pub fn add(who: u64) {
		ACTIVE_MEMBERS.with(|m| m.borrow_mut().insert(who));
	}
	pub fn remove(who: u64) {
		ACTIVE_MEMBERS.with(|m| m.borrow_mut().remove(&who));
	}
}

impl gaia_proposals::MembershipChecker<u64> for MockMembership {
	fn is_active_member(account: &u64) -> bool {
		ACTIVE_MEMBERS.with(|m| m.borrow().contains(account))
	}
}

// ---------------------------------------------------------------------------
// Mock treasury — succeeds unless overridden.
// ---------------------------------------------------------------------------

thread_local! {
	static TREASURY_FAILS: RefCell<bool> = const { RefCell::new(false) };
}

pub struct MockTreasury;

impl MockTreasury {
	pub fn set_fail(fail: bool) {
		TREASURY_FAILS.with(|f| *f.borrow_mut() = fail);
	}
}

impl gaia_proposals::TreasuryHandler<u64, u64> for MockTreasury {
	fn disburse(_to: &u64, _amount: u64) -> DispatchResult {
		if TREASURY_FAILS.with(|f| *f.borrow()) {
			return Err(sp_runtime::DispatchError::Other("mock treasury failure"));
		}
		Ok(())
	}
}

// ---------------------------------------------------------------------------
// Pallet config
// ---------------------------------------------------------------------------

impl gaia_proposals::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Balance = u64;
	type Membership = MockMembership;
	type Treasury = MockTreasury;
	type VotingPeriod = frame_support::traits::ConstU64<10>;
}

// ---------------------------------------------------------------------------
// Test accounts
// ---------------------------------------------------------------------------

/// Active members in genesis.
pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
/// Non-member used in failure-path tests.
pub const DAVE: u64 = 4;

// ---------------------------------------------------------------------------
// Genesis helper
// ---------------------------------------------------------------------------

pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| {
		System::set_block_number(1);
		// Seed genesis members.
		MockMembership::add(ALICE);
		MockMembership::add(BOB);
		MockMembership::add(CHARLIE);
		// Reset treasury flag.
		MockTreasury::set_fail(false);
	});
	ext
}

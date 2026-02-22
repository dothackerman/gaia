use frame_support::traits::{ConstU32, OnInitialize};
use gaia_runtime::{
    AccountId, BalancesConfig, MembershipConfig, Runtime, RuntimeGenesisConfig, System,
};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{BoundedVec, BuildStorage};

pub fn alice() -> AccountId {
    Sr25519Keyring::Alice.to_account_id()
}
pub fn bob() -> AccountId {
    Sr25519Keyring::Bob.to_account_id()
}
pub fn charlie() -> AccountId {
    Sr25519Keyring::Charlie.to_account_id()
}
pub fn dave() -> AccountId {
    Sr25519Keyring::Dave.to_account_id()
}

fn bounded_name(
    name: &[u8],
) -> BoundedVec<u8, ConstU32<{ gaia_membership::pallet::MAX_NAME_LEN }>> {
    BoundedVec::try_from(name.to_vec()).expect("name fits")
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let initial_members: BoundedVec<_, ConstU32<100>> = BoundedVec::try_from(vec![
        (alice(), bounded_name(b"Alice")),
        (bob(), bounded_name(b"Bob")),
        (charlie(), bounded_name(b"Charlie")),
    ])
    .expect("bounded");

    let mut balances = vec![
        (alice(), 1u128 << 60),
        (bob(), 1u128 << 60),
        (charlie(), 1u128 << 60),
        (dave(), 1u128 << 60),
    ];
    balances.push((
        gaia_treasury::Pallet::<Runtime>::account_id(),
        1_000_000_000_000,
    ));

    let genesis = RuntimeGenesisConfig {
        balances: BalancesConfig {
            balances,
            dev_accounts: None,
        },
        membership: MembershipConfig { initial_members },
        ..Default::default()
    };

    let storage = genesis.build_storage().expect("genesis builds");
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn advance_blocks(n: u32) {
    for _ in 0..n {
        let next = System::block_number() + 1;
        System::set_block_number(next);
        System::on_initialize(next);
    }
}

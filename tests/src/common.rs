use frame_support::assert_ok;
use frame_support::traits::{ConstU32, Get, OnInitialize};
use gaia_runtime::{
    AccountId, BalancesConfig, Membership, MembershipConfig, Proposals, Runtime,
    RuntimeGenesisConfig, RuntimeOrigin, System,
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
pub fn eve() -> AccountId {
    Sr25519Keyring::Eve.to_account_id()
}
pub fn ferdie() -> AccountId {
    Sr25519Keyring::Ferdie.to_account_id()
}

/// Build genesis externalities with a custom set of initial members.
///
/// Each tuple is `(AccountId, name_bytes)`. Every listed account plus Dave,
/// Eve, and Ferdie receive a large initial balance. The treasury sovereign
/// account is also seeded.
pub fn new_test_ext_with_members(members: &[(AccountId, &[u8])]) -> sp_io::TestExternalities {
    let initial_members: BoundedVec<_, ConstU32<100>> = BoundedVec::try_from(
        members
            .iter()
            .map(|(id, name)| (id.clone(), bounded_name(name)))
            .collect::<Vec<_>>(),
    )
    .expect("bounded");

    // Collect unique accounts: all members + Dave/Eve/Ferdie
    let mut balance_accounts: std::collections::BTreeSet<AccountId> =
        members.iter().map(|(id, _)| id.clone()).collect();
    for extra in [dave(), eve(), ferdie()] {
        balance_accounts.insert(extra);
    }

    let mut balances: Vec<(AccountId, u128)> = balance_accounts
        .into_iter()
        .map(|id| (id, 1u128 << 60))
        .collect();
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

pub fn bounded_name(
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
        (eve(), 1u128 << 60),
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

/// Advance past the treasury proposal voting window so proposals can be tallied.
///
/// Derives the value from proposal-governance storage, so tests stay in sync
/// if governance updates this parameter.
pub fn advance_past_voting_period() {
    let period = gaia_proposals::ProposalVotingPeriod::<Runtime>::get();
    advance_blocks(period + 1);
}

/// Advance past the membership proposal voting window so proposals can be finalized.
pub fn advance_past_membership_voting_period() {
    let period = <<Runtime as gaia_membership::pallet::Config>::VotingPeriod as Get<u32>>::get();
    advance_blocks(period + 1);
}

/// Build a bounded proposal title from raw bytes.
pub fn bounded_title(
    s: &[u8],
) -> BoundedVec<u8, ConstU32<{ gaia_proposals::pallet::MAX_TITLE_LEN }>> {
    BoundedVec::try_from(s.to_vec()).expect("title fits")
}

/// Build a bounded proposal description from raw bytes.
pub fn bounded_desc(
    s: &[u8],
) -> BoundedVec<u8, ConstU32<{ gaia_proposals::pallet::MAX_DESC_LEN }>> {
    BoundedVec::try_from(s.to_vec()).expect("description fits")
}

/// Submit a minimal treasury spending proposal from Alice and return its id.
pub fn submit_default_proposal() -> u32 {
    assert_ok!(Proposals::submit_proposal(
        RuntimeOrigin::signed(alice()),
        bounded_title(b"t"),
        bounded_desc(b"d"),
        100,
        10
    ));
    gaia_proposals::pallet::ProposalCount::<Runtime>::get()
}

/// Submit a membership proposal and return its id.
pub fn submit_membership_proposal(signer: AccountId, candidate: AccountId, name: &[u8]) -> u32 {
    assert_ok!(Membership::propose_member(
        RuntimeOrigin::signed(signer),
        candidate,
        bounded_name(name)
    ));
    gaia_membership::pallet::MembershipProposalCount::<Runtime>::get()
}

/// Admit a candidate via membership proposal approval in default 3-member genesis.
pub fn admit_candidate(candidate: AccountId, name: &[u8]) -> u32 {
    let proposal_id = submit_membership_proposal(alice(), candidate, name);
    for voter in [alice(), bob(), charlie()] {
        assert_ok!(Membership::vote_on_candidate(
            RuntimeOrigin::signed(voter),
            proposal_id,
            true
        ));
    }
    proposal_id
}

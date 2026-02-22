// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{AccountId, BalancesConfig, MembershipConfig, RuntimeGenesisConfig, SudoConfig};
use alloc::{vec, vec::Vec};
use frame_support::build_struct_json_patch;
use frame_support::traits::ConstU32;
use serde_json::Value;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::Sr25519Keyring;
use sp_runtime::BoundedVec;

fn bounded_name(
    name: &[u8],
) -> BoundedVec<u8, ConstU32<{ gaia_membership::pallet::MAX_NAME_LEN }>> {
    BoundedVec::try_from(name.to_vec()).expect("hardcoded name must fit MAX_NAME_LEN")
}

fn initial_members() -> BoundedVec<
    (
        AccountId,
        BoundedVec<u8, ConstU32<{ gaia_membership::pallet::MAX_NAME_LEN }>>,
    ),
    ConstU32<100>,
> {
    BoundedVec::try_from(vec![
        (
            Sr25519Keyring::Alice.to_account_id(),
            bounded_name(b"Alice"),
        ),
        (Sr25519Keyring::Bob.to_account_id(), bounded_name(b"Bob")),
        (
            Sr25519Keyring::Charlie.to_account_id(),
            bounded_name(b"Charlie"),
        ),
    ])
    .expect("hardcoded initial_members must fit bounded capacity")
}

// Returns the genesis config presets populated with given parameters.
fn testnet_genesis(
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    endowed_accounts: Vec<AccountId>,
    root: AccountId,
) -> Value {
    let initial_members = initial_members();
    assert!(
        !initial_members.is_empty(),
        "runtime genesis requires at least one initial member"
    );

    let treasury_account: AccountId =
        gaia_treasury::Pallet::<crate::Runtime>::account_id();
    let mut balances = endowed_accounts
        .iter()
        .cloned()
        .map(|k| (k, 1u128 << 60))
        .collect::<Vec<_>>();
    balances.push((treasury_account, 1_000_000_000_000));

    build_struct_json_patch!(RuntimeGenesisConfig {
        balances: BalancesConfig { balances },
        aura: pallet_aura::GenesisConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| x.0.clone())
                .collect::<Vec<_>>(),
        },
        grandpa: pallet_grandpa::GenesisConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect::<Vec<_>>(),
        },
        sudo: SudoConfig { key: Some(root) },
        membership: MembershipConfig { initial_members },
    })
}

/// Return the development genesis config.
pub fn development_config_genesis() -> Value {
    testnet_genesis(
        vec![(
            sp_keyring::Sr25519Keyring::Alice.public().into(),
            sp_keyring::Ed25519Keyring::Alice.public().into(),
        )],
        vec![
            Sr25519Keyring::Alice.to_account_id(),
            Sr25519Keyring::Bob.to_account_id(),
            Sr25519Keyring::AliceStash.to_account_id(),
            Sr25519Keyring::BobStash.to_account_id(),
        ],
        sp_keyring::Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Return the local genesis config preset.
pub fn local_config_genesis() -> Value {
    testnet_genesis(
        vec![
            (
                sp_keyring::Sr25519Keyring::Alice.public().into(),
                sp_keyring::Ed25519Keyring::Alice.public().into(),
            ),
            (
                sp_keyring::Sr25519Keyring::Bob.public().into(),
                sp_keyring::Ed25519Keyring::Bob.public().into(),
            ),
        ],
        Sr25519Keyring::iter()
            .filter(|v| v != &Sr25519Keyring::One && v != &Sr25519Keyring::Two)
            .map(|v| v.to_account_id())
            .collect::<Vec<_>>(),
        Sr25519Keyring::Alice.to_account_id(),
    )
}

/// Provides the JSON representation of predefined genesis config for given `id`.
pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
    let patch = match id.as_ref() {
        sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
        sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET => local_config_genesis(),
        _ => return None,
    };
    Some(
        serde_json::to_string(&patch)
            .expect("serialization to json is expected to work. qed.")
            .into_bytes(),
    )
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
    vec![
        PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
        PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected treasury seed balance in all presets.
    const TREASURY_SEED_BALANCE: u128 = 1_000_000_000_000;

    /// Expected endowed-account balance in all presets.
    const ENDOWED_BALANCE: u128 = 1u128 << 60;

    fn member_count(preset: &Value) -> usize {
        preset
            .get("membership")
            .and_then(|m| m.get("initialMembers"))
            .and_then(Value::as_array)
            .map(|members| members.len())
            .expect("membership.initialMembers must exist in runtime preset")
    }

    /// Extract the `balances.balances` array from a preset JSON value.
    fn balances_entries(preset: &Value) -> Vec<(String, u128)> {
        preset
            .get("balances")
            .and_then(|b| b.get("balances"))
            .and_then(Value::as_array)
            .expect("balances.balances must exist in runtime preset")
            .iter()
            .map(|entry| {
                let arr = entry.as_array().expect("each balance entry is a tuple");
                let account = arr[0].as_str().expect("account is a string").to_string();
                let amount = arr[1]
                    .as_number()
                    .and_then(|n| n.as_u128())
                    .expect("balance is a u128");
                (account, amount)
            })
            .collect()
    }

    /// Return the SS58 address of the treasury sovereign account.
    fn treasury_account_ss58() -> String {
        use sp_core::crypto::Ss58Codec;
        let treasury_account: AccountId =
            gaia_treasury::Pallet::<crate::Runtime>::account_id();
        treasury_account.to_ss58check()
    }

    // ---------------------------------------------------------------------------
    // Membership preset tests
    // ---------------------------------------------------------------------------

    #[test]
    fn development_preset_has_non_empty_initial_members() {
        assert!(member_count(&development_config_genesis()) > 0);
    }

    #[test]
    fn local_preset_has_non_empty_initial_members() {
        assert!(member_count(&local_config_genesis()) > 0);
    }

    // ---------------------------------------------------------------------------
    // Treasury balance preset tests
    // ---------------------------------------------------------------------------

    #[test]
    fn development_preset_includes_treasury_account_with_seed_balance() {
        let preset = development_config_genesis();
        let entries = balances_entries(&preset);
        let treasury_ss58 = treasury_account_ss58();

        let treasury_entry = entries
            .iter()
            .find(|(account, _)| *account == treasury_ss58);

        assert!(
            treasury_entry.is_some(),
            "development preset must include treasury account ({treasury_ss58}) in balances"
        );
        assert_eq!(
            treasury_entry.unwrap().1,
            TREASURY_SEED_BALANCE,
            "treasury seed balance must be {TREASURY_SEED_BALANCE}"
        );
    }

    #[test]
    fn local_preset_includes_treasury_account_with_seed_balance() {
        let preset = local_config_genesis();
        let entries = balances_entries(&preset);
        let treasury_ss58 = treasury_account_ss58();

        let treasury_entry = entries
            .iter()
            .find(|(account, _)| *account == treasury_ss58);

        assert!(
            treasury_entry.is_some(),
            "local preset must include treasury account ({treasury_ss58}) in balances"
        );
        assert_eq!(
            treasury_entry.unwrap().1,
            TREASURY_SEED_BALANCE,
            "treasury seed balance must be {TREASURY_SEED_BALANCE}"
        );
    }

    #[test]
    fn development_preset_endowed_accounts_have_correct_balance() {
        let preset = development_config_genesis();
        let entries = balances_entries(&preset);
        let treasury_ss58 = treasury_account_ss58();

        let endowed: Vec<_> = entries
            .iter()
            .filter(|(account, _)| *account != treasury_ss58)
            .collect();

        assert!(
            !endowed.is_empty(),
            "development preset must have at least one endowed account"
        );
        for (account, balance) in &endowed {
            assert_eq!(
                *balance, ENDOWED_BALANCE,
                "endowed account {account} must have balance {ENDOWED_BALANCE}"
            );
        }
    }

    #[test]
    fn local_preset_endowed_accounts_have_correct_balance() {
        let preset = local_config_genesis();
        let entries = balances_entries(&preset);
        let treasury_ss58 = treasury_account_ss58();

        let endowed: Vec<_> = entries
            .iter()
            .filter(|(account, _)| *account != treasury_ss58)
            .collect();

        assert!(
            !endowed.is_empty(),
            "local preset must have at least one endowed account"
        );
        for (account, balance) in &endowed {
            assert_eq!(
                *balance, ENDOWED_BALANCE,
                "endowed account {account} must have balance {ENDOWED_BALANCE}"
            );
        }
    }

    #[test]
    fn treasury_account_is_not_duplicated_in_balances() {
        let treasury_ss58 = treasury_account_ss58();

        for (name, preset) in [
            ("development", development_config_genesis()),
            ("local", local_config_genesis()),
        ] {
            let entries = balances_entries(&preset);
            let count = entries
                .iter()
                .filter(|(account, _)| *account == treasury_ss58)
                .count();
            assert_eq!(
                count, 1,
                "{name} preset must contain exactly one treasury balance entry, found {count}"
            );
        }
    }
}

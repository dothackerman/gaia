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
use frame_support::PalletId;
use serde_json::Value;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_genesis_builder::{self, PresetId};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::AccountIdConversion;
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

    let treasury_account: AccountId = PalletId(*b"ga/trsy0").into_account_truncating();
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

    fn member_count(preset: Value) -> usize {
        preset
            .get("membership")
            .and_then(|m| m.get("initialMembers"))
            .and_then(Value::as_array)
            .map(|members| members.len())
            .expect("membership.initialMembers must exist in runtime preset")
    }

    #[test]
    fn development_preset_has_non_empty_initial_members() {
        assert!(member_count(development_config_genesis()) > 0);
    }

    #[test]
    fn local_preset_has_non_empty_initial_members() {
        assert!(member_count(local_config_genesis()) > 0);
    }
}

// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.

// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>.

use {
    vane_tanssi_runtime::{
        AccountId, MaintenanceModeConfig, MigrationsConfig, PolkadotXcmConfig, Signature,
    },
    cumulus_primitives_core::ParaId,
    sc_chain_spec::{ChainSpecExtension, ChainSpecGroup},
    sc_network::config::MultiaddrWithPeerId,
    sc_service::ChainType,
    serde::{Deserialize, Serialize},
    
    sp_runtime::traits::{IdentifyAccount, Verify},
};

use sp_core::{sr25519, sr25519::Pair as PairType, Pair, Public};
use sp_runtime::MultiSigner;
use sp_core::{MaxEncodedLen,RuntimeDebug};

use parity_scale_codec::{Encode,Decode};
use sp_core::{crypto::{Ss58AddressFormatRegistry, Ss58Codec}};
use vane_tanssi_runtime::CurrencyId::DOT;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<
    vane_tanssi_runtime::RuntimeGenesisConfig,
    Extensions,
>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Orcherstrator's parachain id
pub const ORCHESTRATOR: ParaId = ParaId::new(1000);

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(para_id: ParaId, boot_nodes: Vec<String>) -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());
    properties.insert("isEthereum".into(), false.into());

    let mut default_funded_accounts = pre_funded_accounts();
    default_funded_accounts.sort();
    default_funded_accounts.dedup();
    let boot_nodes: Vec<MultiaddrWithPeerId> = boot_nodes
        .into_iter()
        .map(|x| {
            x.parse::<MultiaddrWithPeerId>()
                .unwrap_or_else(|e| panic!("invalid bootnode address format {:?}: {:?}", x, e))
        })
        .collect();

    ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                default_funded_accounts.clone(),
                para_id,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
            )
        },
        boot_nodes,
        None,
        None,
        None,
        Some(properties),
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: para_id.into(),
        },
    )
}

pub fn local_testnet_config(para_id: ParaId, boot_nodes: Vec<String>) -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "UNIT".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());
    properties.insert("isEthereum".into(), false.into());
    let protocol_id = Some(format!("container-chain-{}", para_id));

    let mut default_funded_accounts = pre_funded_accounts();
    default_funded_accounts.sort();
    default_funded_accounts.dedup();
    let boot_nodes: Vec<MultiaddrWithPeerId> = boot_nodes
        .into_iter()
        .map(|x| {
            x.parse::<MultiaddrWithPeerId>()
                .unwrap_or_else(|e| panic!("invalid bootnode address format {:?}: {:?}", x, e))
        })
        .collect();

    ChainSpec::from_genesis(
        // Name
        &format!("Simple Container {}", para_id),
        // ID
        &format!("simple_container_{}", para_id),
        ChainType::Local,
        move || {
            testnet_genesis(
                default_funded_accounts.clone(),
                para_id,
                get_account_id_from_seed::<sr25519::Public>("Alice"),
            )
        },
        // Bootnodes
        boot_nodes,
        // Telemetry
        None,
        // Protocol ID
        protocol_id.as_deref(),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: para_id.into(),
        },
    )
}

fn testnet_genesis(
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
    root_key: AccountId,
) -> vane_tanssi_runtime::RuntimeGenesisConfig {


    let alice = get_account_id_from_seed::<sr25519::Public>("Alice");
	let bob = get_account_id_from_seed::<sr25519::Public>("Bob");

	let v_dot = "vDOT".as_bytes().to_vec();
	let _v_usdt = "vUSDT".as_bytes().to_vec();
	let _v_usdc = "vUSDC".as_bytes().to_vec();

	// Calculate parachain Soverign account id
	let sovererign_acount = calculate_sovereign_account::<PairType>(id.into()).unwrap();
	let para_account = sp_runtime::AccountId32::from_ss58check(&sovererign_acount).unwrap();


    vane_tanssi_runtime::RuntimeGenesisConfig {
        system: vane_tanssi_runtime::SystemConfig {
            code: vane_tanssi_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
            ..Default::default()
        },
        balances: vane_tanssi_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1 << 60))
                .collect(),
        },
        parachain_info: vane_tanssi_runtime::ParachainInfoConfig {
            parachain_id: id,
            ..Default::default()
        },
        parachain_system: Default::default(),
        sudo: vane_tanssi_runtime::SudoConfig {
            key: Some(root_key),
        },
        authorities_noting: vane_tanssi_runtime::AuthoritiesNotingConfig {
            orchestrator_para_id: ORCHESTRATOR,
            ..Default::default()
        },
        migrations: MigrationsConfig::default(),
        maintenance_mode: MaintenanceModeConfig {
            start_in_maintenance_mode: false,
            ..Default::default()
        },
        // This should initialize it to whatever we have set in the pallet
        polkadot_xcm: PolkadotXcmConfig::default(),
        transaction_payment: Default::default(),

        vane_assets: vane_tanssi_runtime::VaneAssetsConfig {

            metadata: vec![(DOT,v_dot.clone(), v_dot,10)],

            assets: vec![(DOT,para_account.clone(),true,1)],

            accounts: vec![(DOT,para_account.clone(),0)]

        },
        vane_xcm_transfer_system: vane_tanssi_runtime::VaneXcmTransferSystemConfig {
            para_account: Some(para_account)
        },
        tx_pause: Default::default(),
    }
}

/// Get pre-funded accounts
pub fn pre_funded_accounts() -> Vec<AccountId> {
    vec![
        get_account_id_from_seed::<sr25519::Public>("Alice"),
        get_account_id_from_seed::<sr25519::Public>("Bob"),
        get_account_id_from_seed::<sr25519::Public>("Charlie"),
        get_account_id_from_seed::<sr25519::Public>("Dave"),
        get_account_id_from_seed::<sr25519::Public>("Eve"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie"),
        get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
        get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
        get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
        get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
        get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
        get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
    ]
}


// helper functions
#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub struct RococoId(u32);

fn calculate_sovereign_account<Pair>(
	para_id: u32
) -> Result<String, Box<dyn std::error::Error>>
	where
		Pair: sp_core::Pair,
		Pair::Public: Into<MultiSigner>,
{
	// Scale encoded para_id
	let id = RococoId(para_id);
	let encoded_id = hex::encode(id.encode());

	// Prefix para or sibl
	let prefix = hex::encode("para");

	// Join both strings and the 0x at the beginning
	let encoded_key = "0x".to_owned() + &prefix + &encoded_id;

	// Fill the rest with 0s
	let public_str = format!("{:0<width$}", encoded_key, width = (64 + 2) as usize);

	// Convert hex public key to ss58 address
	let public = array_bytes::hex2bytes(&public_str).expect("Failed to convert hex to bytes");
	let public_key = Pair::Public::try_from(&public)
		.map_err(|_| "Failed to construct public key from given hex")?;

	Ok(public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into()))
}

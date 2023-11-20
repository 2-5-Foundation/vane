use cumulus_primitives_core::ParaId;
use vane_tanssi_runtime::{AccountId, Signature, EXISTENTIAL_DEPOSIT,PolkadotXcmConfig};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, sr25519::Pair as PairType, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use staging_xcm::prelude::*;
use sc_network::config::MultiaddrWithPeerId;

use codec::{Encode,Decode};
use sp_core::{crypto::{Ss58AddressFormatRegistry, Ss58Codec}};
use sp_runtime::{MultiSigner};
use vane_tanssi_runtime::CurrencyId::DOT;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
	sc_service::GenericChainSpec<vane_tanssi_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = staging_xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

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
const ORCHESTRATOR: ParaId = ParaId::new(1000);

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
// pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
// 	get_from_seed::<AuraId>(seed)
// }

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
// pub fn template_session_keys(keys: AuraId) -> vane_para_runtime::SessionKeys {
// 	vane_para_runtime::SessionKeys { aura: keys }
// }


pub fn pre_funded_accounts() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
	]
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


// pub fn development_config() -> ChainSpec {
// 	// Give your base currency a unit name and decimal places
// 	let mut properties = sc_chain_spec::Properties::new();
// 	properties.insert("tokenSymbol".into(), "Vane".into());
// 	properties.insert("tokenDecimals".into(), 5.into());
// 	properties.insert("ss58Format".into(), 42.into());
//
// 	ChainSpec::from_genesis(
// 		// Name
// 		"Vane Development",
// 		// ID
// 		"vane-dev",
// 		ChainType::Development,
// 		move || {
// 			testnet_genesis(
// 				// initial collators.
// 				vec![
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Alice"),
// 						get_collator_keys_from_seed("Alice"),
// 					),
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Bob"),
// 						get_collator_keys_from_seed("Bob"),
// 					),
// 				],
// 				vec![
// 					get_account_id_from_seed::<sr25519::Public>("Alice"),
// 					get_account_id_from_seed::<sr25519::Public>("Bob"),
// 					get_account_id_from_seed::<sr25519::Public>("Charlie"),
// 					get_account_id_from_seed::<sr25519::Public>("Dave"),
//
//
// 				],
// 				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
// 				1000.into(),
// 			)
// 		},
// 		Vec::new(),
// 		None,
// 		None,
// 		None,
// 		None,
// 		Extensions {
// 			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
// 			para_id: 1000,
// 		},
// 	)
// }


pub fn local_testnet_config(para_id: ParaId, boot_nodes: Vec<String>) -> ChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 5.into());
	properties.insert("ss58Format".into(), 42.into());
	properties.insert("isEthereum".into(), false.into());
	let protocol_id = Some(format!("vane-network-{}", para_id));

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
		&format!("vane-network_{}", para_id),
		// ID
		&format!("vane-network_{}", para_id),
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

// pub fn local_testnet_config() -> ChainSpec {
// 	// Give your base currency a unit name and decimal places
// 	let mut properties = sc_chain_spec::Properties::new();
// 	properties.insert("tokenSymbol".into(), "USDT".into());
// 	properties.insert("tokenDecimals".into(), 12.into());
// 	properties.insert("ss58Format".into(), 42.into());
//
// 	ChainSpec::from_genesis(
// 		// Name
// 		"Vane Testnet",
// 		// ID
// 		"vane_testnet",
// 		ChainType::Local,
// 		move || {
// 			testnet_genesis(
// 				// initial collators.
// 				vec![
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Alice"),
// 						get_collator_keys_from_seed("Alice"),
// 					),
// 					(
// 						get_account_id_from_seed::<sr25519::Public>("Bob"),
// 						get_collator_keys_from_seed("Bob"),
// 					),
// 				],
// 				vec![
// 					get_account_id_from_seed::<sr25519::Public>("Alice"),
// 					get_account_id_from_seed::<sr25519::Public>("Bob"),
// 					get_account_id_from_seed::<sr25519::Public>("Charlie"),
// 					get_account_id_from_seed::<sr25519::Public>("Dave"),
//
//
// 				],
// 				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
// 				2000.into(),
// 			)
// 		},
// 		// Bootnodes
// 		Vec::new(),
// 		// Telemetry
// 		None,
// 		// Protocol ID
// 		Some("vane-local"),
// 		// Fork ID
// 		None,
// 		// Properties
// 		Some(properties),
// 		// Extensions
// 		Extensions {
// 			relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
// 			para_id: 2000,
// 		},
// 	)
// }

#[derive(Encode,Decode)]
pub struct RococoId(u32);

fn calculate_sovereign_account<Pair>(
	para_id: u32,
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



fn testnet_genesis(
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
	root_key: AccountId,
) -> vane_tanssi_runtime::RuntimeGenesisConfig {

	let alice = get_from_seed::<sr25519::Public>("Alice");
	let bob = get_from_seed::<sr25519::Public>("Bob");

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

		vane_assets: vane_tanssi_runtime::VaneAssetsConfig {

			metadata: vec![(DOT,v_dot.clone(), v_dot,10)],

			assets: vec![(DOT,para_account.clone(),true,1)],

			accounts: vec![(DOT,para_account.clone(),0)]

		},
		// This should initialize it to whatever we have set in the pallet
		polkadot_xcm: PolkadotXcmConfig::default(),
		transaction_payment: Default::default(),
	}
}



// fn testnet_genesis(
// 	invulnerables: Vec<(AccountId, AuraId)>,
// 	endowed_accounts: Vec<AccountId>,
// 	root_key: Option<AccountId>,
// 	id: ParaId,
// ) -> vane_para_runtime::GenesisConfig {
// 	let alice = get_from_seed::<sr25519::Public>("Alice");
// 	let bob = get_from_seed::<sr25519::Public>("Bob");
//
// 	let v_dot = "vDOT".as_bytes().to_vec();
// 	let _v_usdt = "vUSDT".as_bytes().to_vec();
// 	let _v_usdc = "vUSDC".as_bytes().to_vec();
//
// 	// Calculate parachain Soverign account id
// 	let sovererign_acount = calculate_sovereign_account::<PairType>(id.into()).unwrap();
// 	let para_account = sp_runtime::AccountId32::from_ss58check(&sovererign_acount).unwrap();
//
//
// 	vane_para_runtime::RuntimeGenesisConfig {
// 		system: vane_para_runtime::SystemConfig {
// 			code: vane_para_runtime::WASM_BINARY
// 				.expect("WASM binary was not build, please build it!")
// 				.to_vec(),
// 			..Default::default()
// 		},
// 		balances: vane_para_runtime::BalancesConfig {
// 			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
// 		},
//
// 		vane_assets: vane_para_runtime::VaneAssetsConfig {
//
// 			metadata: vec![(DOT,v_dot.clone(), v_dot,10)],
//
// 			assets: vec![(DOT,para_account.clone(),true,1)],
//
// 			accounts: vec![(DOT,para_account.clone(),0)]
//
// 		},
//
// 		vane_xcm: vane_para_runtime::VaneXcmConfig {
// 			para_account: Some(para_account)
// 		},
//
// 		parachain_info: vane_para_runtime::ParachainInfoConfig {
// 			parachain_id: id,
// 			..Default::default()
// 		},
//
// 		collator_selection: vane_para_runtime::CollatorSelectionConfig {
// 			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
// 			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
// 			..Default::default()
// 		},
//
// 		session: vane_para_runtime::SessionConfig {
// 			keys: invulnerables
// 				.into_iter()
// 				.map(|(acc, aura)| {
// 					(
// 						acc.clone(),                 // account id
// 						acc,                         // validator id
// 						template_session_keys(aura), // session keys
// 					)
// 				})
// 				.collect(),
// 		},
// 		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
// 		// of this.
// 		aura: Default::default(),
// 		aura_ext: Default::default(),
// 		sudo: vane_para_runtime::SudoConfig { key: root_key },
// 		council: vane_para_runtime::CouncilConfig {
// 			phantom: std::marker::PhantomData,
// 			members: endowed_accounts.iter().take(4).map(|acc| acc.clone()).collect::<Vec<_>>(),
// 		},
// 		parachain_system: Default::default(),
// 		polkadot_xcm: vane_para_runtime::PolkadotXcmConfig {
// 			safe_xcm_version: Some(SAFE_XCM_VERSION),
// 			..Default::default()
// 		},
// 		transaction_payment: Default::default(),
// 		// vane_assets: Default::default(),
//
// 	}
// }
//

// Vane Live Network
// fn live_genesis(
// 	invulnerables: Vec<(AccountId, AuraId)>,
// 	endowed_accounts: Vec<AccountId>,
// 	root_key: Option<AccountId>,
// 	id: ParaId,
// ) -> vane_runtime::GenesisConfig {
// 	let alice = get_from_seed::<sr25519::Public>("Alice");
// 	let bob = get_from_seed::<sr25519::Public>("Bob");
// 	vane_runtime::GenesisConfig {
// 		system: vane_runtime::SystemConfig {
// 			code: vane_runtime::WASM_BINARY
// 				.expect("WASM binary was not build, please build it!")
// 				.to_vec(),
// 			..Default::default()
// 		},
// 		balances: vane_runtime::BalancesConfig {
// 			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
// 		},
//
//
// 		parachain_info: vane_runtime::ParachainInfoConfig {
// 			parachain_id: id,
// 			..Default::default()
// 		},
// 		collator_selection: vane_runtime::CollatorSelectionConfig {
// 			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
// 			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
// 			..Default::default()
// 		},
// 		session: vane_runtime::SessionConfig {
// 			keys: invulnerables
// 				.into_iter()
// 				.map(|(acc, aura)| {
// 					(
// 						acc.clone(),                 // account id
// 						acc,                         // validator id
// 						template_session_keys(aura), // session keys
// 					)
// 				})
// 				.collect(),
// 		},
// 		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
// 		// of this.
// 		aura: Default::default(),
// 		aura_ext: Default::default(),
// 		sudo: vane_runtime::SudoConfig { key: root_key },
// 		council: vane_runtime::CouncilConfig {
// 			phantom: std::marker::PhantomData,
// 			members: endowed_accounts.iter().take(4).map(|acc| acc.clone()).collect::<Vec<_>>(),
// 		},
// 		parachain_system: Default::default(),
// 		polkadot_xcm: vane_runtime::PolkadotXcmConfig {
// 			safe_xcm_version: Some(SAFE_XCM_VERSION),
// 			..Default::default()
// 		},
// 		transaction_payment: Default::default(),
// 		// vane_assets: Default::default(),
//
// 	}
// }

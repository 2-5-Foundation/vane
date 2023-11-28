use cumulus_primitives_core::ParaId;

use vane_tanssi_runtime::{AccountId, Signature};
use vane_para_runtime::{AccountId as VaneParaRuntimeAcountId, Signature as VaneParaRuntimeSignature, EXISTENTIAL_DEPOSIT, AuraId};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, sr25519::Pair as PairType, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use staging_xcm::prelude::*;
use sc_network::config::MultiaddrWithPeerId;
use sp_core::{MaxEncodedLen,RuntimeDebug};

use codec::{Encode,Decode};
use sp_core::{crypto::{Ss58AddressFormatRegistry, Ss58Codec}};
use sp_runtime::{MultiSigner};
use vane_tanssi_runtime::CurrencyId::DOT;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type TanssiChainSpec = sc_service::GenericChainSpec<vane_tanssi_runtime::GenesisConfig, Extensions>;

pub type ParachainChainSpec = sc_service::GenericChainSpec<vane_para_runtime::GenesisConfig, Extensions>;

/// The default XCM version to set in genesis config.
const SAFE_XCM_VERSION: u32 = staging_xcm::prelude::XCM_VERSION;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

// The extensions for the [`ChainSpec`].
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
const ORCHESTRATOR: ParaId = ParaId::new(2000);

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

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
pub fn template_session_keys(keys: AuraId) -> vane_para_runtime::SessionKeys {
	vane_para_runtime::SessionKeys { aura: keys }
}


pub fn pre_funded_accounts() -> Vec<AccountId> {
	vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
	]
}



pub fn tanssi_config(para_id: ParaId, boot_nodes: Vec<String>) -> TanssiChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 5.into());
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

	TanssiChainSpec::from_genesis(
		// Name
		"Vane-Tanssi-Container-Chain",
		// ID
		"Vane-Tanssi",
		ChainType::Development,
		move || {
			genesis_config(
				ConfigChain::Tanssi,
				default_funded_accounts.clone(),
				para_id,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![] // No invulnerables as tanssi-runtime do not use pallet collator
			).0.unwrap()
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


pub fn parachain_config(para_id: ParaId, boot_nodes: Vec<String>) -> ParachainChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "UNIT".into());
	properties.insert("tokenDecimals".into(), 5.into());
	properties.insert("ss58Format".into(), 42.into());
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

	ParachainChainSpec::from_genesis(
		// Name
		&format!("vane-network"),
		// ID
		&format!("vane-network"),
		ChainType::Local,
		move || {
			genesis_config(
				ConfigChain::Parachain,
				default_funded_accounts.clone(),
				para_id,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed("Bob"),
					),
				],
			).1.unwrap()
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



#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen)]
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


#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen)]
pub enum ConfigChain {
	Tanssi,
	Parachain
}



fn genesis_config(
	chain: ConfigChain,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
	root_key: AccountId,
	invulnerables: Vec<(AccountId, AuraId)>

) -> (Option<vane_tanssi_runtime::RuntimeGenesisConfig>, Option<vane_para_runtime::RuntimeGenesisConfig>) {

	let alice = get_from_seed::<sr25519::Public>("Alice");
	let bob = get_from_seed::<sr25519::Public>("Bob");

	let v_dot = "vDOT".as_bytes().to_vec();
	let _v_usdt = "vUSDT".as_bytes().to_vec();
	let _v_usdc = "vUSDC".as_bytes().to_vec();

	// Calculate parachain Soverign account id
	let sovererign_acount = calculate_sovereign_account::<PairType>(id.into()).unwrap();
	let para_account = sp_runtime::AccountId32::from_ss58check(&sovererign_acount).unwrap();


	 let chain_spec =match chain {

		ConfigChain::Tanssi => {

			let vane_tanssi_runtime_genesis_config = vane_tanssi_runtime::RuntimeGenesisConfig {
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
						.map(|k| (k, 10000000000))
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
				polkadot_xcm: vane_tanssi_runtime::PolkadotXcmConfig::default(),
				transaction_payment: Default::default(),

				vane_xcm_transfer_system: vane_tanssi_runtime::VaneXcmTransferSystemConfig {
					para_account: Some(para_account)
				}
			};

			(Some(vane_tanssi_runtime_genesis_config),None)

		},

		ConfigChain::Parachain => {

			let vane_para_runtime_genesis_config = vane_para_runtime::RuntimeGenesisConfig {
				system: vane_para_runtime::SystemConfig {
					code: vane_para_runtime::WASM_BINARY
						.expect("WASM binary was not build, please build it!")
						.to_vec(),
					..Default::default()
				},
				balances: vane_para_runtime::BalancesConfig {
					balances: endowed_accounts.iter().cloned().map(|k| (k, 10000000000)).collect(),
				},

				vane_assets: vane_para_runtime::VaneAssetsConfig {

					metadata: vec![(DOT,v_dot.clone(), v_dot,10)],

					assets: vec![(DOT,para_account.clone(),true,1)],

					accounts: vec![(DOT,para_account.clone(),0)]

				},

				vane_xcm_transfer_system: vane_para_runtime::VaneXcmTransferSystemConfig {
					para_account: Some(para_account)
				},

				parachain_info: vane_para_runtime::ParachainInfoConfig {
					parachain_id: id,
					..Default::default()
				},

				collator_selection: vane_para_runtime::CollatorSelectionConfig {
					invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
					candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
					..Default::default()
				},

				session: vane_para_runtime::SessionConfig {
					keys: invulnerables
						.into_iter()
						.map(|(acc, aura)| {
							(
								acc.clone(),                 // account id
								acc,                         // validator id
								template_session_keys(aura), // session keys
							)
						})
						.collect(),
				},
				// no need to pass anything to aura, in fact it will panic if we do. Session will take care
				// of this.
				aura: Default::default(),
				aura_ext: Default::default(),
				sudo: vane_para_runtime::SudoConfig { key: Some(root_key) },

				parachain_system: Default::default(),

				polkadot_xcm: vane_para_runtime::PolkadotXcmConfig {
					safe_xcm_version: Some(SAFE_XCM_VERSION),
					..Default::default()
				},

				transaction_payment: Default::default(),

			};

			(None, Some(vane_para_runtime_genesis_config))

		}

	 };

	chain_spec
}

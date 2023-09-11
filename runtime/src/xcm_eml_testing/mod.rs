use xcm_emulator::*;
use integration_tests_common::{Polkadot,Rococo,AssetHubPolkadot};
use polkadot_core_primitives::AccountPublic;
use sp_core::{sr25519, sr25519::Pair as PairType, Pair, Public};
use sp_core::crypto::Ss58AddressFormatRegistry;
use sp_runtime::MultiSigner;
use sp_runtime::traits::IdentifyAccount;
use crate::AuraId;


const SAFE_XCM_VERSION: u32 = XCM_VERSION;

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
	where
		AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

pub fn template_session_keys(keys: AuraId) -> crate::SessionKeys {
	crate::SessionKeys { aura: keys }
}

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
	let public_str = format!("{:0<width$}", encoded_key, width = 64 + 2);

	// Convert hex public key to ss58 address
	let public = array_bytes::hex2bytes(&public_str).expect("Failed to convert hex to bytes");
	let public_key = Pair::Public::try_from(&public)
		.map_err(|_| "Failed to construct public key from given hex")?;

	Ok(public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into()))
}



pub mod accounts {
	use sp_core::sr25519;
	use crate::AuraId;
	use super::*;
	pub const ALICE: &str = "Alice";
	pub const BOB: &str = "Bob";
	pub const CHARLIE: &str = "Charlie";
	pub const DAVE: &str = "Dave";
	pub const EVE: &str = "Eve";


	pub fn init_balances() -> Vec<AccountId> {
		vec![
			get_account_id_from_seed::<sr25519::Public>(ALICE),
			get_account_id_from_seed::<sr25519::Public>(BOB),
			get_account_id_from_seed::<sr25519::Public>(CHARLIE),
			get_account_id_from_seed::<sr25519::Public>(DAVE),
			get_account_id_from_seed::<sr25519::Public>(EVE),

		]
	}

	pub fn invulnerables() -> Vec<(AccountId, AuraId)> {
		vec![
			(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_collator_keys_from_seed("Alice"),
			),
			(
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_collator_keys_from_seed("Bob"),
			),
		]
	}

	pub fn sudo_key() -> AccountId {
		get_account_id_from_seed::<sr25519::Public>(ALICE)
	}
}

pub mod vane_parachain {
	use super::*;
	use integration_tests_common::Storage;
	use sp_core::crypto::Ss58Codec;
	use crate::{EXISTENTIAL_DEPOSIT,Balance};
	use crate::xcm_eml_testing::accounts::{invulnerables, sudo_key};


	pub const PARA_ID: u32 = 2000;
	pub const ED: Balance = EXISTENTIAL_DEPOSIT;
	pub fn genesis() -> Storage {

		let dot_asset = MultiLocation{
			parents: 0,
			interior: X2(PalletInstance(10),GeneralIndex(1)).into()
		};

		let usdt_asset = MultiLocation{
			parents: 0,
			interior: X2(PalletInstance(10),GeneralIndex(2)).into()
		};

		let usdc_asset = MultiLocation{
			parents: 0,
			interior: X2(PalletInstance(10),GeneralIndex(3)).into()
		};

		let v_dot = "vDOT".as_bytes().to_vec();
		let _v_usdt = "vUSDT".as_bytes().to_vec();
		let _v_usdc = "vUSDC".as_bytes().to_vec();

		// Calculate parachain Soverign account id
		let sovererign_acount = calculate_sovereign_account::<PairType>(PARA_ID.into()).unwrap();
		let para_account = sp_runtime::AccountId32::from_ss58check(&sovererign_acount).unwrap();

		// ---******* GENESIS CONFIG ********---//

		let genesis_config = crate::RuntimeGenesisConfig {

			system: crate::SystemConfig {
				code: crate::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
				..Default::default()
			},

			balances: crate::BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, ED * 4096))
					.collect(),
			},

			vane_assets: crate::VaneAssetsConfig {

				metadata: vec![(dot_asset,v_dot.clone(), v_dot,10)],

				assets: vec![(dot_asset,para_account.clone(),true,1)],

				accounts: vec![(dot_asset,para_account.clone(),0)]

			},

			parachain_info: crate::ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
				..Default::default()
			},

			collator_selection: crate::CollatorSelectionConfig {
				invulnerables: invulnerables().iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
				..Default::default()
			},

			session: crate::SessionConfig {
				keys: invulnerables()
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

			polkadot_xcm: crate::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},

			sudo: crate::SudoConfig { key: sudo_key() },
			..Default::default()
		};

		genesis_config.build_storage().unwrap()

	}
}


decl_test_parachains!(
		pub struct VaneParachain {
		genesis = vane_parachain::genesis(),
		on_init = (),
		runtime = {
			Runtime: crate::Runtime,
			RuntimeOrigin: crate::RuntimeOrigin,
			RuntimeCall: crate::RuntimeCall,
			RuntimeEvent: crate::RuntimeEvent,
			XcmpMessageHandler: crate::XcmpQueue,
			DmpMessageHandler: crate::DmpQueue,
			LocationToAccountId: crate::xcm_config::LocationToAccountId,
			System: crate::System,
			Balances: crate::Balances,
			ParachainSystem: crate::ParachainSystem,
			ParachainInfo: crate::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: crate::PolkadotXcm,
			VaneAssets: crate::VaneAssets,
			VaneXcm: crate::VaneXcm,
			VanePayment: crate::VanePayment,

		}
	}
);

decl_test_networks!(
	// Polkadot
		pub struct PolkadotMockNet {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHubPolkadot,
			VaneParachain,
		],
		bridge = ()
	},

	// Rococo
	pub struct RococoMockNet {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			VaneParachain,
		],
		bridge = ()
	}
);

mod tests {

}

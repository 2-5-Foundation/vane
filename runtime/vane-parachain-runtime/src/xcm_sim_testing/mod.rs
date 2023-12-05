

mod parachain;
mod relay_chain;


use xcm_emulator::sp_tracing;
use sp_runtime::BuildStorage;
use staging_xcm::prelude::*;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};
use std::sync::Once;
use frame_support::assert_ok;
use xcm_emulator::bx;


pub const ALICE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([0u8; 32]);
pub const BOB: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([1u8; 32]);
pub const MRISHO: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([2u8; 32]);
pub const HAJI: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([3u8; 32]);

pub const VANE: sp_runtime::AccountId32 = sp_runtime::AccountId32::new([5u8; 32]);

pub const INITIAL_BALANCE: u128 = 1_000_000;

static INIT: Once = Once::new();
fn init_tracing() {
	INIT.call_once(|| {
		// Add test tracing (from sp_tracing::init_for_tests()) but filtering for xcm logs only
		let _ = tracing_subscriber::fmt()
			.with_max_level(tracing::Level::TRACE)
			.with_env_filter("xcm=trace,system::events=trace") // Comment out this line to see all traces
			.with_test_writer()
			.init();
	});
}

// Vane Parachain
decl_test_parachain! {
	pub struct Vane {
		Runtime = parachain::Runtime,
		XcmpMessageHandler = parachain::MsgQueue,
		DmpMessageHandler = parachain::MsgQueue,
		new_ext = para_ext(2000),
	}
}



decl_test_relay_chain! {
	pub struct Relay {
		Runtime = relay_chain::Runtime,
		RuntimeCall = relay_chain::RuntimeCall,
		RuntimeEvent = relay_chain::RuntimeEvent,
		XcmConfig = relay_chain::XcmConfig,
		MessageQueue = relay_chain::MessageQueue,
		System = relay_chain::System,
		new_ext = relay_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = Relay,
		parachains = vec![
			(2000, Vane),
		],
	}
}



use staging_xcm_executor::traits::ConvertLocation;
use vane_xcm_transfer_system::CurrencyId;

pub fn parent_account_id() -> parachain::AccountId {
	let location = (Parent,);
	parachain::LocationToAccountId::convert_location(&location.into()).unwrap()
}

pub fn child_account_id(para: u32) -> relay_chain::AccountId {
	let location = (Parachain(para),);
	relay_chain::LocationToAccountId::convert_location(&location.into()).unwrap()
}

pub fn child_account_account_id(para: u32, who: sp_runtime::AccountId32) -> relay_chain::AccountId {
	let location = (Parachain(para), AccountId32 { network: None, id: who.into() });
	relay_chain::LocationToAccountId::convert_location(&location.into()).unwrap()
}


pub fn parent_account_account_id(who: sp_runtime::AccountId32) -> parachain::AccountId {
	let location = (Parent, AccountId32 { network: None, id: who.into() });
	parachain::LocationToAccountId::convert_location(&location.into()).unwrap()
}


// Externatility for testing rserve based custom asset transfer
// whereby local asset is set to 0 and the Origin will mint the token
//
// The purpose is to make sure the Vane derivitive tokens to be in same supply with foreign chain asset
// And the issuer account is set to Vane sovereign Account
pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	// pallet_balances::GenesisConfig::<Runtime> {
	// 	balances: vec![
	// 		(ALICE, INITIAL_BALANCE),
	// 		(parent_account_id(), INITIAL_BALANCE),
	// 		(BOB,1_000_000)
	// 	],
	// }
	// .assimilate_storage(&mut t)
	// .unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			// (ALICE,INITIAL_BALANCE),
			(child_account_id(2000), INITIAL_BALANCE)
		],

	}.assimilate_storage(&mut t).unwrap();



	let asset1_name = "vDOT".as_bytes().to_vec();
	let _asset2_name = "vUSDT".as_bytes().to_vec();

	// pallet_assets::GenesisConfig::<Runtime> {
	//
	// 	metadata: vec![(asset1,asset1_name.clone(),asset1_name,10)],
	// 	assets: vec![(asset1,VANE,true,1)],
	// 	accounts: vec![(asset1,VANE,100_000)]
	//
	// }.assimilate_storage(&mut t).unwrap();

	pallet_assets::GenesisConfig::<Runtime> {

		metadata: vec![(CurrencyId::DOT,asset1_name.clone(),asset1_name,10)],
		assets: vec![(CurrencyId::DOT,child_account_id(2000),true,1)],
		accounts: vec![(CurrencyId::DOT,child_account_id(2000),0)]

	}.assimilate_storage(&mut t).unwrap();

	vane_xcm_transfer_system::GenesisConfig::<Runtime> {
		para_account: Some(child_account_id(2000)),
	}.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
		MsgQueue::set_para_id(para_id.into());
	});
	ext
}




pub fn relay_ext() -> sp_io::TestExternalities {
	use relay_chain::{Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, 100_000_000),
			(child_account_id(2000), 10),
 
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

pub type RelayChainPalletBalances = pallet_balances::Pallet<relay_chain::Runtime>;
pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type VanePalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type VanePalletAsset = pallet_assets::Pallet<parachain::Runtime>;
pub type VanePalletBalances = pallet_balances::Pallet<parachain::Runtime>;
pub type VaneXcmTransferSystem = vane_xcm_transfer_system::Pallet<parachain::Runtime>;
pub type FrameSystem  = frame_system::Pallet<parachain::Runtime>;

#[cfg(test)]
mod tests {

use super::*;



	// Helper function for forming buy execution message
	fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
		BuyExecution { fees: fees.into(), weight_limit: Unlimited }
	}



	// This test check that the Xcm Reserve Transfered Dot token from Relay Chain being deposited to the multi_id form between Alice & Bob
	// And all the necessary storage entities are taking place.
	// This functionality of directly depositing into multi id can be found in the implemented AssetTransactor::transfer.
	// AssetTransactor is responsible for handling token behaviour inside destination chain ( Note: check in staging_xcm_executor)
	#[test]
	fn transfer_dot_from_relay_to_vane_deposits_into_multi_id_works(){

		init_tracing();

		MockNet::reset();

		let amount = 100_000u128;
		let asset_amount = 1000u128;

		Relay::execute_with(||{

			// Send Xcm reserve Transfer with dest = para 1(vane) from Alice account to Bob
			let call = parachain::RuntimeCall::VaneXcmTransfer(vane_xcm_transfer_system::Call::tester {  });
			let inner_msg = Xcm::<()>(vec![
				Transact { origin_kind: OriginKind::SovereignAccount, require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024), call:call.encode().into() }
			]);

			let message = Xcm::<()>(vec![
				WithdrawAsset((Here, amount).into()),
				buy_execution((Here,amount)),
				DepositReserveAsset { assets: All.into(), dest: AccountId32 { network: None, id: BOB.into() }.into(), xcm: inner_msg }
			]);

			let inner_asset_messages = Xcm::<()>(
				vec![
					
					buy_execution((Here, amount)),
					DepositAsset { assets: All.into(), beneficiary: AccountId32 { network: None, id: ALICE.into() }.into() }
					// TransferAsset { assets: (MultiAsset::from(10::1)), beneficiary: AccountId32 { network: None, id: BOB.into() }.into()  }
				]
			);

			let asset_message = Xcm::<relay_chain::RuntimeCall>(vec![
				TransferReserveAsset { 
					assets: (Here, amount).into(),
					dest: (Parachain(2000).into()),
					xcm: inner_asset_messages
				}
				
			]);

			// assert_ok!(
			// 	relay_chain::XcmPallet::send(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(2000).into()),
			// 		bx!(VersionedXcm::V3(message))
			// 	)
			//
			// );

			// Normal ReserveAssetTransfer

			// Alice -> vane sovereign ---> Polkadot Relay
			// following instructions -> Buy execution on vane para and deposit equivalent asset to beneficiary account inside vane
			// assert_ok!(
			// 	relay_chain::XcmPallet::reserve_transfer_assets(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(2000).into()),
			// 		bx!(AccountId32 { network: None, id: ALICE.into() }.into()),
			// 		bx!((Here, amount).into()),
			// 		0
			// 	)
			// );

			// Transfer from Alice to Bob
			let vDotAsset = ( 
				X2(
					PalletInstance(10),
					GeneralIndex(1)
				),
				10000
			);
				

			// let transfer_message = Xcm::<()>(vec![
			// 	TransferAsset { assets: vDotAsset.into(), beneficiary: AccountId32 { network: None, id: BOB.into() }.into() }
			// ]);

			// assert_ok!(
			// 	relay_chain::XcmPallet::send(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(2000).into()),
			// 		bx!(VersionedXcm::V3(transfer_message))
			// 	)	
			// );

			// assert_ok!(
			// 	relay_chain::XcmPallet::send(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(2000).into()),
			// 		bx!(VersionedXcm::V3(()))

			// 	)
			// );

			// assert_ok!(
			// 	relay_chain::XcmPallet::send(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(2000).into()),
			// 		bx!(VersionedXcm::V3(asset_message))
			// 	)
			// );

			
			

			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

			// Check the vane soverign account
			// assert_eq!(
			// 	RelayChainPalletBalances::free_balance(child_account_id(2000)),
			// 	amount
			// )

		});

		// Emit Vane parachain events
		Vane::execute_with(||{

			//assert_ok!(VaneXcmTransferSystem::tester(parachain::RuntimeOrigin::signed(ALICE)));

			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));
			// Check the multi_id created account in pallet asset
			// assert_eq!(
			// 	VanePalletAsset::total_supply(CurrencyId::DOT),
			// 	100000
			// );

			// assert_eq!(
			// 	VanePalletAsset::balance(CurrencyId::DOT, ALICE),
			// 	amount
			// );

			// let txn_receipt = VaneXcmTransferSystem::get_payer_txn_receipt(ALICE,BOB);
			// println!("{:?}",txn_receipt)

		});

	}


	// This test checks transaction lifecycle from Relay Chain to Vane and back to Relay Chain with confirmations in place and fees token being deposited
	#[test]
	fn full_transaction_execution_and_confirmation_works(){

	}


	// This test checks reverting txn, sends xcm message to refund the tokens being held in vane soverign account.
	#[test]
	fn reverting_works(){

	}
}

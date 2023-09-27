

mod parachain;
mod relay_chain;

use frame_support::sp_tracing;
use sp_runtime::BuildStorage;
use xcm::prelude::*;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain, TestExt};
use std::sync::Once;


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
			//.with_max_level(tracing::Level::TRACE)
			//.with_env_filter("xcm=trace,system::events=trace") // Comment out this line to see all traces
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



use xcm_executor::traits::ConvertLocation;
use vane_primitive::CurrencyId;

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
			// (parent_account_id(), INITIAL_BALANCE)
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
		assets: vec![(CurrencyId::DOT,child_account_id(1),true,1)],
		accounts: vec![(CurrencyId::DOT,child_account_id(1),0)]

	}.assimilate_storage(&mut t).unwrap();

	vane_xcm::GenesisConfig::<Runtime> {
		para_account: Some(child_account_id(1)),
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
			(ALICE, 100_000),
			(child_account_id(1), 1000),

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
pub type VanePalletVaneXcm = vane_xcm::Pallet<parachain::Runtime>;

#[cfg(test)]
mod tests {
	use super::*;


	use frame_support::{assert_ok};
	use frame_support::macro_magic::__private::syn::token::In;
	use frame_support::traits::fungibles::Inspect;
	use sp_runtime::traits::Dispatchable;
	use xcm::v3::OriginKind::{Native, SovereignAccount};
	use xcm_emulator::bx;
	use xcm_simulator::TestExt;
	use vane_payment::Confirm;
	use vane_payment::helper::Token;

	// Helper function for forming buy execution message
	fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
		BuyExecution { fees: fees.into(), weight_limit: Unlimited }
	}

	#[test]
	fn vane_remote_soln1_native_token_works(){

		// Alice --> RC                                           RC
		//           -  (Reserve transfer)                         ^
		//           ˯                                             -
		//      Reserve Chain                                 Reserve Chain
		//           -  (Deposit Equivalent)                       ^
		//           ˯                                             -
		//         Vane  --------> MultiSig(Alice,Bob) --------> VaneXcm
		//           -        									   ^
		//           - ----------> Confirmation                    -
		//                          -                              -
		//                          --->Ms(A,B)--->Bob -------------


		sp_tracing::init_for_tests();


		MockNet::reset();

		let amount = 100_000u128;
		let asset_amount = 1000u128;
		//Relay Chain enviroment

		// Reserve Transfer native vane token from Relay to Vane Parachain
		Relay::execute_with(||{
			assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
				relay_chain::RuntimeOrigin::signed(ALICE),
				Box::new(Parachain(1).into()),
				Box::new(AccountId32 { network: None, id: ALICE.into() }.into()),
				Box::new((Here, amount).into()),
				0,
			));

			// Transfer reserve manually
			let inner_msg = Xcm::<()>(vec![

			]);

			let message = Xcm::<()>(vec![
				TransferReserveAsset {
					assets: (Here,amount).into(),
					dest: Parachain(1).into(),
					xcm: inner_msg,
				}
			]);

			// assert_ok!(
			// 	RelayChainPalletXcm::send(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		bx!(Parachain(1).into()),
			// 		bx!(VersionedXcm::V3(message))
			// 	)
			// );

			// Relay chain events
			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

			// Assert if the tokens are in Vane sovereign Account in the relay chain
			assert_eq!(
				relay_chain::Balances::free_balance(&child_account_id(2000)),
				amount + 1000
			);
		});
		//
		// let asset1 = MultiLocation{
		// 	parents: 0,
		// 	interior: X2(PalletInstance(10),GeneralIndex(1)).into()
		// };
		//
		//
		println!("Vane Area \n");

		Vane::execute_with(||{
			assert_eq!(
				VanePalletBalances::free_balance(ALICE),
				amount
			);
		//
		// 	// Test custom asset transfer
		// 	assert_ok!(
		// 		VanePalletAsset::transfer_keep_alive(
		// 			parachain::RuntimeOrigin::signed(VANE),
		// 			asset1,
		// 			MRISHO.into(),
		// 			1000
		// 		)
		// 	);
		//
		// 	assert_eq!(
		// 		VanePalletAsset::balance(asset1,VANE),
		// 		100_000 - 1000
		// 	);
		// 	assert_eq!(
		// 		VanePalletAsset::balance(asset1,MRISHO),
		// 		1000
		// 	);
		//
		// 	// Test custom asset transfer with local xcm execute
		// 	let asset_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::transfer_keep_alive {
		// 		id: asset1,
		// 		target: BOB.into(),
		// 		amount: asset_amount,
		// 	});
		//
		// 	let local_asset_message = Xcm::<parachain::RuntimeCall>(vec![
		// 		Transact {
		// 			origin_kind: SovereignAccount,
		// 			require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
		// 			call:asset_call.encode().into()
		// 		}
		//
		// 	]);
		//
		// 	assert_ok!(
		// 		VanePalletXcm::execute(
		// 			parachain::RuntimeOrigin::signed(VANE),
		// 			Box::new(VersionedXcm::V3(local_asset_message)),
		// 			Weight::from_parts(1_000_000_005, 1025 * 1024)
		// 		)
		// 	);
		//
		// 	assert_eq!(
		// 		VanePalletAsset::balance(asset1,VANE),
		// 		100_000 - asset_amount*2
		// 	);
		// 	assert_eq!(
		// 		VanePalletAsset::balance(asset1,BOB),
		// 		1000
		// 	);

			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});

	}

	#[test]
	fn vane_remote_soln1_custom_asset_derivitive_works(){
		MockNet::reset();


		// Test minting
		Vane::execute_with(||{
			assert_ok!(VanePalletAsset::mint(
				parachain::RuntimeOrigin::signed(child_account_id(1)),
				CurrencyId::DOT,
				MRISHO.into(),
				1000
			));

			// Check if the asset is minted
			assert_eq!(
				VanePalletAsset::balance(CurrencyId::DOT,MRISHO),
				1000
			);
			// Check total issuance & supply
			assert_eq!(
				VanePalletAsset::total_supply(CurrencyId::DOT) + VanePalletAsset::total_issuance(CurrencyId::DOT),
				2000
			);


			// Test minting with xcm_execute
			let asset_mint_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::mint {
				id: CurrencyId::DOT,
				beneficiary: BOB.into(),
				amount: 1000,
			});

			let local_asset_message = Xcm::<parachain::RuntimeCall>(vec![
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call:asset_mint_call.encode().into()
				}

			]);

			assert_ok!(
				VanePalletXcm::execute(
					parachain::RuntimeOrigin::signed(child_account_id(1)),
					Box::new(VersionedXcm::V3(local_asset_message)),
					Weight::from_parts(1_000_000_005, 1025 * 1024)
				)
			);

			// Check if the asset is minted
			assert_eq!(
				VanePalletAsset::balance(CurrencyId::DOT,BOB),
				1000
			);
			// Check total issuance & supply
			assert_eq!(
				VanePalletAsset::total_supply(CurrencyId::DOT) + VanePalletAsset::total_issuance(CurrencyId::DOT),
				4000
			);

		});


	}

	#[test]
	fn vane_remote_soln1_custom_asset_derivitive_xcm_works(){
		MockNet::reset();

		//init_tracing();
		// Alice in relay chain initiates reserve based transfer
		Relay::execute_with(||{


			let asset_mint_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::mint {
				id: CurrencyId::DOT,
				beneficiary: ALICE.into(),
				amount: 1000,
			});

			let test_storing = parachain::RuntimeCall::VaneXcm(vane_xcm::Call::test_storing {
				acc: ALICE,
				num: 50,
			});

			let test_transfer = parachain::RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
				dest: BOB.into(),
				value: 1000,
			});

			let inner_message = Xcm::<()>(vec![
				Transact {
					origin_kind: SovereignAccount, // Try native & sovereign
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call: test_storing.encode().into(),
				}
			]);

			let xcm_dummy = Xcm::<()>(vec![

			]);

			// let outer_message = Xcm::<()>(vec![
			// 	TransferReserveAsset {
			// 		assets: (Here,1000).into(),
			// 		dest: Parachain(1).into(),
			// 		xcm: xcm_dummy,
			// 	}]
			// );

			// let outer_message_2 = Xcm::<relay_chain::RuntimeCall>(vec![
			// 	WithdrawAsset((Here,1000).into()),
			// 	DepositAsset {
			// 		assets: AllCounted(1).into(),
			// 		beneficiary: (Parachain(1).into()),
			// 	},
			// 	// InitiateReserveWithdraw {
			// 	// 	assets: AllCounted(1).into(),
			// 	// 	reserve: (Parachain(1).into()),
			// 	// 	xcm: xcm_dummy,
			// 	// }
			// ]);

 			assert_ok!(
				RelayChainPalletXcm::send(
					relay_chain::RuntimeOrigin::signed(ALICE),
					Box::new(Parachain(1).into()),
					Box::new(VersionedXcm::V3(inner_message.into()))
				)
			);

			// Call is filtered

			// assert_ok!(
			// 	RelayChainPalletXcm::execute(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		Box::new(VersionedXcm::V3(outer_message_2)),
			// 		Weight::from_parts(1_000_000_005,1025*1024),
			// 	)
			// );

			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

			// assert_eq!(
			//	relay_chain::Balances::free_balance(ALICE),
			// 	INITIAL_BALANCE -1000
			// );

			// assert_eq!(
			// 	relay_chain::Balances::free_balance(&child_account_id(1)),
			// 	1000
			// );


		});

		println!("Vane Area \n");
		// Relay chain sends Reserve Asset Deposited Instruction to Vana
		// But we add Transact instruction to manually mint the tokens

		Vane::execute_with(||{

			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));

			assert_eq!(
				VanePalletVaneXcm::get_test_stored(ALICE),
				50
			);
		});

	}

	#[test]
	fn vane_remote_soln1_custom_asset_derivitive_manual_xcm_works(){
		MockNet::reset();


		Relay::execute_with(||{

			// 1. Alice -> Parachain(1)
			assert_ok!(
				RelayChainPalletBalances::transfer_keep_alive(
					relay_chain::RuntimeOrigin::signed(ALICE),
					child_account_id(1).into(),
					1000
				)
			);

			// 2. Test remote signed transact instruction
			let test_call = parachain::RuntimeCall::VaneXcm(vane_xcm::Call::test_storing {
				acc: ALICE,
				num: 1000,
			});


			let asset_mint_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::mint {
				id: CurrencyId::DOT,
				beneficiary: ALICE.into(),
				amount: 1000,
			});

			let message = Xcm::<()>(vec![
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call: test_call.encode().into(),
				}
			]);

			// Will Try more on this

			// let transfer_msg = Xcm::<()>(vec![
			// 	Transact {
			// 		origin_kind: SovereignAccount,
			// 		require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
			// 		call: test_call.encode().into(),
			// 	},
			// 	WithdrawAsset((Here,1000).into()),
			// 	InitiateReserveWithdraw {
			// 		assets: All.into(),
			// 		reserve: X1(Parachain(1)).into(),
			// 		xcm: message,
			// 	},
			//
			// ]);

			assert_ok!(
				RelayChainPalletXcm::send(
					relay_chain::RuntimeOrigin::signed(ALICE),
					Box::new(X1(Parachain(1)).into()),
					Box::new(VersionedXcm::V3(message))
				)
			);

			assert_eq!(
				relay_chain::Balances::free_balance(&child_account_id(1)),
				1000
			);


			//
			// let message_2 = Xcm::<()>(vec![
			// 	TransferAsset {
			// 		assets: (Here, 1000).into(),
			// 		beneficiary: X1(Parachain(1)).into(),
			// 	}
			// ]);

			// ------ ** This call is filtered and its annoying ** ----------------

			// assert_ok!(
			// 	RelayChainPalletXcm::execute(
			// 		relay_chain::RuntimeOrigin::signed(ALICE),
			// 		Box::new(VersionedXcm::V3(message_2)),
			// 		Weight::from_parts(1_000_000_005,1025*1024)
			// 	)
			// );


			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});

		println!("Vane Area \n");

		Vane::execute_with(||{

			assert_eq!(
				VanePalletVaneXcm::get_test_stored(ALICE),
				1000
			);


			// Mint the equivalent tokens
			let asset_mint_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::mint {
				id: CurrencyId::DOT,
				beneficiary: ALICE.into(),
				amount: 1000,
			});

			let local_asset_message = Xcm::<parachain::RuntimeCall>(vec![
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call:asset_mint_call.encode().into()
				}

			]);

			assert_ok!(
				VanePalletXcm::execute(
					parachain::RuntimeOrigin::signed(child_account_id(1)),
					Box::new(VersionedXcm::V3(local_asset_message)),
					Weight::from_parts(1_000_000_005, 1025 * 1024)
				)
			);

			assert_eq!(
				VanePalletAsset::balance(CurrencyId::DOT,ALICE),
				1000
			);

			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));

			// UMP
			// from Vane's Alice to RC's Alice

			//1. Burn local assets, 2. XCM message sending from SeoverignAcc to Alice

			let asset_burn_call = parachain::RuntimeCall::VaneAssets(pallet_assets::Call::burn {
				id: CurrencyId::DOT,
				amount: 1000,
				who: ALICE.into(),
			});

			assert_ok!(
				asset_burn_call.dispatch(parachain::RuntimeOrigin::signed(child_account_id(1)))
			);

			assert_eq!(
				VanePalletAsset::balance(CurrencyId::DOT,ALICE),
				0
			);

			// After burning
			let return_message = Xcm::<()>(vec![
				TransferAsset {
					assets: (Here,1000).into(),
					beneficiary: AccountId32 { network: None, id: BOB.into() }.into(),
				}
			]);

			assert_ok!(
				VanePalletXcm::send_xcm(
					Here,
					Parent,
					return_message
				)
			);


		});

		Relay::execute_with(||{
			//Check
			assert_eq!(
				relay_chain::Balances::free_balance(BOB),
				1000 + 100_000
			);

		})

	}

	#[test]
	fn vane_remote_soln1_custom_asset_derivitive_manual_xcm_pallet_works(){

		init_tracing();

		MockNet::reset();

		// test AliasOrigin Instruction
		Relay::execute_with(||{
			println!("Parent Account : {:?}",parent_account_id());

			println!("Parent Account ALICE : {:?}",parent_account_account_id(ALICE));

			println!("Parachain Account : {:?}", child_account_id(1));

			// 1. Alice -> Parachain(1)
			assert_ok!(
				RelayChainPalletBalances::transfer_keep_alive(
					relay_chain::RuntimeOrigin::signed(ALICE),
					child_account_id(1).into(),
					1000
				)
			);



			// 2. Test remote signed transact instruction
			let test_transfer_call = parachain::RuntimeCall::VaneXcm(vane_xcm::Call::vane_transfer {
				payee: BOB.into(),
				amount: 1000,
				currency: Token::Dot,
				asset_id: CurrencyId::DOT,
			});

			let message = Xcm::<()>(vec![
				DescendOrigin(AccountId32 {network: None, id: ALICE.into() }.into()), // look into remote derived accounts
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call: test_transfer_call.encode().into(),
				}
			]);

			assert_ok!(
				RelayChainPalletXcm::send(
					relay_chain::RuntimeOrigin::signed(ALICE),
					Box::new(X1(Parachain(1)).into()),
					Box::new(VersionedXcm::V3(message))
				)
			);

			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});

		println!("Vane Area");

		Vane::execute_with(||{


			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});
	}

	#[test]
	fn vane_remote_soln1_custom_asset_derivitive_manual_xcm_confirm_pallet_works(){
		sp_tracing::init_for_tests();


		MockNet::reset();

		// test AliasOrigin Instruction
		Relay::execute_with(||{

			// 1. Alice -> Parachain(1)
			assert_ok!(
				RelayChainPalletBalances::transfer_keep_alive(
					relay_chain::RuntimeOrigin::signed(ALICE),
					child_account_id(1).into(),
					1000
				)
			);

			// Check Sovereign Account balance
			assert_eq!(
				relay_chain::Balances::free_balance(child_account_id(1)),
				1000
			);



			// 2. Test remote signed transact instruction
			let test_transfer_call = parachain::RuntimeCall::VaneXcm(vane_xcm::Call::vane_transfer {
				payee: BOB.into(),
				amount: 1000,
				currency: Token::Dot,
				asset_id: CurrencyId::DOT,
			});

			println!(" Encoded Transfer Call : {:?}",test_transfer_call.encode());

			let message = Xcm::<()>(vec![
				DescendOrigin(AccountId32 {network: None, id: ALICE.into() }.into()), // look into remote derived accounts
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call: test_transfer_call.encode().into(),
				}
			]);

			assert_ok!(
				RelayChainPalletXcm::send(
					relay_chain::RuntimeOrigin::signed(ALICE),
					Box::new(X1(Parachain(1)).into()),
					Box::new(VersionedXcm::V3(message.clone()))
				)
			);


		});

		println!("Vane Area");

		Vane::execute_with(||{
			// Receiving the msg and executing
		});

		Vane::execute_with(||{


			// confirm the transaction
			// Payee confirming on vane chain
			// get the reference_no
			//let receipt = vane_payment::PayerTxnReceipt::<parachain::Runtime>::get(ALICE,BOB);

			let tt = vane_payment::PayeeTxnReceipt::<parachain::Runtime>::get(BOB);

			//println!(" Receipts: {:#?}",tt[0].reference_no);

			assert_ok!(
			 VanePalletVaneXcm::vane_confirm(
					parachain::RuntimeOrigin::signed(BOB),
					Confirm::Payee,
					tt[0].reference_no.to_vec(),
					tt[0].amount,
					CurrencyId::DOT
				)
			);

		});

		println!("Relay Area");

		Relay::execute_with(||{
			// Confirm for Payer


			let hardcoded_ref = vec![ 179, 58, 27, 164, 106, 103];
			// 2. Test remote signed transact instruction
			let test_confirm_call = parachain::RuntimeCall::VaneXcm(vane_xcm::Call::vane_confirm {
				who: Confirm::Payer,
				amount: 1000, // hardcoded but it should be taken from receipt
				asset_id: CurrencyId::DOT,
				reference_no: hardcoded_ref,
			});

			let message = Xcm::<()>(vec![
				DescendOrigin(AccountId32 {network: None, id: ALICE.into() }.into()), // look into remote derived accounts
				Transact {
					origin_kind: SovereignAccount,
					require_weight_at_most: Weight::from_parts(1_000_000_000,1024*1024),
					call: test_confirm_call.encode().into(),
				}
			]);

			assert_ok!(
				RelayChainPalletXcm::send(
					relay_chain::RuntimeOrigin::signed(ALICE),
					Box::new(X1(Parachain(1)).into()),
					Box::new(VersionedXcm::V3(message.clone()))
				)
			);

			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});

		println!("Vane Area");

		Vane::execute_with(||{
			// Execution of Payer confirmation
			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));

		});

		// Check for BOB balance in Relay Chain
		Relay::execute_with(||{
			assert_eq!(
				relay_chain::Balances::free_balance(BOB),
				999
			);
			assert_eq!(
				relay_chain::Balances::free_balance(child_account_id(1)),
				1
			);
		})


	}

	#[test]
	fn vane_remote_soln2_works(){

	}
}

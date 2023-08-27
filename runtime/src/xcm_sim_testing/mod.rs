

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
		new_ext = para_ext(1),
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
			(1, Vane),
		],
	}
}

use xcm_executor::traits::ConvertLocation;

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


pub fn para_ext(para_id: u32) -> sp_io::TestExternalities {
	use parachain::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![
			(ALICE, INITIAL_BALANCE),
			(parent_account_id(), INITIAL_BALANCE),
			(BOB,1_000_000)
		],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	// Vane asset
	let asset1 = MultiLocation{
		parents: 0,
		interior: X2(PalletInstance(10),GeneralIndex(1)).into()
	};

	let asset_name = "vDot".as_bytes().to_vec();

	pallet_assets::GenesisConfig::<Runtime> {

		metadata: vec![(asset1,asset_name.clone(),asset_name,10)],
		assets: vec![(asset1,VANE,true,1)],
		accounts: vec![(asset1,VANE,100_000)]

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
			(ALICE, INITIAL_BALANCE),
			(child_account_id(1), INITIAL_BALANCE),
			(BOB,1_000_000)
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


pub type RelayChainPalletXcm = pallet_xcm::Pallet<relay_chain::Runtime>;
pub type VanePalletXcm = pallet_xcm::Pallet<parachain::Runtime>;
pub type VanePalletAsset = pallet_assets::Pallet<parachain::Runtime>;

#[cfg(test)]
mod tests {
	use super::*;


	use frame_support::{assert_ok};
	use xcm_simulator::TestExt;

	// Helper function for forming buy execution message
	fn buy_execution<C>(fees: impl Into<MultiAsset>) -> Instruction<C> {
		BuyExecution { fees: fees.into(), weight_limit: Unlimited }
	}

	#[test]
	fn vane_remote_soln1_works(){

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


		//init_tracing();

		MockNet::reset();

		let amount = 100_000u128;
		let asset_amount = 1000u128;
		//Relay Chain enviroment

		// Reserve Transfer from Relay to Vane Parachain
		Relay::execute_with(||{
			assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
				relay_chain::RuntimeOrigin::signed(ALICE),
				Box::new(Parachain(1).into()),
				Box::new(AccountId32 { network: None, id: ALICE.into() }.into()),
				Box::new((Here, amount).into()),
				0,
			));

			// Relay chain events
			relay_chain::System::events().iter().for_each(|e| println!("{:#?}",e));

			// Assert if the tokens are in Vane sovereign Account in the relay chain
			assert_eq!(
				relay_chain::Balances::free_balance(&child_account_id(1)),
				INITIAL_BALANCE + amount
			);
		});


		let asset1 = MultiLocation{
			parents: 0,
			interior: X2(PalletInstance(10),GeneralIndex(1)).into()
		};

		Vane::execute_with(||{

			// Test custom asset transfer
			assert_ok!(
				VanePalletAsset::transfer_keep_alive(
					parachain::RuntimeOrigin::signed(VANE),
					asset1,
					MRISHO.into(),
					1000
				)
			);

			assert_eq!(
				VanePalletAsset::balance(asset1,VANE),
				100_000 - 1000
			);
			assert_eq!(
				VanePalletAsset::balance(asset1,MRISHO),
				1000
			);

			// Test custom asset transfer with local xcm execute

			let local_asset_message = Xcm::<parachain::RuntimeCall>(vec![
				WithdrawAsset(MultiAssets::from( vec![MultiAsset{
					id: Concrete(asset1),
					fun: asset_amount.into()
				}])),

				//  buy_execution(MultiAsset {
				// 	 id: Concrete(asset1),
				// 	fun: amount.into()
				// }),
				// DepositAsset { assets: All.into(), beneficiary: AccountId32 { network: None, id: BOB.into() }.into() },
			]);

			assert_ok!(
				VanePalletXcm::execute(
					parachain::RuntimeOrigin::signed(VANE),
					Box::new(VersionedXcm::V3(local_asset_message)),
					Weight::from_parts(1_000_000_005, 1025 * 1024)
				)
			);

			parachain::System::events().iter().for_each(|e| println!("{:#?}",e));


			// Assert if the tokens are in Vane sovereign Account in the relay chain
			// assert_eq!(
			// 	relay_chain::Balances::free_balance(&child_account_id(1)),
			// 	INITIAL_BALANCE + amount
			// );


			// let messages =  VersionedXcm::<()>::V3(Xcm(vec![
			// 	WithdrawAsset(MultiAssets::from( vec![MultiAsset{
			// 		id: AssetId::Concrete(MultiLocation::here()),
			// 		fun: Fungibility::Fungible(amount)
			// 	}])),
			// 	BuyExecution {
			// 		fees: MultiAsset{
			// 			id: AssetId::Concrete(MultiLocation::here()),
			// 			fun: Fungibility::Fungible(amount)
			// 		},
			// 		weight_limit: Unlimited
			// 	},
			// 	DepositAsset {
			// 		assets: MultiAssetFilter::Wild(WildMultiAsset::All),
			// 		beneficiary: MultiLocation { parents: 0, interior: Junctions::X1(Junction::AccountId32 {
			// 			network: None,
			// 			id: BOB.into()
			// 		})}
			// 	}
			// ]));

			// let messages = VersionedXcm::V3(Xcm(vec![
			// 	TransferAsset {
			// 		assets: MultiAssets::from( vec![MultiAsset{
            //             id: AssetId::Concrete(MultiLocation::here()),
            //             fun: Fungibility::Fungible(amount)
            //         }]),

			// 		beneficiary: MultiLocation { parents: 0, interior: Junctions::X1(Junction::AccountId32 { network: None, id: BOB.into() }) }
			// 	}
			// ]));



			// let messages = Xcm::<()>(vec![
			// 	TransferAsset {
			// 		 assets: (Here, amount).into(),
			// 		 beneficiary: MultiLocation { parents: 0, interior: X1(AccountId32 { network: None, id: BOB.into() }) }
			// 		}
			// ]);



			// WithdrawAsset(MultiAssets::from(vec![
			// 	MultiAsset {
			// 		id: Concrete(AccountId32 { network: None, id: ALICE.into() }.into()),
			// 		fun: amount.into()
			// 	}
			// ])),

			// buy_execution(MultiAsset {
			// 	id: Concrete(AccountId32 { network: None, id: ALICE.into() }.into()),
			// 	fun: amount.into()
			// }),


			//
			// let message = Xcm::<()>(vec![
			//
			// 	WithdrawAsset(( Concrete(AccountId32 { network: None, id: ALICE.into() }.into()), amount).into()),
			//
			//  	//buy_execution((Here, amount)),
			//
			// 	DepositAsset { assets: All.into(), beneficiary: AccountId32 { network: None, id: BOB.into() }.into() },
			// ]);
			//
			// let call = relay_chain::RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive { dest: MRISHO.into(), value: amount.into() });

			// local balance transfer
			// let local_message = Xcm::<()>(vec![
			// 	// TransferAsset {
			// 	// 	assets: (Here, amount).into(),
			// 	// 	beneficiary: MultiLocation { parents: 0, interior: Junctions::X1(Junction::AccountId32 { network: None, id: BOB.into() }) }
			// 	//    }
			// 	DescendOrigin(Junctions::from(AccountId32 { network: None, id: ALICE.into() })),
			// 	// Transact {
			// 	// 	origin_kind: OriginKind::SovereignAccount,
			// 	// 	require_weight_at_most: Weight::from_parts(1_000_000_000, 1024 * 1024),
			// 	// 	call: call.encode().into()
			// 	// },
			// 	WithdrawAsset((Here, amount).into()),
			// 	buy_execution((Here,amount)),
			// 	DepositAsset { assets: All.into(), beneficiary: AccountId32 { network: None, id: MRISHO.into() }.into() },
			//
			// 	//DescendOrigin(Junctions::from(AccountId32 { network: None, id: BOB.into() }))
			// ]);

			// assert_ok!(
			// 	VanePalletXcm::execute(
			// 		parachain::RuntimeOrigin::signed(ALICE),
			// 		Box::new(VersionedXcm::V3(local_message)),
			// 		Weight::from_parts(1_000_000_005, 1025 * 1024)
			// 	)

			// );


			// assert_ok!(
			// 	VanePalletXcm::send_xcm(Here, Parent, local_message)
			// );
			//
			// parachain::System::events().iter().for_each(|e| println!("{:#?}",e));
			// println!("\n");

			// assert_eq!(
			// 	parachain::Balances::free_balance(MRISHO),
			// 	100_000
			// );

			// assert_ok!(
			// 	VanePalletXcm::send(
			// 		parachain::RuntimeOrigin::signed(ALICE),
			// 		Box::new(Parent.into()),
			// 		Box::new(xcm::VersionedXcm::V3(message.clone().into()))
			// 	)
			// );

				//X1(AccountId32 { network: None, id: ALICE.into() }.into())

			//  assert_ok!(VanePalletXcm::send(

			// 	parachain::RuntimeOrigin::signed(ALICE),
			// 	Box::new(xcm::VersionedMultiLocation::V3(MultiLocation::parent())),
			// 	Box::new(messages)

			// ));



		});


	}

	#[test]
	fn vane_remote_soln2_works(){

	}
}

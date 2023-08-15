#![cfg_attr(not(feature = "std"), no_std)]

use super::pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use utils::*;

pub mod utils {
    use frame_system::RawOrigin;
    use sp_runtime::{MultiAddress, traits::StaticLookup};
    use vane_payment::helper::Token;
    use xcm::{
        v3::{
            MultiLocation,Junctions,Junction,
            MultiAssetFilter, WildMultiAsset,
            AssetId,Fungibility ,Xcm, WeightLimit,
            Instruction, MultiAsset, MultiAssets
        },
        VersionedXcm
    };
    use sp_std::{vec::Vec,vec};
    use sp_std::boxed::Box;
    
    use super::*;
    
    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
    
    impl<T: Config> Pallet<T>{
        pub fn vane_multisig_record(
            payer: T::AccountId,
            payee: T::AccountId,
            amount: u128,
            currency: Token
        ) -> Result<T::AccountId,Error<T>>{

            let accounts = vane_payment::AccountSigners::<T>::new(payee.clone(), payer.clone(), None);
			let multi_id = vane_payment::Pallet::<T>::derive_multi_id(accounts.clone());

            let ref_no = vane_payment::Pallet::<T>::derive_reference_no(payer.clone(), payee.clone(), multi_id.clone());

			vane_payment::AllowedSigners::<T>::insert(&payer, &ref_no, accounts);

            let ticket =
				vane_payment::TxnTicket::<T>::new(payee.clone(), payer.clone(), ref_no.clone(), amount.clone(),Some(currency));
			// Store to each storage item for txntickets
			// Useful for getting refrence no for TXN confirmation
			vane_payment::PayeeTxnTicket::<T>::mutate(&payee, |p_vec| p_vec.push(ticket.clone()));

			vane_payment::PayerTxnTicket::<T>::mutate(&payer, &payee, |p_vec| p_vec.push(ticket.clone()));

			vane_payment::Pallet::<T>::create_multi_account(multi_id.clone()).map_err(|_| Error::<T>::UnexpectedError)?;

			let time = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::MultisigAccountCreated {
				id: multi_id.clone(),
				time,
			});

            Ok(multi_id)
        }


        pub fn vane_xcm_transfer_dot(
            origin: OriginFor<T>,
            amount: u128,
            multi_id: T::AccountId // Multi Id Account
        ) -> DispatchResult{

            // Construct an XCM message vector to be executed at RelayChain
            // 1. Withdraw Asset
            // 2. Buy Execution
            // 3. Deposit Asset
            let payee:[u8;32] = multi_id.encode().try_into().unwrap();
            let dest = xcm::VersionedMultiLocation::V3(MultiLocation::parent());

            // We dont put error handling atm
            let message = VersionedXcm::<()>::V3(Xcm(
                vec![
                    // put to holding register
                    Instruction::<()>::WithdrawAsset(MultiAssets::from( vec![MultiAsset{
                        id: AssetId::Concrete(MultiLocation::here()),
                        fun: Fungibility::Fungible(amount)
                    }])),

                    // buy weight for xcm execution
                    Instruction::<()>::BuyExecution {
                        fees: MultiAsset{
                            id: AssetId::Concrete(MultiLocation::here()),
                            fun: Fungibility::Fungible(amount)
                        },
                        weight_limit: WeightLimit::Unlimited // At the moment we dont restrict how much weight should be used, xcm message should use as much weight as it needs
                    },
                    // Add Deposit Asset to vane parachain soverign account as fees

                    // Deposit funds to beneficiary address from holding register
                    Instruction::<()>::DepositAsset {
                        assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                        beneficiary: MultiLocation { parents: 0, interior: Junctions::X1(Junction::AccountId32 {
                            network: None,
                            id: payee.into()
                        })}
                    }
                ]
            ));
                
            
            pallet_xcm::Pallet::<T>::send(origin, Box::from(dest), Box::from(message))?;
            // Event
            
            Ok(())
        }



        pub fn vane_xcm_confirm_transfer_dot(
            payee: T::AccountId,
            amount: u128
        ) -> DispatchResult{

            // Check if we can limit only certain origin to be able to execute this messages from multi_id account
            // Construct an XCM message vector to be executed at RelayChain
            // 1. Transact instruction with origin a signed(multi_id)

            let payee_inner:[u8;32] = payee.clone().encode().try_into().unwrap();

            let dest = xcm::VersionedMultiLocation::V3(MultiLocation::parent());

            let message = VersionedXcm::<()>::V3(Xcm(
                vec![
                    // put to holding register
                    Instruction::<()>::WithdrawAsset(MultiAssets::from( vec![MultiAsset{
                        id: AssetId::Concrete(MultiLocation::here()),
                        fun: Fungibility::Fungible(amount)
                    }])),

                    // buy weight for xcm execution
                    Instruction::<()>::BuyExecution {
                        fees: MultiAsset{
                            id: AssetId::Concrete(MultiLocation::here()),
                            fun: Fungibility::Fungible(amount)
                        },
                        weight_limit: WeightLimit::Unlimited // At the moment we dont restrict how much weight should be used, xcm message should use as much weight as it needs
                    },
                    // Add Deposit Asset to vane parachain soverign account as fees

                    // Deposit funds to beneficiary address from holding register
                    Instruction::<()>::DepositAsset {
                        assets: MultiAssetFilter::Wild(WildMultiAsset::All),
                        beneficiary: MultiLocation { parents: 0, interior: Junctions::X1(Junction::AccountId32 {
                            network: None,
                            id: payee_inner.into()
                        })}
                    }
                ]
            ));

            // pallet xcm Call send
            pallet_xcm::Pallet::<T>::send(RawOrigin::Signed(payee).into(), Box::from(dest), Box::from(message))?;
            // Event

            Ok(())
        }


        pub fn vane_xcm_transfer_assethub_dot() -> DispatchResult{
            Ok(())
        }

        pub fn vane_xcm_confirm_transfer_assethub_dot() -> DispatchResult{

            Ok(())
        }
    }
}

#![cfg_attr(not(feature = "std"), no_std)]

use super::pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use utils::*;
use pallet_assets;

pub mod utils {
	use frame_support::traits::UnfilteredDispatchable;
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
	use xcm::prelude::{GeneralIndex, PalletInstance, X2};

	use super::*;

    type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;



    impl<T: Config> Pallet<T>{
        pub fn vane_multisig_record(
            payer: T::AccountId,
            payee: T::AccountId,
            amount: u128,
            currency: Token
        ) -> Result<T::AccountId,Error<T>>{

			// ****** CRUCIAL ******
			// Check the balance receipt in Vane Soverign Account before proceeding


			let accounts = vane_payment::AccountSigners::<T>::new(payee.clone(), payer.clone(), None);
			let multi_id = vane_payment::Pallet::<T>::derive_multi_id(accounts.clone());

			let multi_id_account_lookup = T::Lookup::unlookup(multi_id.clone());


			let ref_no = vane_payment::Pallet::<T>::derive_reference_no(payer.clone(), payee.clone(), multi_id.clone());

			vane_payment::AllowedSigners::<T>::insert(&payer, &ref_no, accounts);

			//if the multi_id is the same as previous Receipt just increment the total amount and add the txn_no amount

            let receipt =
				vane_payment::TxnReceipt::<T>::new(payee.clone(), payer.clone(),multi_id_account_lookup, ref_no.clone(), amount.clone(),(amount),Some(currency));

			// Store to each storage item for txntickets
			// Useful for getting reference no for TXN confirmation
			// Start with the payee storage

			vane_payment::PayeeTxnReceipt::<T>::mutate(&payee, |p_vec|{
				// Check if the multi_id already exists in the receipts and get its index of the receipt
				let index = p_vec.iter().position(|receipt| receipt.multi_id == multi_id_account_lookup);
				if let Some(idx) = index {
					// Get the receipt
					let mut receipt = p_vec.get(idx).ok_or(Error::<T>::UnexpectedError)?;
					receipt.update_txn(amount)

				}else{
					p_vec.push(receipt.clone())
				}
			});

			// Update for Payer/Sender Txn receipt
			let existing_payer_receipt = vane_payment::PayerTxnReceipt::<T>::get(&payer,&payee);

			if let Some(receipt) = existing_payer_receipt {

				vane_payment::PayerTxnReceipt::<T>::mutate(&payer, &payee, |receipt_inner|{
					receipt_inner.update_txn(amount)
				});

			}else{

				vane_payment::PayerTxnReceipt::<T>::insert(&payer,&payee,receipt);
			}


			vane_payment::Pallet::<T>::create_multi_account(multi_id.clone()).map_err(|_| Error::<T>::UnexpectedError)?;

			let time = <frame_system::Pallet<T>>::block_number();

			Self::deposit_event(Event::MultisigAccountCreated {
				id: multi_id.clone(),
				time,
			});

            Ok(multi_id)
        }


        pub fn vane_xcm_transfer_dot(
            amount: u128,
            multi_id: AccountIdLookupOf<T>, // Multi Id Account
			asset_id: T::AssetIdParameter
		) -> DispatchResult {


			// Deposit Amount to MultiSig Account
			let issuer = ParaAccount::<T>::get().unwrap();

			let amount: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			<pallet_assets::Pallet<T>>::mint(
				RawOrigin::Signed(issuer).into(),
				asset_id,
				multi_id,
				amount // handle this error
			)?;


			// Event

            Ok(())
        }



        pub fn vane_xcm_confirm_transfer_dot(

            payee: T::AccountId,
			multi_id: AccountIdLookupOf<T>,
            amount: u128,
			asset_id: T::AssetIdParameter

        ) -> DispatchResult{

			let issuer = ParaAccount::<T>::get().unwrap();

			// Burn the asset in the multi_id account
			let amount: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			<pallet_assets::Pallet<T>>::burn(
				RawOrigin::Signed(issuer).into(),
				asset_id,
				multi_id,
				amount
			)?;
			// Change the status in the ticket to be Sent


			// Send XCM instruction to send funds from Parachain sovereign account to payee acount



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

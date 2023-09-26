#![cfg_attr(not(feature = "std"), no_std)]

use super::pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use utils::*;
use pallet_assets;

pub mod utils {
	use core::ops::Add;
	use frame_support::traits::UnfilteredDispatchable;
	use frame_system::RawOrigin;
    use sp_runtime::{MultiAddress, traits::StaticLookup};
	use sp_runtime::DispatchError::BadOrigin;
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
	use xcm::latest::Parent;
	use xcm::prelude::{AccountId32, GeneralIndex, Here, PalletInstance, TransferAsset, X2};

	use super::*;




    impl<T: Config> Pallet<T>{

		// pub fn ensure_xcm_signed<OuterOrigin, AccountId>(o: OuterOrigin) -> Result<AccountId, Error<T>>
		// 	where
		// 		OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>>,
		// {
		// 	log::info!(
		// 			target:"",
		// 			"{:?}",
		// 			o
		// 		);
		// 	match o.into() {
		// 		Ok(RawOrigin::Signed(t)) => Ok(t),
		// 		_ => Err(Error::<T>::NotTheCaller),
		// 	}
		// }

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


			let ref_no = vane_payment::Pallet::<T>::derive_reference_no(payer.clone(), payee.clone(), multi_id.clone());

			vane_payment::AllowedSigners::<T>::insert(&payer, ref_no.to_vec(), accounts);

			//if the multi_id is the same as previous Receipt just increment the total amount and add the txn_no amount

            let receipt =
				vane_payment::TxnReceipt::<T>::new(payee.clone(), payer.clone(),multi_id.clone(), ref_no.clone(), amount.clone(),(amount),Some(currency));

			// Store to each storage item for txntickets
			// Useful for getting reference no for TXN confirmation
			// Start with the payee storage

			vane_payment::PayeeTxnReceipt::<T>::mutate(&payee, |p_vec|{
				// Check if the multi_id already exists in the receipts and get its index of the receipt
				let index = p_vec.iter().position(|receipt| receipt.multi_id == multi_id);
				if let Some(idx) = index {
					// Get the receipt
					let mut receipt = p_vec.get_mut(idx).ok_or(Error::<T>::UnexpectedError).unwrap();
					receipt.update_txn(amount);
					receipt.update_amount(amount)

				}else{
					p_vec.push(receipt.clone())
				}
			});

			// Update for Payer/Sender Txn receipt
			let existing_payer_receipt = vane_payment::PayerTxnReceipt::<T>::get(&payer,&payee);

			if let Some(receipt) = existing_payer_receipt {

				vane_payment::PayerTxnReceipt::<T>::mutate(&payer, &payee, |receipt_inner|{
					receipt_inner.clone().unwrap().update_txn(amount)
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
			multi_id_acc: T::AccountId, // Just for types difference usage but this and multi_id are sme value
			asset_id: T::AssetIdParameter
		) -> DispatchResult {


			// Deposit Amount to MultiSig Account
			let issuer = ParaAccount::<T>::get().unwrap();

			let balance: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			// Check if the multi_id account does contain the tokens
			let to_check_balance:<T as pallet_assets::Config>::Balance = 0u128.try_into().map_err(|_| Error::<T>::UnexpectedError)?;

			if <pallet_assets::Pallet<T>>::balance(asset_id.into(),multi_id_acc.clone()) != to_check_balance {

				let current_balance = <pallet_assets::Pallet<T>>::balance(asset_id.into(),multi_id_acc);
				let to_mint = current_balance.add(balance.try_into().expect("Failed to add balances in pallet  vane_xcm"));

				<pallet_assets::Pallet<T>>::mint(
					RawOrigin::Signed(issuer).into(),
					asset_id,
					multi_id.clone(),
					to_mint // handle this error
				)?;

			}else{
				<pallet_assets::Pallet<T>>::mint(
					RawOrigin::Signed(issuer).into(),
					asset_id,
					multi_id.clone(),
					balance // handle this error
				)?;
			}

			let time = <frame_system::Pallet<T>>::block_number();
			// Event
			Self::deposit_event(Event::DotXcmTransferInitiated {
				time,
				amount,
				multi_id,
			});

            Ok(())
        }



        pub fn vane_xcm_confirm_transfer_dot(
			payer: T::AccountId,
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

			// Change the status in the payer receipt


			// Send XCM instruction to send funds from Parachain sovereign account to payee acount
			let payee_id:[u8;32] = payee.encode().try_into().unwrap();

			let message = Xcm::<()>(vec![
				// Transfer equivalent funds from Sovereign Account to payee account
				TransferAsset {
					assets: (Here, 999).into(), // We must have a function to calculate fees
					beneficiary: (AccountId32 {network: None, id: payee_id}).into(),
				}
			]);

			<pallet_xcm::Pallet<T>>::send_xcm(Here,Parent,message).map_err(|_| Error::<T>::ErrorSendingXcm)?;

			// Change the status in the payee receipt


            // Event
			Self::deposit_event(
				Event::MessageTransferedToPolkadot
			);

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

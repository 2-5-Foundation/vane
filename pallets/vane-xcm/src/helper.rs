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
            amount: u128
        ) -> DispatchResult{





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

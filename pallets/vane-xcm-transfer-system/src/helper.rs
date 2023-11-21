#![cfg_attr(not(feature = "std"), no_std)]

use super::pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use utils::*;
use pallet_assets;

pub mod utils {
	use core::ops::Add;
	use frame_support::parameter_types;
	use sp_std::ops::{Mul, Sub};
	use frame_system::{AccountInfo, RawOrigin};
	use sp_runtime::traits::{TrailingZeroInput};
    use staging_xcm::{
        v3::{
            Xcm, WeightLimit,
        },

    };
    use sp_std::{vec::Vec,vec};
	use staging_xcm::latest::Parent;
	use staging_xcm::prelude::{AccountId32, All, BuyExecution, DepositAsset, Here, WithdrawAsset};
	use sp_io::hashing::blake2_256;


	use super::*;



	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountSigners<T: Config> {
		payee: T::AccountId,
		payer: T::AccountId,
	}
	impl<T> AccountSigners<T>
		where
			T: Config,
	{
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
		) -> Self {
			AccountSigners { payee, payer }
		}
		pub(super) fn get_accounts(&self) -> (&T::AccountId, &T::AccountId) {
			(&self.payer,&self.payee)
		}

		// refer here https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html?highlight=enum#enum-values

	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq,MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Token {
		DOT,
		USDT
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq,MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum XcmStatus {
		Sent,
		Completed,
		Tbc
	}

	// Confirmation enum which will be used to confirm the account_ids before dispatching multi-sig
	// Call
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm {
		Payer,
		Payee,
	}

	parameter_types! {
		pub const MAX_BYTES: u8 = 50;
		pub const MAX_NO_TXNS: u8 = 20;
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug,MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]

	pub struct TxnReceipt<T: Config> {
		payee: T::AccountId,
		payer: T::AccountId,
		pub multi_id: T::AccountId,
		pub amount: u128,
		pub reference_no: BoundedVec<u8,MAX_BYTES>,
		currency: Option<Token>,
		no_txn: BoundedVec<u128, MAX_NO_TXNS>,
		pub xcm_status: XcmStatus
	}

	impl<T: Config> TxnReceipt<T> {
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
			multi_id: T::AccountId,
			ref_no: BoundedVec<u8,MAX_BYTES>,
			amount: u128,
			txn: u128,
			currency:Option<Token>
		) -> Self {

			let mut no_txn = BoundedVec::new();
			no_txn.to_vec().push(txn);

			Self {
				payee, payer, reference_no: ref_no,
				amount,currency, no_txn,
				xcm_status: XcmStatus::Tbc, multi_id
			}
		}

		pub fn update_txn(&mut self, txn: u128){
			self.no_txn.try_push(txn).unwrap() // Put error handling
		}

		pub fn update_amount(&mut self, amount: u128){
			self.amount += amount
		}
	}


	// Call executed struct information
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CallExecuted<T: Config> {
		payer: T::AccountId,
		payee: T::AccountId,
		allowed_multi_id: T::AccountId,
		confirmed_multi_id: T::AccountId,
		proof: T::Hash,
		time: BlockNumberFor<T>,
	}

	impl<T> CallExecuted<T>
		where
			T: Config,
	{
		pub(super) fn new(
			payer: T::AccountId,
			payee: T::AccountId,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId,
			proof: T::Hash,
			time: BlockNumberFor<T>,
		) -> Self {
			CallExecuted { payer, payee, allowed_multi_id, confirmed_multi_id, proof, time }
		}
	}



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


			let accounts = AccountSigners::<T>::new(payee.clone(), payer.clone());
			let multi_id = Self::derive_multi_id(accounts.clone());


			let ref_no = Self::derive_reference_no(payer.clone(), payee.clone(), multi_id.clone());

			AllowedSigners::<T>::insert(&payer, ref_no.to_vec(), accounts);

			//if the multi_id is the same as previous Receipt just increment the total amount and add the txn_no amount

            let receipt =
				TxnReceipt::<T>::new(payee.clone(), payer.clone(),multi_id.clone(), ref_no.clone(), amount.clone(),(amount),Some(currency));

			// Store to each storage item for txntickets
			// Useful for getting reference no for TXN confirmation
			// Start with the payee storage

			PayeeTxnReceipt::<T>::mutate(&payee, |p_vec|{
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
			let existing_payer_receipt = PayerTxnReceipt::<T>::get(&payer,&payee);

			if let Some(mut receipt) = existing_payer_receipt {

				// vane_payment::PayerTxnReceipt::<T>::mutate(&payer, &payee, |receipt_inner|{
				// 	receipt_inner.clone().unwrap().update_txn(amount)
				// });
				receipt.update_txn(amount);
				receipt.update_amount(amount);

			}else{

				PayerTxnReceipt::<T>::insert(&payer,&payee,receipt);
			}


			Self::create_multi_account(multi_id.clone()).map_err(|_| Error::<T>::UnexpectedError)?;

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
			let amount_type: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			<pallet_assets::Pallet<T>>::burn(
				RawOrigin::Signed(issuer).into(),
				asset_id,
				multi_id.clone(),
				amount_type
			)?;

			// Change the status in the payer receipt


			// Send XCM instruction to send funds from Parachain sovereign account to payee acount
			//let payee_id:[u8;32] = payee.encode().try_into().unwrap();

			// Take 1 amount

			let new_amount = amount.sub(10_000_000_000);

			let message = Xcm::<()>(vec![
				// Transfer equivalent funds from Sovereign Account to payee account
				// TransferAsset {
				// 	assets: (Here, amount).into(), // We must have a function to calculate fees
				// 	beneficiary: (AccountId32 {network: None, id: payee.encode().try_into().unwrap()}).into(),
				// }
				WithdrawAsset((Here,new_amount).into()),
				BuyExecution { fees: (Here,new_amount).into(), weight_limit: WeightLimit::Unlimited },
				DepositAsset { assets: All.into(), beneficiary:  (AccountId32 {network: None, id: payee.encode().try_into().unwrap()}).into() }
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


		// Util functions

		pub fn derive_reference_no(
			payer: T::AccountId,
			payee: T::AccountId,
			multi_id: T::AccountId,
		) -> BoundedVec<u8,MAX_BYTES> {
			let mut buffer = Vec::new();
			buffer.append(&mut payer.using_encoded(blake2_256).to_vec());
			buffer.append(&mut payee.using_encoded(blake2_256).to_vec());
			buffer.append(&mut multi_id.using_encoded(blake2_256).to_vec());

			let reference = blake2_256(&buffer[..]);
			return reference[20..26].to_vec().try_into().unwrap(); // Proper error handling should be done
		}


		pub  fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId {

			let (acc1, acc2) = account_object.get_accounts();

			let entropy =  (b"vane/salt", acc1, acc2).using_encoded(blake2_256);

			let multi_account = Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed");

			multi_account
		}

		pub fn create_multi_account(multi_id: T::AccountId) -> DispatchResult {
			let account_info = AccountInfo::<T::Nonce, T::AccountData> { ..Default::default() };

			// Ensure the multi_id account is not yet registered in the storage
			if <frame_system::Pallet<T>>::account_exists(&multi_id) {
				return Ok(());
			} else {
				// Register to frame_system Account Storage item;
				<frame_system::Account<T>>::set(multi_id, account_info);
				Ok(())
			}
		}

	}


}

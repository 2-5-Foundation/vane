#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
mod helper;

#[frame_support::pallet]
mod pallet{
	use frame_support::Blake2_128;
	use frame_support::pallet_prelude::DispatchResult;
	use frame_system::pallet_prelude::OriginFor;
	use pallet_xcm;
	use vane_payment;
	use crate::helper;

	use frame_support::Blake2_128Concat;
	
	use frame_support::pallet_prelude::*;
	use frame_support::parameter_types;
	use frame_system::pallet_prelude::*;
	use vane_payment::helper::Token;
	use vane_payment::{Confirm,ConfirmedSigners};
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config + vane_payment::Config {
	
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}


	#[pallet::pallet]
	pub struct Pallet<T>(_);


	#[pallet::storage]
	pub type MultiSigToPayee<T: Config> = StorageMap<_,Blake2_128,T::AccountId,T::AccountId>;

	#[pallet::error]
	pub enum Error<T>{
		NotEnoughFees,
		UnexpectedError,
		NotSupportedYet
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		MultisigAccountCreated{
			time: T::BlockNumber,
			id: T::AccountId,
			// amount
		},
		DotXcmTransferInitiated {
			time: T::BlockNumber,
			sender: T::AccountId,
			multi_id: T::AccountId
			// TXN HASH for Dot side txn_hash: T::Hash,
		},
		PayerAddressConfirmedXcm {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
			reference_no: Vec<u8>,
		},
		PayeeAddressConfirmedXcm {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
			reference_no: Vec<u8>,
		}
	}



	#[pallet::call]
	impl<T: Config> Pallet<T>{
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn vane_transfer(
			origin: OriginFor<T>,
			payee: T::AccountId,
			amount: u128, // Fungibility
			currency: Token
		) -> DispatchResult{
			let payer = ensure_signed(origin.clone())?;

			// Construct a Multisig Account
			let multi_id = Self::vane_multisig_record(payer, payee, amount, currency.clone())?;

			// Check the Token type
			match currency {
				Token::Dot => {
					Self::vane_transfer(origin,multi_id,amount,currency)?;
				},
				Token::Usdt => {
					Err(Error::<T>::NotSupportedYet)?
				}
			};

			Ok(())
		}


		//Vane Transfer Confirmation

		#[pallet::call_index(1)]
		#[pallet::weight(10)]
		pub fn vane_confirm(
			origin: OriginFor<T>,
			who: vane_payment::Confirm,
			reference_no: Vec<u8>,
			amount: u128, // From PayerTxnTicket

		) -> DispatchResult {

			
			let user_account = ensure_signed(origin)?;
			// Check the storage
			let b_vec = vane_payment::ConfirmedSigners::<T>::get(reference_no.clone());

			if let Some(addr) = b_vec.get(0) {
				if addr.eq(&user_account) {
					return Err(vane_payment::Error::<T>::PayeeAlreadyConfirmed.into());

				// Else for checking if payee tries to confirm twice.
				} else {
					vane_payment::ConfirmedSigners::<T>::try_mutate(reference_no.clone(), |vec| {
						vec.try_push(user_account.clone())
					})
					.map_err(|_| vane_payment::Error::<T>::ExceededSigners)?;

					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::PayerAddressConfirmedXcm {
						account_id: user_account,
						timestamp: time,
						reference_no: reference_no.clone(),
					});

					// Construct AccountSigner object from ConfirmedSigners storage

					let confirmed_acc_signers = vane_payment::AccountSigners::<T>::new(
						vane_payment::ConfirmedSigners::<T>::get(reference_no.clone())
							.get(0)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),
						vane_payment::ConfirmedSigners::<T>::get(reference_no.clone())
							.get(1)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),
						// The default resolver is none but logic will be made to be customizable
						None,
					);

					// Derive the multi_id of newly constructed AccountSigner and one from
					// AllowedSigners
					let confirmed_multi_id = vane_payment::Pallet::<T>::derive_multi_id(confirmed_acc_signers);

					// Get the AllowedSigners from storage
					let payer = vane_payment::ConfirmedSigners::<T>::get(reference_no.clone())
						.get(1)
						.ok_or(Error::<T>::UnexpectedError)?
						.clone();

					let payee = vane_payment::ConfirmedSigners::<T>::get(reference_no.clone())
						.get(0)
						.ok_or(vane_payment::Error::<T>::UnexpectedError)?
						.clone();

					let allowed_signers =
						vane_payment::AllowedSigners::<T>::get(payer.clone(), reference_no.clone())
							.ok_or(vane_payment::Error::<T>::NotAllowedPayeeOrPaymentNotInitialized)?;

					let allowed_multi_id = vane_payment::Pallet::<T>::derive_multi_id(allowed_signers);
					// Compute the hash of both multi_ids (proof)
					if confirmed_multi_id.eq(&allowed_multi_id) {
						// Dipatch xcm call
						Self::vane_xcm_confirm_transfer_dot(payee,amount)?

					} else {
						return Err(vane_payment::Error::<T>::FailedToMatchAccounts.into());
					}
				}

			// Else block from If let Some()
			} else {
				match who {
					Confirm::Payer => return Err(vane_payment::Error::<T>::WaitForPayeeToConfirm.into()),

					Confirm::Payee => {
						ConfirmedSigners::<T>::try_mutate(reference_no.clone(), |vec| {
							vec.try_push(user_account.clone())
						})
						.map_err(|_| vane_payment::Error::<T>::ExceededSigners)?;

						let time = <frame_system::Pallet<T>>::block_number();

						Self::deposit_event(Event::PayeeAddressConfirmedXcm {
							account_id: user_account,
							timestamp: time,
							reference_no,
						});
					},
				};
			};

			Ok(())
		}
	
	
	}

}

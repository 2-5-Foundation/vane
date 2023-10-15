#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
pub mod helper;

// pub use orml_tokens;
// pub use orml_xtokens;
pub use orml_traits;
pub use orml_xcm_support;


#[frame_support::pallet]
mod pallet{
	use xcm::prelude::{GeneralIndex, PalletInstance,MultiLocation, X2};
	use log;
	use frame_support::Blake2_128;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use pallet_xcm;
	use vane_payment;
	use sp_runtime::traits::{StaticLookup};
	use frame_support::dispatch::RawOrigin;
	//use vane_primitive::CurrencyId;

	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use vane_payment::helper::Token;
	use vane_payment::{Confirm, ConfirmedSigners, TxnReceipt};
	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config + vane_payment::Config + pallet_assets::Config {

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	#[pallet::pallet]
	pub struct Pallet<T>(_);


	#[pallet::storage]
	pub type MultiSigToPayee<T: Config> = StorageMap<_,Blake2_128,T::AccountId,T::AccountId>;

	#[pallet::storage]
	pub type TestStorage<T: Config> = StorageMap<_,Blake2_128, T::AccountId, u32,ValueQuery>;

	#[pallet::storage]
	pub type ParaAccount<T: Config> = StorageValue<_,T::AccountId>;


	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config>{
		pub para_account: Option<T::AccountId>
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self){
			// panicks if account is not present
			let acc = &self.para_account.clone().unwrap();
			ParaAccount::<T>::put(acc);
		}
	}

	#[pallet::error]
	pub enum Error<T>{
		NotEnoughFees,
		UnexpectedError,
		NotSupportedYet,
		NotTheCaller,
		ErrorSendingXcm,
		ReceiptNotFound
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		MultisigAccountCreated{
			time: BlockNumberFor<T>,
			id: T::AccountId,
			// amount
		},
		DotXcmTransferInitiated {
			time: BlockNumberFor<T>,
			amount: u128,
			multi_id: AccountIdLookupOf<T>
			// TXN HASH for Dot side txn_hash: T::Hash,
		},
		PayerAddressConfirmedXcm {
			account_id: T::AccountId,
			timestamp:BlockNumberFor<T>,
			reference_no: Vec<u8>,
		},
		PayeeAddressConfirmedXcm {
			account_id: T::AccountId,
			timestamp: BlockNumberFor<T>,
			reference_no: Vec<u8>,
		},
		MessageTransferedToPolkadot,

	}



	#[pallet::call]
	impl<T: Config> Pallet<T>{
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn vane_transfer(
			origin: OriginFor<T>,
			payee: AccountIdLookupOf<T>,
			amount: u128, // Fungibility
			currency: Token,
			asset_id: T::AssetIdParameter

		) -> DispatchResult{
			// log the origin

			let caller = ensure_signed(origin.clone())?;
			log::info!(
				target: "",
				" Caller {:?}",
				caller,
			);


			let payee_acc = T::Lookup::lookup(payee.clone())?;

			//ensure!( caller_acc == payer, Error::<T>::NotTheCaller);
			// Construct a Multisig Account

			let multi_id = Self::vane_multisig_record(caller, payee_acc, amount, currency.clone())?;

			// Check the Token type

			match currency {
				Token::DOT => {

					let multi_id_acc = T::Lookup::unlookup(multi_id.clone());
					let asset = asset_id;

					Self::vane_xcm_transfer_dot(amount,multi_id_acc,multi_id,asset)?;
				},
				Token::USDT => {
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
			who: Confirm,
			reference_no: Vec<u8>,
			amount: u128, // From PayerTxnTicket
			asset_id: T::AssetIdParameter
		) -> DispatchResult {


			let user_account = ensure_signed(origin)?;
			// Check the storage
			let b_vec = ConfirmedSigners::<T>::get(reference_no.clone());

			if let Some(addr) = b_vec.get(0) {
				if addr.eq(&user_account) {
					return Err(vane_payment::Error::<T>::PayeeAlreadyConfirmed.into());

				// Else for checking if payee tries to confirm twice.
				} else {
					ConfirmedSigners::<T>::try_mutate(reference_no.clone(), |vec| {
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
						ConfirmedSigners::<T>::get(reference_no.clone())
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
						// Dispatch xcm call

						let multi_id_multi_acc = T::Lookup::unlookup(allowed_multi_id);

						Self::vane_xcm_confirm_transfer_dot(payer,payee,multi_id_multi_acc,amount,asset_id)?

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

	impl<T: Config> Pallet<T>{

		pub fn read_payer_receipt(origin: OriginFor<T>,payee: T:: AccountId) -> Result<TxnReceipt<T>, DispatchError>{
			let payer = ensure_signed(origin)?;
			let receipt = vane_payment::PayerTxnReceipt::<T>::get(payer,payee).ok_or(Error::<T>::ReceiptNotFound)?;
			Ok(receipt)
		}

	}

}

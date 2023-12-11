#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
pub mod helper;

pub use helper::*;
use log;
use frame_support::Blake2_128;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use pallet_xcm;
use sp_runtime::traits::{StaticLookup};
use sp_std::vec::Vec;
use frame_support::parameter_types;


#[frame_support::pallet]
mod pallet{

	use super::helper::{TxnReceipt,CallExecuted,AccountSigners,Token,Confirm};
	use super::*;


	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_xcm::Config + pallet_assets::Config + pallet_balances::Config {

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

	// Max signers for Confirm Signers Bounded Vec
	parameter_types! {
		pub const MaxSigners: u16 = 2;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	// Number of multi-sig transactions done by a specific account_id
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_account_multitxns)]
	pub type AccountMultiTxns<T: Config> =
	StorageMap<_, Blake2_256, T::AccountId, Vec<CallExecuted<T>>, ValueQuery>;

	// Signers which will be stored when payer initiates the transaction,
	//This will be used to create a multi_id account which is shared with both payer and specified payee
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_allowed_signers)]
	pub type AllowedSigners<T: Config> =
	StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, Vec<u8>, AccountSigners<T>>;


	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_confirmed_signers)]

	// Signers who have confirmed the transaction which will be compared to allowed signers as verification process
	pub type ConfirmedSigners<T: Config> =
	StorageMap<_, Twox64Concat, Vec<u8>, BoundedVec<T::AccountId, MaxSigners>, ValueQuery>;

	// Number of reverted or faulty transaction a payer did
	#[pallet::storage]
	#[pallet::getter(fn get_failed_txn_payer)]
	pub type RevertedTxnPayer<T: Config> = StorageMap<_, Blake2_256, T::AccountId, u32, ValueQuery>;

	// Number of reverted or faulty transaction a payee did
	#[pallet::storage]
	#[pallet::getter(fn get_failed_txn_payee)]
	pub type RevertedTxnPayee<T: Config> = StorageMap<_, Blake2_256, T::AccountId, u32, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn get_payer_txn_receipt)]
	pub type PayerTxnReceipt<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		TxnReceipt<T>
	>;

	// TxnTicket Payee
	// This is used to notify the payee as their is new pending transaction which needs confirmation
	#[pallet::storage]
	#[pallet::unbounded]
	#[pallet::getter(fn get_payee_txn_receipt)]
	pub type PayeeTxnReceipt<T: Config> =
	StorageMap<_, Blake2_128Concat, T::AccountId, Vec<TxnReceipt<T>>, ValueQuery>;

	#[pallet::storage]
	pub type MultiSigToPayee<T: Config> = StorageMap<_,Blake2_128,T::AccountId,T::AccountId>;

	#[pallet::storage]
	#[pallet::getter(fn get_para_account)]
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

		ReceiptNotFound,

		FailedToMatchAccounts,

		MultiAccountExists,

		ExceededSigners,

		AccountAlreadyExist,

		WaitForPayeeToConfirm,

		WaitForPayerToConfirm,

		PayerAlreadyConfirmed,

		PayeeAlreadyConfirmed,

		NotAllowedPayeeOrPaymentNotInitialized,

		MultiSigCallFailed,

		TxnReceiptUnavailable
		
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
		XcmTokenTransferInitiated {
			time: BlockNumberFor<T>,
			amount: u128,
			multi_id: T::AccountId,
			token: T::AssetIdParameter
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
		#[pallet::weight(1000000000000000)]
		pub fn vane_transfer(
			origin: OriginFor<T>,
			payee: AccountIdLookupOf<T>,
			amount: u128, // Fungibility
			currency: Token,
			asset_id: T::AssetIdParameter

		) -> DispatchResult{
			// log the origin

			let payer = ensure_signed(origin.clone())?;
			

			let payee_acc = T::Lookup::lookup(payee.clone())?;

			//ensure!( caller_acc == payer, Error::<T>::NotTheCaller);
			// Construct a Multisig Account

			let multi_id = Self::vane_multisig_record(payer.clone(), payee_acc.clone(), amount, currency.clone())?;

			// Check the Token type

			match currency {
				Token::DOT => {

					let multi_id_acc = T::Lookup::unlookup(multi_id.clone());
					let asset = asset_id;

					Self::vane_xcm_transfer_dot(amount,payer,multi_id_acc,payee_acc,asset)?;
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
					return Err(Error::<T>::PayeeAlreadyConfirmed.into());

				// Else for checking if payee tries to confirm twice.
				} else {
					ConfirmedSigners::<T>::try_mutate(reference_no.clone(), |vec| {
						vec.try_push(user_account.clone())
					})
					.map_err(|_| Error::<T>::ExceededSigners)?;

					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::PayerAddressConfirmedXcm {
						account_id: user_account,
						timestamp: time,
						reference_no: reference_no.clone(),
					});

					// Construct AccountSigner object from ConfirmedSigners storage

					let confirmed_acc_signers = AccountSigners::<T>::new(
						ConfirmedSigners::<T>::get(reference_no.clone())
							.get(0)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),
						ConfirmedSigners::<T>::get(reference_no.clone())
							.get(1)
							.ok_or(Error::<T>::UnexpectedError)?
							.clone(),
					);

					// Derive the multi_id of newly constructed AccountSigner and one from
					// AllowedSigners
					let confirmed_multi_id = Self::derive_multi_id(confirmed_acc_signers);

					// Get the AllowedSigners from storage
					let payer = ConfirmedSigners::<T>::get(reference_no.clone())
						.get(1)
						.ok_or(Error::<T>::UnexpectedError)?
						.clone();

					let payee = ConfirmedSigners::<T>::get(reference_no.clone())
						.get(0)
						.ok_or(Error::<T>::UnexpectedError)?
						.clone();

					let allowed_signers =
						AllowedSigners::<T>::get(payer.clone(), reference_no.clone())
							.ok_or(Error::<T>::NotAllowedPayeeOrPaymentNotInitialized)?;

					let allowed_multi_id = Self::derive_multi_id(allowed_signers);
					// Compute the hash of both multi_ids (proof)
					if confirmed_multi_id.eq(&allowed_multi_id) {
						// Dispatch xcm call

						let multi_id_multi_acc = T::Lookup::unlookup(allowed_multi_id);

						Self::vane_xcm_confirm_transfer_dot(payer,payee,multi_id_multi_acc,amount,asset_id)?

					} else {
						return Err(Error::<T>::FailedToMatchAccounts.into());
					}
				}

			// Else block from If let Some()
			} else {
				match who {
					Confirm::Payer => return Err(Error::<T>::WaitForPayeeToConfirm.into()),

					Confirm::Payee => {
						ConfirmedSigners::<T>::try_mutate(reference_no.clone(), |vec| {
							vec.try_push(user_account.clone())
						})
						.map_err(|_| Error::<T>::ExceededSigners)?;

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

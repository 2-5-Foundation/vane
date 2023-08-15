#![cfg_attr(not(feature = "std"), no_std)]
//------------Inner descriptions-----------------------------------------//
// The pallet should be generic
// The main extrinsic is the multisig Call which consist of following inputs;
//          -origin (signed)
//          -reference = Option<>
//          -payee_address = Option<>
//
// The 'reference' should have an account_id associated with it.
// The call mainly intention is to be used when paying for a product
// whereby a seller and a buyer are the participants of the multi-sig call.
// But it can be used in any other usecases provided that
// the usecase marries the call requirements.
//
// What does the call do? inner function set_callers()
// will register the account_ids needed for making
// the call. First id will be the origin, second will be from reference object.
//
// The inner Call is balance's call transfer function.

//------------------------------------------------------------------------------------------//
// We must keep track of seller and buyer bad behaviours in storage item so that we can introduce
// further punishments when bad repeated behaviour occurs

pub use pallet::*;

pub mod helper;

// A multi-signature implementation for `Vane Payment System`

#[frame_support::pallet]
pub mod pallet {
	use crate::helper::TxnTicketOrder;

	pub use super::helper::{
		AccountSigners, CallExecuted, Confirm, ResolverChoice, RevertReasons, TxnTicket,
	};
	use frame_support::{
		pallet, pallet_prelude::*, parameter_types, traits::tokens::currency::Currency,
		Blake2_128Concat,
	};
	use frame_system::{ensure_signed, pallet_prelude::*};

	//use vane_primitive::OrderTrait;
	use sp_io::hashing::blake2_256;
	use sp_runtime::traits::{StaticLookup, TrailingZeroInput};
	use sp_std::vec::Vec;
	use vane_order;
	use vane_register::{self, BalanceOf};

	pub(super) type AccountOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOfPay<T> = <<T as Config>::Currency as Currency<AccountOf<T>>>::Balance;

	// Max signers for Confirm Signers Bounded Vec
	parameter_types! {
		pub const MaxSigners: u16 = 2;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::config]
	pub trait Config:
		frame_system::Config + vane_register::Config + vane_order::Config + pallet_balances::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		//type Order: OrderTrait + TypeInfo + Decode + Encode + Clone + PartialEq + Debug;
		type Currency: Currency<Self::AccountId>;
	}

	// Not yet implemented
	#[pallet::storage]
	#[pallet::getter(fn get_resolver)]
	pub(super) type ResolverSigner<T: Config> = StorageValue<_, T::AccountId>;

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
	#[pallet::getter(fn get_signers)]

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

	// Order txn confirmation tracker
	// Key --> Payee_id
	// Value ----> (payer_id,payee_id)
	#[pallet::storage]
	pub type OrderTracker<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		u32,
		(T::AccountId, T::AccountId),
	>;

	// TxnTicket Payer
	// Showing pending uncofirmed txn
	// Keys ->(payer, payee) , allowing related txn to be in one place

	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayerTxnTicket<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Vec<TxnTicket<T>>,
		ValueQuery,
	>;

	// TxnTicket Payee
	// This is used to notify the payee as their is new pending transaction which needs confirmation
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayeeTxnTicket<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<TxnTicket<T>>, ValueQuery>;

	// Ignore the Order txn at the moment
	// Ticket for Order transactions
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayerTxnTicketOrder<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Blake2_128Concat,
		T::AccountId,
		Vec<TxnTicketOrder<T>>,
		ValueQuery,
	>;
	// TxnTicket Payee
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayeeTxnTicketOrder<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<TxnTicketOrder<T>>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallExecuted {
			multi_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		MultiAccountCreated {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
		},

		BalanceTransferredAndLocked {
			to_multi_id: T::AccountId,
			from: T::AccountId,
			timestamp: T::BlockNumber,
			reference_no: Vec<u8>,
		},

		PayeeAddressConfirmed {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
			reference_no: Vec<u8>,
		},

		PayerAddressConfirmed {
			account_id: T::AccountId,
			timestamp: T::BlockNumber,
			reference_no: Vec<u8>,
		},

		SubmittedPayment {
			from_account: T::AccountId,
			to_account: T::AccountId,
			amount: BalanceOfPay<T>,
			resolver: Option<ResolverChoice>,
			timestamp: T::BlockNumber,
		},
		SubmittedOrderPayment {
			from_account: T::AccountId,
			to_account: T::AccountId,
			amount: BalanceOf<T>,
			resolver: Option<ResolverChoice>,
			timestamp: T::BlockNumber,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		// Any system error
		UnexpectedError,

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
		// For Vane Register
		ProductNotFound,

		OrderNotFound,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Responsible for normal payments
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn vane_pay(
			origin: OriginFor<T>,
			payee: T::AccountId,
			amount: BalanceOfPay<T>,
			resolver: Option<ResolverChoice>,
		) -> DispatchResult {
			// 1. Check if the Payee is in the Register Storage
			// 2.
			let payer = ensure_signed(origin)?;

			match resolver {
				Option::None => {
					Self::inner_vane_pay_wo_resolver(payer.clone(), payee.clone(), amount,None)?;
					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::SubmittedPayment {
						from_account: payer,
						to_account: payee,
						amount,
						resolver: None,
						timestamp: time,
					})
				},
				_ => (),
			}

			Ok(())
		}

		/// Responsible for handling order type payments
		#[pallet::call_index(1)]
		#[pallet::weight(10)]
		pub fn vane_order_pay(
			origin: OriginFor<T>,
			seller: T::AccountId,
			item_no: u32,
			resolver: Option<ResolverChoice>,
		) -> DispatchResult {
			let buyer_id = ensure_signed(origin)?;
			match resolver {
				Option::None => {
					// Every item_no is linked with seller_id, as there can be same order_no referencing
					// different seller_ids.
					// seller_id -------> item_no ------(i)
					// Verifying If the Item is actually there ---- (ii)
					// PayeeProducts ---> [ProductProfile]
					let vec_products = <vane_register::PayeeProducts<T>>::get(&seller);
					let check_product =
						vec_products.iter().find(|product| product.product_id == item_no);
					ensure!(matches!(check_product, Some(_prod)), Error::<T>::ProductNotFound);

					// Check if if the order does exist -----(iii)
					// order_no + payer_id ------> Check order existence

					// Storage entities
					// PayeeRef ----> seller_id --> [(buyer_id,item_no,order_no)]
					// Iterate over and look for matching (buyer_id + item_no) and take  the order_no
					let vec_ref = <vane_order::PayeeOrderRef<T>>::get(&seller);
					let (_, _, order_no) = vec_ref
						.iter()
						.find(|(buyer_id, item_no, _)| buyer_id == buyer_id && item_no == item_no)
						.unwrap();

					// Payer -----> buyer_id --> [Order]
					// Iterate over and look for matching order_no + seller_id
					// Take the Order object ,, NOTE: Dont delete yet untill confirmation period
					let vec_order = <vane_order::PayerOrder<T>>::get(&buyer_id);
					let order = vec_order.iter().find(|ord| ord.order_no == *order_no);
					ensure!(matches!(order, Some(_ord)), Error::<T>::OrderNotFound);
					let order = order.unwrap();
					// -------------------------------------------
					Self::inner_vane_order_pay_wo_resolver(
						order.payer_id.clone(),
						order.payee_id.clone(),
						order.amount.clone(),
					)?;
					let time = <frame_system::Pallet<T>>::block_number();

					Self::deposit_event(Event::SubmittedOrderPayment {
						from_account: order.payer_id.clone(),
						to_account: order.payee_id.clone(),
						resolver: None,
						timestamp: time,
						amount: order.amount,
					});
				},
				_ => (),
			}

			Ok(())
		}

		// Get the confirm account address and store them in Signers Storage Item. Sort and make
		// sure buyer's address is first
		// Always make sure if its the buyer, he should be first in the vector,
		// 		1. Store the account_id in the Signer Storage Item,
		// 		2. Then next steps will follow after this,

		#[pallet::call_index(2)]
		#[pallet::weight(10)]
		pub fn confirm_pay(
			origin: OriginFor<T>,
			who: Confirm,
			reference_no: Vec<u8>,
		) -> DispatchResult {
			// 1. Check if 0 index is a occupied and if true check if its a Payee if true return Err
			// 2. If its not a Payee then add new account which it will be a Payer
			// 3. If index 0 is not occupied then check if the address is a Payer, if its true
			// return Err 4. If the address is a Payee then push it to the vec.
			//---------------------------------------------------------------------------------------//
			// This will ensure that in 0th index is always Payee address and the Payer cannot
			// confirm first

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

					Self::deposit_event(Event::PayerAddressConfirmed {
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
						// The default resolver is none but logic will be made to be customizable
						None,
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
						let encoded_proof = (allowed_multi_id.clone(), confirmed_multi_id.clone())
							.using_encoded(blake2_256);
						let proof =
							Decode::decode(&mut TrailingZeroInput::new(encoded_proof.as_ref()))
								.map_err(|_| Error::<T>::UnexpectedError)?;

						Self::dispatch_transfer_call(
							proof,
							payer,
							payee,
							allowed_multi_id,
							confirmed_multi_id,
						)?;
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

						Self::deposit_event(Event::PayeeAddressConfirmed {
							account_id: user_account,
							timestamp: time,
							reference_no,
						});
					},
				};
			};

			Ok(())
		}

		// If the payer accidently makes a mistake due to RevertReasons the funds can be refunded
		// back Punishment will occur if the reason is personal.

		// We should introduce some sort of limit for WrongAddress reason occurrence.
		// Fee punishments on wrong address to limit spamming (ideally 1% of amount)
		#[pallet::call_index(5)]
		#[pallet::weight(10)]
		pub fn revert_fund(origin: OriginFor<T>, reason: RevertReasons) -> DispatchResult {
			Ok(())
		}
	}
}

#![cfg_attr(not(feature = "std"), no_std)]
// Setting up OCW

pub mod ocw;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		ensure,
		pallet_prelude::{DispatchResult, *},
		traits::UnfilteredDispatchable,
		Blake2_128Concat,
	};
	use frame_system::{ensure_root, offchain::AppCrypto, pallet_prelude::*, RawOrigin};

	use frame_system::offchain::{
		CreateSignedTransaction, SendSignedTransaction, SignedPayload, Signer, SigningTypes,
		SubmitTransaction,
	};

	use vane_payment::{BalanceOfPay, Confirm, ResolverChoice};
	use vane_register;

	use sp_std::vec::Vec;

	#[pallet::config]
	pub trait Config:
		vane_payment::Config + CreateSignedTransaction<Call<Self>> + frame_system::Config
	{
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(_n: BlockNumberFor<T>) {}
	}

	// Call mapping
	//(Pallet Index, Call Index)
	#[derive(Encode, Decode, TypeInfo, RuntimeDebug, Clone)]
	#[scale_info(skip_type_params(T))]
	pub enum PaymentCalls<T: Config> {
		VanePay {
			signer: T::AccountId,
			payee: T::AccountId,
			amount: u128,
			resolver: Option<ResolverChoice>,
		},
		ConfirmPay {
			signer: T::AccountId,
			who: Confirm,
			reference: Vec<u8>,
		},
	}

	// Other Pallets Calls enum

	// List of AccountId Requesting Calls executions
	// Keeping track on the latest and update the list
	#[pallet::storage]
	#[pallet::unbounded]
	pub type TrackAccounts<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	// Key is the user password
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PaymentCallMemPool<T: Config> =
		StorageMap<_, Blake2_128Concat, Vec<u8>, Vec<PaymentCalls<T>>, ValueQuery>;

	// Storing the delegant signer using Sudo
	#[pallet::storage]
	pub type DelegatedSigner<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CallPlaced { who: T::AccountId, which: Vec<u8> },
	}

	#[pallet::error]
	pub enum Error<T> {
		CallPlacingFailed,
		UnAuthorized,
		CallDispatchFailed,
		UnexpectedError, // System Error
	}

	// OCW Functions
	impl<T: Config> Pallet<T> {
		pub fn register_delegator() -> Result<(), &'static str>
		where
			<T as frame_system::Config>::AccountId: frame_system::offchain::AppCrypto<
				<T as SigningTypes>::Public,
				<T as SigningTypes>::Signature,
			>,
		{
			Ok(())
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Set delagated signer
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn set_delegated_signer(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			let _caller = ensure_root(origin)?;
			DelegatedSigner::<T>::set(Some(account));
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10)]
		pub fn place_vane_pay(
			origin: OriginFor<T>,
			signer: T::AccountId,
			payee: T::AccountId,
			amount: u128,
			password: Vec<u8>,
		) -> DispatchResult {
			// Check if it matches the registered delegated signer For security
			let caller = ensure_signed(origin)?;
			let d_signer = DelegatedSigner::<T>::get().unwrap(); // Handle error
			ensure!(caller == d_signer, Error::<T>::UnAuthorized);

			// Place vane_pay call
			let call = PaymentCalls::<T>::VanePay {
				signer: signer.clone(),
				payee: payee.clone(),
				amount,
				resolver: None,
			};
			// Store in the CallsMemPool
			PaymentCallMemPool::<T>::try_mutate(password, |calls| {
				calls.push(call);
				Ok(())
			})
			.map_err(|_: Error<T>| Error::<T>::UnAuthorized)?;

			// Try manual dispatch
			let VaneCall = vane_payment::Call::<T>::vane_pay { payee, amount, resolver: None }
				.dispatch_bypass_filter(RawOrigin::Signed(signer).into())
				.map_err(|_| Error::<T>::CallDispatchFailed)?;

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10)]
		pub fn place_vane_confirm(
			origin: OriginFor<T>,
			signer: T::AccountId,
			reference: Vec<u8>,
			who: u8, // 1 = Payee, 2= Payer
			password: Vec<u8>,
		) -> DispatchResult {
			// Check if it matches the registered delegated signer For security
			let caller = ensure_signed(origin)?;
			let d_signer = DelegatedSigner::<T>::get().unwrap(); // Handle error
			ensure!(caller == d_signer, Error::<T>::UnAuthorized);

			let mut confirm = Confirm::Payee;

			if who == 1 {
				confirm = Confirm::Payee;
			} else if who == 2 {
				confirm = Confirm::Payer;
			};

			let call = PaymentCalls::<T>::ConfirmPay {
				signer: signer.clone(),
				who: confirm.clone(),
				reference: reference.clone(),
			};

			// Storage
			// Store in the CallsMemPool
			PaymentCallMemPool::<T>::try_mutate(password, |calls| {
				calls.push(call);
				Ok(())
			})
			.map_err(|_: Error<T>| Error::<T>::UnAuthorized)?;

			// Try manual dispatch
			let VaneCall =
				vane_payment::Call::<T>::confirm_pay { who: confirm, reference_no: reference }
					.dispatch_bypass_filter(RawOrigin::Signed(signer).into())
					.map_err(|_| Error::<T>::CallDispatchFailed)?;

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10)]
		pub fn place_vane_revert(origin: OriginFor<T>, signer: T::AccountId) -> DispatchResult {
			Ok(())
		}

		// Pallet Register Calls
	}
}

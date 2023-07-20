#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;


#[frame_support::pallet]
mod pallet{
	use frame_support::Blake2_128Concat;
	
	use frame_support::pallet_prelude::*;
	use frame_support::parameter_types;
	use frame_system::pallet_prelude::*;
   

	#[pallet::config]
	pub trait Config: frame_system::Config{
		type Currency: Currency<Self::AccountId>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	


	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::error]
	pub enum Error<T>{
		AccountAlreadyRegistered,
		UserIsNotRegistered
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>{
		PayerRegistered{
			id: T::AccountId,
			time: T::BlockNumber
		},
	
	}




	#[pallet::call]
	impl<T: Config> Pallet<T>{
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn test_xcm(
			origin: OriginFor<T>,
		) -> DispatchResult{


			Ok(())
		}
	
	
	}

}

#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;
pub mod helper;

#[frame_support::pallet]
mod pallet{
	use frame_support::Blake2_128Concat;
	
	use frame_support::pallet_prelude::*;
	use frame_support::parameter_types;
	use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;
    use sp_std::vec::Vec;
	use frame_support::{traits::{Currency, ExistenceRequirement}};
    use crate::helper::{PayeeAccountProfile,ProductProfile, PayerAccountProfile};
	use crate::helper::utils::{Confirm};

	#[pallet::config]
	pub trait Config: frame_system::Config{
		type Currency: Currency<Self::AccountId>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	parameter_types! {
		pub const MaxVaneId:u8 = 5;
	}
	//Vane id
	#[derive(RuntimeDebug, Clone, TypeInfo, MaxEncodedLen, Encode, Decode, PartialEq)]
	pub struct VaneId(pub BoundedVec<u8,MaxVaneId>);


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
			time: BlockNumberFor<T>
		},
		PayeeRegistered {
			id: T::AccountId,
			time: BlockNumberFor<T>
		}
	}


	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayeeStorage<T:Config> =
	StorageMap<_,Blake2_128,T::AccountId,PayeeAccountProfile<T>>;

	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayerStorage<T: Config> =
	StorageMap<_,Blake2_128,T::AccountId,PayerAccountProfile<T>>;

	// Add first time buyer quick buying feature without registrations


	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayeeProducts<T: Config> =
	StorageMap<_,Blake2_128Concat,T::AccountId,Vec<ProductProfile<T>>, ValueQuery>;


	#[pallet::call]
	impl<T: Config> Pallet<T>{
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn register_payer(

			origin: OriginFor<T>,
			name:Option<Vec<u8>>,
			email: Option<Vec<u8>>,
			kyc: Option<Vec<u8>>

		) -> DispatchResult{

			let signer = ensure_signed(origin)?;
			ensure!(PayerStorage::<T>::contains_key(signer.clone()),Error::<T>::AccountAlreadyRegistered);

			let time = <frame_system::Pallet<T>>::block_number();
			// Generate a VaneId
			// Without checking the kyc 
			let hash = (kyc.unwrap()).using_encoded(blake2_128);
			let five_bits_hash =  hash.to_vec()[..6].to_vec();

			let vane_id = VaneId(five_bits_hash.try_into().unwrap());
						
			let acc_profile = PayerAccountProfile::<T>::new(name,vane_id,email,signer.clone(),time);

			PayerStorage::<T>::insert(signer.clone(),acc_profile);
			Self::deposit_event(Event::PayerRegistered { id: signer , time });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10)]
		pub fn register_payee(
			origin:OriginFor<T>,
			name:Vec<u8>,ig_link:Vec<u8>,
			location: Vec<u8>
			//coordinates:(long,lat)
		) -> DispatchResult {

			let signer = ensure_signed(origin)?;
			ensure!(PayeeStorage::<T>::contains_key(signer.clone()),Error::<T>::AccountAlreadyRegistered);


				let time = <frame_system::Pallet<T>>::block_number();
			    let acc_profile = PayeeAccountProfile::<T>::new(name,signer.clone(),location,ig_link,time);

			    PayeeStorage::<T>::insert(signer.clone(),acc_profile);
			    Self::deposit_event(Event::PayeeRegistered { id: signer , time });

			    Ok(())
		}

		// A reference of product
		#[pallet::call_index(3)]
		#[pallet::weight(10)]
		pub fn update_products(
			origin:OriginFor<T>,
			product_id:u32,
			link:Vec<u8>,// url string
			amount: BalanceOf<T>,
			image_url:Option<Vec<u8>>
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			// Check if the seller is registered
			ensure!(<PayeeStorage<T>>::contains_key(seller.clone()), Error::<T>::UserIsNotRegistered);

			// Construct Product Object
			let product = ProductProfile::<T>::new(image_url,amount,product_id,seller.clone(),link);
			PayeeProducts::<T>::mutate(&seller,|p_vec|{
				p_vec.push(product)
			});
			Ok(())
		}
		//Idea on registering products
	}

}

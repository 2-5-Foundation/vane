#![cfg_attr(not(feature = "std"), no_std)]

mod helper;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::helper::{Order, OrderStatus};
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ExistenceRequirement},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;
	use vane_register;
	#[pallet::config]
	pub trait Config: frame_system::Config + vane_register::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId>;
	}

	//pub(super) type BalanceOf<T> = <<T as vane_register::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayerOrder<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Order<T>>, ValueQuery>;

	// (PayersId, item_id, order_no)
	// With payers id, item_id you can get the reference order from specific payeeId
	#[pallet::storage]
	#[pallet::unbounded]
	pub type PayeeOrderRef<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<(T::AccountId, u32, u32)>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		OrderPlaced { buyer: T::AccountId, order_no: u32, item_id: u32, seller: T::AccountId },

		OrderCancelled,
	}

	#[pallet::error]
	pub enum Error<T> {
		ProductDontExist,
		UnexpectedError, // System Error
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10)]
		pub fn place_order(
			origin: OriginFor<T>,
			item_id: u32,
			seller_id: T::AccountId,
		) -> DispatchResult {
			let payer = ensure_signed(origin)?;
			// Get the Product Object

			let products = <vane_register::PayeeProducts<T>>::get(seller_id.clone());
			let prod_pos = products
				.iter()
				.position(|p| p.product_id == item_id)
				.ok_or(Error::<T>::ProductDontExist)?;
			let product_to_order = products.get(prod_pos).ok_or(Error::<T>::UnexpectedError)?;

			// Take the existing order from the stated payee account and increment it
			let order_vec_payer = PayerOrder::<T>::get(&payer);

			let order = order_vec_payer.iter().find(|&ord| ord.payee_id == seller_id.clone());

			let order_no = order.unwrap().order_no + 1;
			let order_time = <frame_system::Pallet<T>>::block_number();
			// For testing

			let expected_time = order_time + order_time;

			let order = Order::<T>::new(
				order_no,
				item_id,
				product_to_order.amount,
				seller_id.clone(),
				payer.clone(),
				order_time,
				expected_time,
			);

			// Store to payer
			PayerOrder::<T>::mutate(&payer, |vec| vec.push(order));
			// Store the reference to Payee storage item
			PayeeOrderRef::<T>::mutate(&seller_id, |vec| {
				vec.push((payer.clone(), item_id, order_no))
			});
			// Event
			Self::deposit_event(Event::OrderPlaced {
				buyer: payer,
				item_id,
				order_no,
				seller: seller_id,
			});

			Ok(())
		}
	}
}

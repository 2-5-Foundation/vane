#![cfg_attr(not(feature = "std"), no_std)]

pub use utils::*;

use super::pallet::Config;
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use vane_primitive::OrderTrait;
use vane_register::BalanceOf;
pub mod utils {
	use super::*;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone)]
	pub enum OrderStatus {
		Completed,
		Initiated,
		Halted,  // Problem occured
		Pending, //Not yet paid
		TimeOut,
		CancelledByPayer,
		CancelledByPayee,
	}

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, RuntimeDebug, Clone)]
	#[scale_info(skip_type_params(T))]
	pub struct Order<T: Config> {
		pub order_no: u32,
		pub item_id: u32,
		pub payee_id: T::AccountId,
		pub payer_id: T::AccountId,
		pub amount: BalanceOf<T>,
		pub ordered_time: BlockNumberFor<T>,
		pub expected_time: BlockNumberFor<T>,
		pub status: OrderStatus,
	}

	impl<T: Config> Order<T> {
		pub fn new(
			order_no: u32,
			item_id: u32,
			amount: BalanceOf<T>,
			payee_id: T::AccountId,
			payer_id: T::AccountId,
			order_time: BlockNumberFor<T>,
			expect_time: BlockNumberFor<T>,
		) -> Self {
			Self {
				order_no,
				item_id,
				payee_id,
				amount,
				payer_id,
				ordered_time: order_time,
				expected_time: expect_time,
				status: OrderStatus::Initiated,
			}
		}
	}

	impl<T: Config> OrderTrait for Order<T> {
		fn get_order_number(&self) -> u32 {
			self.item_id
		}
	}
}

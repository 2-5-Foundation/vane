#![cfg_attr(not(feature = "std"), no_std)]



use super::pallet::{Config,BalanceOf};

pub use utils::*;

pub mod utils {
	use codec::{Decode, Encode};
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use frame_support::sp_runtime::Saturating;
	use pallet_balances::Reasons;
    use sp_std::{vec::Vec,vec};
    use crate::{BalanceOf, VaneId};
	
    use super::Config;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm {
		Payer,
		Payee,
	}

    // Payer AccountData
#[derive(Decode,Encode,TypeInfo,Clone,RuntimeDebug,PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct PayerAccountProfile<T: Config>{
	name: Option<Vec<u8>>,
	id: VaneId,
	email: Option<Vec<u8>>,
	account_id: T::AccountId,
	orders_completed:Option<u16>,
	genesis_time: BlockNumberFor<T>
}

impl<T:Config> PayerAccountProfile<T>{
	pub fn new(name:Option<Vec<u8>>,id:VaneId,email: Option<Vec<u8>>, account_id:T::AccountId, time: BlockNumberFor<T>) -> Self {
		Self{
			name,
			id,
			email,
			account_id,
			orders_completed:None,
			genesis_time: time
		}
	}
}


//Product Profile
#[derive(Encode,Decode,RuntimeDebug,Clone,TypeInfo,PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct ProductProfile<T:Config>{
	pub image_url:Option<Vec<u8>>,
	pub product_id:u32,
	pub seller: T::AccountId,
	pub link: Vec<u8>,
	pub amount: BalanceOf<T>,
	
}
impl<T:Config> ProductProfile<T>{
	pub fn new(image_url:Option<Vec<u8>>,amount: BalanceOf<T>, product_id:u32,seller:T::AccountId,link:Vec<u8>) -> Self{
		Self{
			image_url,
			product_id,
			amount,
			seller,
			link
		}
	}
}

// Actions for products

#[derive(Encode,Decode,TypeInfo,RuntimeDebug)]
pub enum Actions{
	Add,
	Remove
}
// Payee/Seller Account Profile
#[derive(Encode,Decode,RuntimeDebug,TypeInfo,PartialEq)]
#[scale_info(skip_type_params(T))]
pub struct PayeeAccountProfile<T: Config> {
	name: Vec<u8>,
	account_id: T::AccountId,
	location: Vec<u8>,
	ig_link: Vec<u8>,
	pub orders_completed: Option<u32>,
	pub orders_failed: Option<u16>,
	pub time_average:Option<u32>,
	genesis_time: BlockNumberFor<T>
}

impl<T:Config> PayeeAccountProfile<T> {
	pub fn new(name: Vec<u8>,account_id:T::AccountId, location: Vec<u8>,ig_link:Vec<u8>, time: BlockNumberFor<T>) -> Self{
		Self{
			name,
			account_id,
			location,
			ig_link,
			orders_completed: None,
			orders_failed: None,
			time_average: None,
			genesis_time: time
		}
	}

	
}

}


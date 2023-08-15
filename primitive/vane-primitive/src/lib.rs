#![cfg_attr(not(feature = "std"), no_std)]

use codec::MaxEncodedLen;
use codec::{Decode, Encode};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;

pub trait OrderTrait {
	fn get_order_number(&self) -> u32;
	//fn get_delivery_time(),
	//fn calculate_delivery_time()
}

// OrType Storage Key primitive
// A type of storage_map whereby 2 keys store one value
// Accessing the value only require one key which is registered.

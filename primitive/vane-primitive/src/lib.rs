#![cfg_attr(not(feature = "std"), no_std)]

use codec::{ MaxEncodedLen};
use codec::{Decode, Encode};
use sp_core::{crypto::{Ss58AddressFormatRegistry, Ss58Codec}};
use frame_support::{pallet_prelude::*, parameter_types};
use sp_std::boxed::Box;
use sp_runtime::{MultiSigner};
use scale_info::prelude::format;
use scale_info::prelude::string::String;


pub trait OrderTrait {
	fn get_order_number(&self) -> u32;
	//fn get_delivery_time(),
	//fn calculate_delivery_time()
}





//orml re written traits & macros and types
/// A trait for querying a value by a key.
pub trait GetByKey<Key, Value> {
	/// Return the value.
	fn get(k: &Key) -> Value;
}

/// Create new implementations of the `GetByKey` trait.
///
/// The implementation is typically used like a map or set.
///
/// Example:
/// ```ignore
/// use primitives::CurrencyId;
/// parameter_type_with_key! {
///     pub Rates: |currency_id: CurrencyId| -> u32 {
///         match currency_id {
///             CurrencyId::DOT => 1,
///             CurrencyId::KSM => 2,
///             _ => 3,
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! parameter_type_with_key {
	(
		pub $name:ident: |$k:ident: $key:ty| -> $value:ty $body:block;
	) => {
		pub struct $name;
		impl GetByKey<$key, $value> for $name {
			fn get($k: &$key) -> $value {
				$body
			}
		}
	};
}


// parameter_types! {
// 	pub const MAX_BYTES: u8 = 200;
// }

// #[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen)]
// pub struct RococoId(u32);

// pub fn calculate_sovereign_account<Pair>(
// 	para_id: u32,
// ) -> Result<String, ()>
// 	where
// 		Pair: sp_core::Pair,
// 		Pair::Public: Into<MultiSigner>,
// {
// 	// Scale encoded para_id
// 	let id = RococoId(2000).encode_hex();

// 	// Prefix para or sibl
// 	let prefix = "para".encode_hex();

// 	// Join both strings and the 0x at the beginning
// 	let encoded_key = "0x".to_owned() + &prefix + &id;

// 	// Fill the rest with 0s
// 	let public_str = format!("{:0<width$}", encoded_key, width = (64 + 2) as usize);

// 	// Convert hex public key to ss58 address
// 	let public = array_bytes::hex2bytes(&public_str).expect("Failed to convert hex to bytes");
// 	let public_key = Pair::Public::try_from(&public)
// 		.map_err(|_| "Failed to construct public key from given hex").unwrap();

// 	Ok(public_key.to_ss58check_with_version(Ss58AddressFormatRegistry::SubstrateAccount.into()))
// }
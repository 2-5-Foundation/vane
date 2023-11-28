#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::AccountInfo;
use sp_std::fmt::Debug;
use sp_std::result;
use codec::{FullCodec, MaxEncodedLen};
use codec::{Decode, Encode};
use frame_support::dispatch::{RawOrigin};
use frame_support::pallet_prelude::*;
use frame_support::sp_runtime::traits::StaticLookup;
use frame_support::traits::{ContainsPair, EnsureOriginWithArg, Everything, OriginTrait, UnfilteredDispatchable};
use frame_support::traits::fungibles::{Balanced, Inspect};
use frame_support::traits::tokens::{Fortitude, Precision, Preservation, WithdrawConsequence};
use scale_info::TypeInfo;
use staging_xcm::latest::prelude::*;
use sp_io::hashing::blake2_256;

//
// pub use primitive_mod::*;
// #[frame_support::pallet]
// pub mod primitive_mod {
// 	use super::*;
// 	#[pallet::config]
// 	pub trait Config: frame_system::Config + pallet_assets::Config {}
//
// 	#[pallet::pallet]
// 	pub struct Pallet<T>(sp_std::marker::PhantomData<T>);
//
// 	pub type Balance<T> = <T as pallet_assets::Config>::Balance;
//
//
//
//
// 	}
// }


pub trait OrderTrait {
	fn get_order_number(&self) -> u32;
	//fn get_delivery_time(),
	//fn calculate_delivery_time()
}

// OrType Storage Key primitive
// A type of storage_map whereby 2 keys store one value
// Accessing the value only require one key which is registered.


// For Multi Currency usage
//use orml_traits::{GetByKey, parameter_type_with_key};
use pallet_assets::{AssetDetails, Config};
use sp_core::serde::{Deserialize, Serialize};
use sp_runtime::traits::{AtLeast32BitUnsigned, Convert, Zero, TrailingZeroInput};
use sp_runtime::SaturatedConversion;
use staging_xcm_executor::Assets;
use staging_xcm_executor::traits::{ConvertLocation, Error, MatchesFungible, TransactAsset};
use pallet_xcm::{EnsureXcm, Origin as XcmOrigin};


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

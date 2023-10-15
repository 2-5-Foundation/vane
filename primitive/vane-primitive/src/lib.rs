#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::fmt::Debug;
use sp_std::result;
use codec::{FullCodec, MaxEncodedLen};
use codec::{Decode, Encode};
use frame_support::dispatch::{RawOrigin, UnfilteredDispatchable};
use frame_support::pallet_prelude::*;
use frame_support::traits::{ContainsPair, EnsureOriginWithArg, Everything, OriginTrait};
use frame_support::traits::fungibles::{Balanced, Inspect};
use frame_support::traits::tokens::{Fortitude, Precision, Preservation, WithdrawConsequence};
use frame_support::traits::tokens::Precision::BestEffort;
use frame_system::pallet_prelude::*;
use scale_info::TypeInfo;
use xcm::latest::prelude::*;

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
use orml_traits::{GetByKey, parameter_type_with_key};
use pallet_assets::{AssetDetails, Config};
use sp_core::serde::{Deserialize, Serialize};
use sp_runtime::traits::{AtLeast32BitUnsigned, Convert, StaticLookup, Zero};
use orml_xcm_support::{OnDepositFail, UnknownAsset};
use sp_runtime::SaturatedConversion;
use xcm_executor::Assets;
use xcm_executor::traits::{ConvertLocation, Error, MatchesFungible, TransactAsset};
use pallet_xcm::{EnsureXcm, Origin as XcmOrigin};

// Struct for matching Vane custom derived assets
pub struct VaneDerivedAssets;

impl ContainsPair<CurrencyId, MultiLocation> for VaneDerivedAssets {
	fn contains(a: &CurrencyId, b: &MultiLocation) -> bool {
		// match the MultiAsset && MultiLocation to point to only Polkadot Dot && AssetHub USDT (1987)
		match b {
			MultiLocation{parents:1, interior: Here} => {
				true
			},
			// For assetHub
			_ => false
		}
	}
}

impl ContainsPair<MultiAsset, MultiLocation> for VaneDerivedAssets {
	fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
		match b {
			MultiLocation{parents:1, interior: Here} => {
				true
			},
			// For assetHub
			_ => false
		}
	}
}


pub struct VaneForeignCreators<IsForeign, AccountOf, AccountId>(
	sp_std::marker::PhantomData<(IsForeign, AccountOf, AccountId)>,
);
impl<
	IsForeign: ContainsPair<CurrencyId, MultiLocation>,
	AccountOf: ConvertLocation<AccountId>,
	AccountId: Clone,
	RuntimeOrigin: From<XcmOrigin> + OriginTrait + Clone,
> EnsureOriginWithArg<RuntimeOrigin, CurrencyId>

for VaneForeignCreators<IsForeign, AccountOf, AccountId>
	where
		RuntimeOrigin::PalletsOrigin:
		From<XcmOrigin> + TryInto<XcmOrigin, Error = RuntimeOrigin::PalletsOrigin>,
{
	type Success = AccountId;

	fn try_origin(
		origin: RuntimeOrigin,
		asset_location: &CurrencyId,
	) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
		let origin_location = EnsureXcm::<Everything>::try_origin(origin.clone())?;
		if !IsForeign::contains(asset_location, &origin_location) {
			return Err(origin)
		}
		AccountOf::convert_location(&origin_location).ok_or(origin)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin(a: &MultiLocation) -> Result<RuntimeOrigin, ()> {
		Ok(pallet_xcm::Origin::Xcm(*a).into())
	}
}





#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, codec::MaxEncodedLen, TypeInfo, serde::Serialize,
serde::Deserialize,)]
pub enum CurrencyId {
	DOT,
	USDT,
	USDC
}


pub struct VaneMultiCurrencyAdapter<
	MultiCurrency,
	UnknownAsset,
	Match,
	AccountId,
	AccountIdConvert,
	CurrencyId,
	CurrencyIdConvert,
	DepositFailureHandler,
>(
	PhantomData<(
		MultiCurrency,
		UnknownAsset,
		Match,
		AccountId,
		AccountIdConvert,
		CurrencyId,
		CurrencyIdConvert,
		DepositFailureHandler,
	)>,
);

impl<
	MultiCurrency: VaneMultiCurrency<AccountId, CurrencyId = CurrencyId>,
	UnknownAsset: orml_xcm_support::UnknownAsset,
	Match: MatchesFungible<MultiCurrency::Balance>,
	AccountId: Debug + Clone,
	AccountIdConvert: ConvertLocation<AccountId>,
	CurrencyId: FullCodec + Eq + PartialEq + Copy + Debug,
	CurrencyIdConvert: Convert<MultiAsset, Option<CurrencyId>>,
	DepositFailureHandler: OnDepositFail<CurrencyId, AccountId, MultiCurrency::Balance>,
> TransactAsset
for VaneMultiCurrencyAdapter<
	MultiCurrency,
	UnknownAsset,
	Match,
	AccountId,
	AccountIdConvert,
	CurrencyId,
	CurrencyIdConvert,
	DepositFailureHandler,
>
{
	fn deposit_asset(asset: &MultiAsset, location: &MultiLocation, _context: &XcmContext) -> xcm::v3::Result {
		match (
			AccountIdConvert::convert_location(location),
			CurrencyIdConvert::convert(asset.clone()),
			Match::matches_fungible(asset),
		) {
			// known asset
			(Some(who), Some(currency_id), Some(amount)) => MultiCurrency::deposit(currency_id, &who, amount)
				.or_else(|err| DepositFailureHandler::on_deposit_currency_fail(err, currency_id, &who, amount)),
			// unknown asset
			_ => UnknownAsset::deposit(asset, location)
				.or_else(|err| DepositFailureHandler::on_deposit_unknown_asset_fail(err, asset, location)),
		}
	}

	fn withdraw_asset(
		asset: &MultiAsset,
		location: &MultiLocation,
		_maybe_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		UnknownAsset::withdraw(asset, location).or_else(|_| {
			let who = AccountIdConvert::convert_location(location)
				.ok_or_else(|| XcmError::from(Error::AccountIdConversionFailed))?;
			let currency_id = CurrencyIdConvert::convert(asset.clone())
				.ok_or_else(|| XcmError::from(Error::AssetIdConversionFailed))?;
			let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
				.ok_or_else(|| XcmError::from(Error::AssetNotHandled))?
				.saturated_into();
			MultiCurrency::withdraw(currency_id, &who, amount).map_err(|e| XcmError::FailedToTransactAsset(e.into()))
		})?;

		Ok(asset.clone().into())
	}

	fn transfer_asset(
		asset: &MultiAsset,
		from: &MultiLocation,
		to: &MultiLocation,
		_context: &XcmContext,
	) -> result::Result<Assets, XcmError> {
		let from_account =
			AccountIdConvert::convert_location(from).ok_or_else(|| XcmError::from(Error::AccountIdConversionFailed))?;
		let to_account =
			AccountIdConvert::convert_location(to).ok_or_else(|| XcmError::from(Error::AccountIdConversionFailed))?;
		let currency_id = CurrencyIdConvert::convert(asset.clone())
			.ok_or_else(|| XcmError::from(Error::AssetIdConversionFailed))?;
		let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
			.ok_or_else(|| XcmError::from(Error::AssetNotHandled))?
			.saturated_into();
		MultiCurrency::transfer(currency_id, &from_account, &to_account, amount)
			.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

		Ok(asset.clone().into())
	}
}



// VaneMulticurrencyTrait
pub trait VaneMultiCurrency<AccountId> {
	/// The currency identifier.
	type CurrencyId: FullCodec
	+ Eq
	+ PartialEq
	+ Copy
	+ Debug
	+ scale_info::TypeInfo
	+ MaxEncodedLen;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned
	+ FullCodec
	+ Copy
	+ MaybeSerializeDeserialize
	+ Debug
	+ Default
	+ scale_info::TypeInfo
	+ MaxEncodedLen;

	// Public immutables

	/// Existential deposit of `currency_id`.
	fn minimum_balance(currency_id: CurrencyId) -> u128;

	/// The total amount of issuance of `currency_id`.
	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance;

	// The combined balance of `who` under `currency_id`.
	fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	// The free balance of `who` under `currency_id`.
	fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResultWithPostInfo;

	/// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(currency_id, who, value)` (but without the
	/// side-effects) assuming there are no balance changes in the meantime and
	/// only the reserved balance is not taken into account.
	fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible. If this is
	/// less than `amount`, then a non-zero excess value will be returned.
	fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance;
}



parameter_type_with_key! {
		pub ExistentialDeposits: |currency_id: CurrencyId| -> u128 {
			match currency_id {
				CurrencyId::DOT => 1_000_000_000, // DOT

				_ => { unimplemented!()}
			}
		};
}


pub struct MultiCurrencyAsset<T: frame_system::Config>(PhantomData<T>);

impl<T: pallet_assets::Config > VaneMultiCurrency<T::AccountId> for MultiCurrencyAsset<T>{
	type CurrencyId = T::AssetIdParameter;
	type Balance = T::Balance;

	fn minimum_balance(currency_id: CurrencyId) -> u128 {
		ExistentialDeposits::get(&currency_id)
	}


	fn total_issuance(currency_id: Self::CurrencyId) -> T::Balance {
		<pallet_assets::Pallet<T>>::total_issuance(currency_id.into())
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> T::Balance {
		<pallet_assets::Pallet<T>>::total_balance(currency_id.into(),who)
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> T::Balance {
		<pallet_assets::Pallet<T>>::balance(currency_id.into(),who)
	}

	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		let reason = <pallet_assets::Pallet<T>>::can_withdraw(currency_id.into(),who,amount);
		match reason {
			WithdrawConsequence::BalanceLow => { Err(<pallet_assets::Error<T>>::BalanceLow)? }
			WithdrawConsequence::WouldDie => { Err(<pallet_assets::Error<T>>::WouldDie)?}
			WithdrawConsequence::UnknownAsset => {Err(<pallet_assets::Error<T>>::Unknown)?}
			WithdrawConsequence::Underflow => { Err(<pallet_assets::Error<T>>::Unknown)?}
			WithdrawConsequence::Overflow => { Err(<pallet_assets::Error<T>>::Unknown)?}
			WithdrawConsequence::Frozen => { Err(<pallet_assets::Error<T>>::Frozen)?}
			WithdrawConsequence::ReducedToZero(_) => { Err(<pallet_assets::Error<T>>::MinBalanceZero)?}
			WithdrawConsequence::Success => {Ok(())}
		}
	}

	fn transfer(currency_id: Self::CurrencyId, from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResultWithPostInfo {
		let origin = RawOrigin::Signed(from.clone());
		let oo = T::RuntimeOrigin::from(origin);

		// 1. Construct a multi id account
		// send the funds from alice to the multi id account ( Alice, Bob)

		let to_account = T::Lookup::unlookup(to.clone());
		//<pallet_assets::Pallet<T>>::transfer(origin,currency_id,to.into(),amount)
		pallet_assets::Call::<T,()>::transfer {
			id: currency_id,
			target: to_account,
			amount,
		}.dispatch_bypass_filter(oo).into()
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> sp_runtime::DispatchResult {
		let _ = <pallet_assets::Pallet<T>>::deposit(currency_id.into(), who, amount, Precision::Exact)?;
		Ok(())
	}

	fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> sp_runtime::DispatchResult {
		// Check the Fortitude
		let _ = <pallet_assets::Pallet<T>>::withdraw(currency_id.into(), who, amount, Precision::Exact, Preservation::Expendable, Fortitude::Polite)?;
		Ok(())
	}

	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: T::Balance) -> bool {
		if value.is_zero() {
			return true;
		}
		<pallet_assets::Pallet<T>>::balance(currency_id.into(), who) >= value
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> T::Balance {
		todo!() // later on
	}
}


// MultiCurrency Converter
pub struct MultiCurrencyConverter<T: Config>(PhantomData<T>);

impl<T: Config> Convert<MultiAsset, Option<CurrencyId>> for MultiCurrencyConverter<T>{
	fn convert(asset: MultiAsset) -> Option<CurrencyId> {
		match asset.id {
			Concrete(MultiLocation{parents:1,interior:Here}) => Some(CurrencyId::DOT),
			_ => {None}
		}
	}
}

impl<T: Config> Convert<MultiLocation, Option<CurrencyId>> for MultiCurrencyConverter<T>{
	fn convert(location: MultiLocation) -> Option<CurrencyId> {
		if location == MultiLocation::parent() {
			return Some(CurrencyId::DOT)
		}else{
			None
		}
	}
}

impl<T: Config> Convert<CurrencyId, Option<MultiLocation>> for MultiCurrencyConverter<T> {
	fn convert(asset: CurrencyId) -> Option<MultiLocation> {
		if asset == CurrencyId::DOT{
			Some(MultiLocation::parent())
		} else {
			None
		}
	}
}

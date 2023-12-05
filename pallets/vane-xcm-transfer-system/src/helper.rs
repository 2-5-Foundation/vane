#![cfg_attr(not(feature = "std"), no_std)]

use super::pallet::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
pub use utils::*;
use pallet_assets;
use pallet_assets::{AssetDetails, Config};
use sp_core::serde::{Deserialize, Serialize};
use sp_runtime::traits::{AtLeast32BitUnsigned, Convert, Zero, TrailingZeroInput};
use sp_runtime::SaturatedConversion;
use staging_xcm_executor::Assets;
use staging_xcm_executor::traits::{ConvertLocation, Error as XError, MatchesFungible, TransactAsset};
use pallet_xcm::{EnsureXcm, Origin as XcmOrigin};
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
use vane_primitive::GetByKey;

pub use utils::*;

pub mod utils {
	use core::ops::Add;
	use frame_support::parameter_types;
	use sp_core::crypto::Ss58Codec;
use sp_std::ops::{Mul, Sub};
	use frame_system::{AccountInfo, RawOrigin};
	use sp_runtime::traits::{TrailingZeroInput};
    use staging_xcm::{
        v3::{
            Xcm, WeightLimit, NetworkId::Polkadot, Junctions, Junction,
        }
    };
    use sp_std::{vec::Vec,vec};
	use staging_xcm::latest::Parent;
	use staging_xcm::prelude::{AccountId32, All, BuyExecution, DepositAsset, Here, WithdrawAsset};
	use sp_io::hashing::blake2_256;


	use super::*;



	#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct AccountSigners<T: Config> {
		payee: T::AccountId,
		payer: T::AccountId,
	}
	impl<T> AccountSigners<T>
		where
			T: Config,
	{
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
		) -> Self {
			AccountSigners { payee, payer }
		}
		pub(super) fn get_accounts(&self) -> (&T::AccountId, &T::AccountId) {
			(&self.payer,&self.payee)
		}

		// refer here https://doc.rust-lang.org/stable/book/ch06-01-defining-an-enum.html?highlight=enum#enum-values

	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq,MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Token {
		DOT,
		USDT
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq,MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum XcmStatus {
		Sent,
		Completed,
		Tbc
	}

	// Confirmation enum which will be used to confirm the account_ids before dispatching multi-sig
	// Call
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub enum Confirm {
		Payer,
		Payee,
	}

	#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord, codec::MaxEncodedLen, TypeInfo, serde::Serialize,
		serde::Deserialize,)]
		pub enum CurrencyId {
			DOT,
			USDT,
			USDC,
		}

		// impl<T: pallet_assets::Config> From<<T::AssetIdParameter> for CurrencyId {
			
		// };
		
	
	vane_primitive::parameter_type_with_key! {
		pub ExistentialDeposits: |currency_id: CurrencyId| -> u128 {
			match currency_id {
				CurrencyId::DOT => 1_000_000_000, // DOT

				_ => { unimplemented!()}
			}
		};
	}

	parameter_types! {
		pub const MAX_BYTES: u8 = 50;
		pub const MAX_NO_TXNS: u8 = 20;
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug,MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]

	pub struct TxnReceipt<T: Config + pallet_assets::Config> {
		payee: T::AccountId,
		payer: T::AccountId,
		pub multi_id: T::AccountId,
		pub amount: u128,
		pub reference_no: BoundedVec<u8,MAX_BYTES>,
		currency: T::AssetId,
		no_txn: BoundedVec<u128, MAX_NO_TXNS>,
		pub xcm_status: XcmStatus
	}

	impl<T: Config> TxnReceipt<T> {
		pub fn new(
			payee: T::AccountId,
			payer: T::AccountId,
			multi_id: T::AccountId,
			ref_no: BoundedVec<u8,MAX_BYTES>,
			amount: u128,
			txn: u128,
			currency: T::AssetId
		) -> Self {

			let mut no_txn = BoundedVec::new();
			no_txn.to_vec().push(txn);

			Self {
				payee, payer, reference_no: ref_no,
				amount,currency, no_txn,
				xcm_status: XcmStatus::Tbc, multi_id
			}
		}

		pub fn update_txn(&mut self, txn: u128){
			self.no_txn.try_push(txn).unwrap() // Put error handling
		}

		pub fn update_amount(&mut self, amount: u128){
			self.amount += amount
		}
	}


	// Call executed struct information
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct CallExecuted<T: Config> {
		payer: T::AccountId,
		payee: T::AccountId,
		allowed_multi_id: T::AccountId,
		confirmed_multi_id: T::AccountId,
		proof: T::Hash,
		time: BlockNumberFor<T>,
	}

	impl<T> CallExecuted<T>
		where
			T: Config,
	{
		pub(super) fn new(
			payer: T::AccountId,
			payee: T::AccountId,
			allowed_multi_id: T::AccountId,
			confirmed_multi_id: T::AccountId,
			proof: T::Hash,
			time: BlockNumberFor<T>,
		) -> Self {
			CallExecuted { payer, payee, allowed_multi_id, confirmed_multi_id, proof, time }
		}
	}



	// XCM TRAITS & TYPES

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
		type Balance:
		FullCodec
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
		) -> DispatchResult;

		/// Add `amount` to the balance of `who` under `currency_id` and increase
		/// total issuance.
		fn deposit(currency_id: Self::CurrencyId, receiver: &AccountId, amount: Self::Balance) -> DispatchResult;

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

	pub trait UnknownAssetTrait {
		/// Deposit unknown asset.
		fn deposit(asset: &MultiAsset, to: &MultiLocation) -> DispatchResult;
	
		/// Withdraw unknown asset.
		fn withdraw(asset: &MultiAsset, from: &MultiLocation) -> DispatchResult;
	}
	
	const NO_UNKNOWN_ASSET_IMPL: &str = "NoUnknownAssetImpl";
	
	impl UnknownAssetTrait for () {
		fn deposit(_asset: &MultiAsset, _to: &MultiLocation) -> DispatchResult {
			Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
		}
		fn withdraw(_asset: &MultiAsset, _from: &MultiLocation) -> DispatchResult {
			Err(DispatchError::Other(NO_UNKNOWN_ASSET_IMPL))
		}
	}
	
	

	pub struct VaneMultiCurrencyAdapter<
		MultiCurrency,
		UnknownAsset,
		Match,
		AccountId,
		AccountIdConvert,
		CurrencyId,
		CurrencyIdConvert,
		//DepositFailureHandler,
		>(
			PhantomData<(
				MultiCurrency,
				UnknownAsset,
				Match,
				AccountId,
				AccountIdConvert,
				CurrencyId,
				CurrencyIdConvert,
				//DepositFailureHandler,
			)>,
		);

		impl<
			MultiCurrency: VaneMultiCurrency<AccountId, CurrencyId = CurrencyId>,
			UnknownAsset: UnknownAssetTrait,
			Match: MatchesFungible<MultiCurrency::Balance>,
			AccountId: Debug + Clone,
			AccountIdConvert: ConvertLocation<AccountId>,
			CurrencyId: FullCodec + Eq + PartialEq + Copy + Debug,
			CurrencyIdConvert: Convert<MultiAsset, Option<CurrencyId>>,
			//DepositFailureHandler: ,
		> TransactAsset
		for VaneMultiCurrencyAdapter<
			MultiCurrency,
			UnknownAsset,
			Match,
			AccountId,
			AccountIdConvert,
			CurrencyId,
			CurrencyIdConvert,
			//DepositFailureHandler,
		>
		{
			fn deposit_asset(asset: &MultiAsset, location: &MultiLocation, context: &XcmContext) -> staging_xcm::v3::Result {
				let sender = context.origin;
				
				match (
					AccountIdConvert::convert_location(location),
					CurrencyIdConvert::convert(asset.clone()),
					Match::matches_fungible(asset),
				) {
					// known asset
					(Some(receiver), Some(currency_id), Some(amount)) => Ok(MultiCurrency::deposit(currency_id, &receiver, amount).unwrap()),// DepositFailAsset handler
					// unknown asset
					_ => Ok(UnknownAsset::deposit(asset, location).unwrap()), // DepositFailAsset handler
				}
			}

			fn withdraw_asset(
				asset: &MultiAsset,
				location: &MultiLocation,
				_maybe_context: Option<&XcmContext>,
			) -> Result<Assets, XcmError> {
				UnknownAsset::withdraw(asset, location).or_else(|_| {
					let who = AccountIdConvert::convert_location(location)
						.ok_or_else(|| XcmError::from(XError::AccountIdConversionFailed))?;
					let currency_id = CurrencyIdConvert::convert(asset.clone())
						.ok_or_else(|| XcmError::from(XError::AssetIdConversionFailed))?;
					let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
						.ok_or_else(|| XcmError::from(XError::AssetNotHandled))?;
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
					AccountIdConvert::convert_location(from).ok_or_else(|| XcmError::from(XError::AccountIdConversionFailed))?;

				let to_account =
					AccountIdConvert::convert_location(to).ok_or_else(|| XcmError::from(XError::AccountIdConversionFailed))?;

				let currency_id = CurrencyIdConvert::convert(asset.clone())
					.ok_or_else(|| XcmError::from(XError::AssetIdConversionFailed))?;

				let amount: MultiCurrency::Balance = Match::matches_fungible(asset)
					.ok_or_else(|| XcmError::from(XError::AssetNotHandled))?;

				MultiCurrency::transfer(currency_id, &from_account, &to_account, amount)
					.map_err(|e| XcmError::FailedToTransactAsset(e.into()))?;

				Ok(asset.clone().into())
			}
		}



		pub struct MultiCurrencyAsset<T: frame_system::Config>(PhantomData<T>);

		impl<T: pallet_assets::Config + crate::pallet::Config + pallet_balances::Config + pallet_utility::Config > VaneMultiCurrency<T::AccountId> for MultiCurrencyAsset<T>{

			type CurrencyId = T::AssetIdParameter;
			type Balance = <T as pallet_assets::Config>::Balance;

			fn minimum_balance(currency_id: CurrencyId) -> u128 {
				ExistentialDeposits::get(&currency_id)
			}


			fn total_issuance(currency_id: Self::CurrencyId) -> <T as pallet_assets::Config>::Balance{
				<pallet_assets::Pallet<T>>::total_issuance(currency_id.into())
			}

			fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> <T as pallet_assets::Config>::Balance {
				<pallet_assets::Pallet<T>>::total_balance(currency_id.into(),who)
			}

			fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> <T as pallet_assets::Config>::Balance {
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

			fn transfer(currency_id: Self::CurrencyId, from: &T::AccountId, to: &T::AccountId, amount: Self::Balance) -> DispatchResult {
				let origin = RawOrigin::Signed(from.clone());
				let payer_origin = <T as frame_system::Config>::RuntimeOrigin::from(origin);

				// 1. Construct a multi id account
				// send the funds from alice to the multi id account ( Alice, Bob)
				let account_signers = AccountSigners::<T>::new(to.clone(), from.clone());
				let multi_id = Pallet::derive_multi_id(account_signers.clone());

				Pallet::<T>::create_multi_account(multi_id.clone())?;

				// Register AllowedSigners
				let ref_no = Pallet::<T>::derive_reference_no(from.clone(), to.clone(), multi_id.clone());

				AllowedSigners::<T>::insert(&from, ref_no.to_vec(), account_signers);


				//if the multi_id is the same as previous Receipt just increment the total amount and add the txn_no amount

				let amount_u128:u128 = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)?; // handle error

				
				let receipt =
					TxnReceipt::<T>::new(to.clone(), from.clone(),multi_id.clone(), ref_no.clone(), amount_u128.clone(),amount_u128,currency_id.into());


				// Store to each storage item for txntickets
				// Useful for getting reference no for TXN confirmation
				// Start with the payee storage

				PayeeTxnReceipt::<T>::mutate(&to, |p_vec|{
					// Check if the multi_id already exists in the receipts and get its index of the receipt
					let index = p_vec.iter().position(|receipt| receipt.multi_id == multi_id);
					if let Some(idx) = index {
						// Get the receipt
						let receipt = p_vec.get_mut(idx).ok_or(Error::<T>::UnexpectedError).unwrap();
						receipt.update_txn(amount_u128);
						receipt.update_amount(amount_u128)

					}else{
						p_vec.push(receipt.clone())
					}
				});

				// Update for Payer/Sender Txn receipt
				let existing_payer_receipt = PayerTxnReceipt::<T>::get(&from.clone(),&to.clone());

				if let Some(mut receipt) = existing_payer_receipt {

					// vane_payment::PayerTxnReceipt::<T>::mutate(&payer, &payee, |receipt_inner|{
					// 	receipt_inner.clone().unwrap().update_txn(amount)
					// });
					receipt.update_txn(amount_u128);
					receipt.update_amount(amount_u128);

				}else{

					PayerTxnReceipt::<T>::insert(from,to,receipt);
				}

				// Fund the accounts for fees.

				// calculate the fees to fund
				let fees_amount: <T as pallet_balances::Config>::Balance = 100u32.into();

				let fund_payer_call = pallet_balances::Call::<T,()>::transfer_keep_alive { dest: T::Lookup::unlookup(from.clone()), value: fees_amount };
				let fund_payee_call = 	pallet_balances::Call::<T,()>::transfer_keep_alive { dest: T::Lookup::unlookup(to.clone()), value: fees_amount };


				// Chain Soverign Account
				let para_account = ParaAccount::<T>::get().unwrap(); // It should panic and we have to avoid that

				let para_origin = RawOrigin::Signed(para_account.clone());
				let para_account_origin = <T as frame_system::Config>::RuntimeOrigin::from(para_origin);

				
				fund_payer_call.dispatch_bypass_filter(para_account_origin.clone()).unwrap();
				// handle error if it fails it should panic no continuing

				
				fund_payee_call.dispatch_bypass_filter(para_account_origin).unwrap();
				 // handle error if it fails it should panic no continuing

				// Ok(pallet_utility::Call::<T>::batch_all {

				// 	 calls: vec![
				// 		fund_payer_call,
				// 		fund_payee_call
				// 		] 

				// 	}.dispatch_bypass_filter(para_account_origin)?); Evaluate how we cn use pallet utility for batch calls


				let to_account = T::Lookup::unlookup(multi_id.clone());

				
				pallet_assets::Call::<T,()>::transfer {
					id: currency_id,
					target: to_account,
					amount
				}.dispatch_bypass_filter(payer_origin).unwrap(); // Error handling

				// Emit an event
				let time = <frame_system::Pallet<T>>::block_number();

				Pallet::deposit_event( Event::<T>::XcmTokenTransferInitiated { 
					time,
					amount: amount_u128,
					multi_id,
					token: currency_id 
				});

				Ok(())



			}


			fn deposit(currency_id: Self::CurrencyId, receiver: &T::AccountId, amount: Self::Balance) -> Result<(),DispatchError> {
				


				let _ = <pallet_assets::Pallet<T>>::deposit(currency_id.into(), receiver, amount, Precision::Exact)?;
				Ok(())
			}

			fn withdraw(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Result<(),DispatchError> {
				// Check the Fortitude
				let _ = <pallet_assets::Pallet<T>>::withdraw(currency_id.into(), who, amount, Precision::Exact, Preservation::Expendable, Fortitude::Polite)?;
				Ok(())
			}

			fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: <T as pallet_assets::Config>::Balance) -> bool {
				if value.is_zero() {
					return true;
				}
				<pallet_assets::Pallet<T>>::balance(currency_id.into(), who) >= value
			}

			fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> <T as pallet_assets::Config>::Balance {
				todo!() // later on
			}
		}



		// MultiCurrency Converter
		pub struct MultiCurrencyConverter<T: Config>(PhantomData<T>);

		impl<T: Config> Convert<MultiAsset, Option<CurrencyId>> for MultiCurrencyConverter<T>{
			fn convert(asset: MultiAsset) -> Option<CurrencyId> {
				match asset.id {
					Concrete(MultiLocation{parents:1,interior:Here}) => Some(CurrencyId::DOT),
					_ => Some(CurrencyId::DOT)
				}
			}
		}

		impl<T: Config> Convert<MultiLocation, Option<CurrencyId>> for MultiCurrencyConverter<T>{
			fn convert(location: MultiLocation) -> Option<CurrencyId> {
				if location == MultiLocation::parent() {
					return Some(CurrencyId::DOT)
				}else{
					Some(CurrencyId::DOT)
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




	impl<T: Config + crate::pallet::Config> Pallet<T>{


		// pub fn ensure_xcm_signed<OuterOrigin, AccountId>(o: OuterOrigin) -> Result<AccountId, Error<T>>
		// 	where
		// 		OuterOrigin: Into<Result<RawOrigin<AccountId>, OuterOrigin>>,
		// {
		// 	log::info!(
		// 			target:"",
		// 			"{:?}",
		// 			o
		// 		);
		// 	match o.into() {
		// 		Ok(RawOrigin::Signed(t)) => Ok(t),
		// 		_ => Err(Error::<T>::NotTheCaller),
		// 	}
		// }

        // pub fn vane_multisig_record(
        //     payer: T::AccountId,
        //     payee: T::AccountId,
        //     amount: u128,
        //     currency: Token
        // ) -> Result<T::AccountId,Error<T>>{

		// 	// ****** CRUCIAL ******
		// 	// Check the balance receipt in Vane Soverign Account before proceeding


		// 	let accounts = AccountSigners::<T>::new(payee.clone(), payer.clone());
		// 	let multi_id = Self::derive_multi_id(accounts.clone());


		// 	let ref_no = Self::derive_reference_no(payer.clone(), payee.clone(), multi_id.clone());

		// 	AllowedSigners::<T>::insert(&payer, ref_no.to_vec(), accounts);

		// 	//if the multi_id is the same as previous Receipt just increment the total amount and add the txn_no amount

        //     let receipt =
		// 		TxnReceipt::<T>::new(payee.clone(), payer.clone(),multi_id.clone(), ref_no.clone(), amount.clone(),amount,Some(currency));

		// 	// Store to each storage item for txntickets
		// 	// Useful for getting reference no for TXN confirmation
		// 	// Start with the payee storage

		// 	PayeeTxnReceipt::<T>::mutate(&payee, |p_vec|{
		// 		// Check if the multi_id already exists in the receipts and get its index of the receipt
		// 		let index = p_vec.iter().position(|receipt| receipt.multi_id == multi_id);
		// 		if let Some(idx) = index {
		// 			// Get the receipt
		// 			let mut receipt = p_vec.get_mut(idx).ok_or(Error::<T>::UnexpectedError).unwrap();
		// 			receipt.update_txn(amount);
		// 			receipt.update_amount(amount)

		// 		}else{
		// 			p_vec.push(receipt.clone())
		// 		}
		// 	});

		// 	// Update for Payer/Sender Txn receipt
		// 	let existing_payer_receipt = PayerTxnReceipt::<T>::get(&payer,&payee);

		// 	if let Some(mut receipt) = existing_payer_receipt {

		// 		// vane_payment::PayerTxnReceipt::<T>::mutate(&payer, &payee, |receipt_inner|{
		// 		// 	receipt_inner.clone().unwrap().update_txn(amount)
		// 		// });
		// 		receipt.update_txn(amount);
		// 		receipt.update_amount(amount);

		// 	}else{

		// 		PayerTxnReceipt::<T>::insert(&payer,&payee,receipt);
		// 	}


		// 	Self::create_multi_account(multi_id.clone()).map_err(|_| Error::<T>::UnexpectedError)?;

		// 	let time = <frame_system::Pallet<T>>::block_number();

		// 	Self::deposit_event(Event::MultisigAccountCreated {
		// 		id: multi_id.clone(),
		// 		time,
		// 	});

        //     Ok(multi_id)
        // }


        pub fn vane_xcm_transfer_dot(
            amount: u128,
            multi_id: AccountIdLookupOf<T>, // Multi Id Account
			multi_id_acc: T::AccountId, // Just for types difference usage but this and multi_id are sme value
			asset_id: T::AssetIdParameter
		) -> DispatchResult {


			// Deposit Amount to MultiSig Account
			let issuer = ParaAccount::<T>::get().unwrap();

			let balance: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			// Check if the multi_id account does contain the tokens
			let to_check_balance:<T as pallet_assets::Config>::Balance = 0u128.try_into().map_err(|_| Error::<T>::UnexpectedError)?;

			if <pallet_assets::Pallet<T>>::balance(asset_id.into(),multi_id_acc.clone()) != to_check_balance {

				let current_balance = <pallet_assets::Pallet<T>>::balance(asset_id.into(),multi_id_acc);
				let to_mint = current_balance.add(balance.try_into().expect("Failed to add balances in pallet  vane_xcm"));

				<pallet_assets::Pallet<T>>::mint(
					RawOrigin::Signed(issuer).into(),
					asset_id,
					multi_id.clone(),
					to_mint // handle this error
				)?;

			}else{
				<pallet_assets::Pallet<T>>::mint(
					RawOrigin::Signed(issuer).into(),
					asset_id,
					multi_id.clone(),
					balance // handle this error
				)?;
			}

			let time = <frame_system::Pallet<T>>::block_number();
			// Event
			Self::deposit_event(Event::DotXcmTransferInitiated {
				time,
				amount,
				multi_id,
			});

            Ok(())
        }



        pub fn vane_xcm_confirm_transfer_dot(
			payer: T::AccountId,
            payee: T::AccountId,
			multi_id: AccountIdLookupOf<T>,
            amount: u128,
			asset_id: T::AssetIdParameter

        ) -> DispatchResult{

			let issuer = ParaAccount::<T>::get().unwrap();

			// Burn the asset in the multi_id account
			let amount_type: <T as pallet_assets::Config>::Balance = amount.try_into().map_err(|_| Error::<T>::UnexpectedError)? ;

			<pallet_assets::Pallet<T>>::burn(
				RawOrigin::Signed(issuer).into(),
				asset_id,
				multi_id.clone(),
				amount_type
			)?;

			// Change the status in the payer receipt


			// Send XCM instruction to send funds from Parachain sovereign account to payee acount
			//let payee_id:[u8;32] = payee.encode().try_into().unwrap();

			// Take 1 amount

			let new_amount = amount.sub(10_000_000_000);

			let message = Xcm::<()>(vec![
				// Transfer equivalent funds from Sovereign Account to payee account
				// TransferAsset {
				// 	assets: (Here, amount).into(), // We must have a function to calculate fees
				// 	beneficiary: (AccountId32 {network: None, id: payee.encode().try_into().unwrap()}).into(),
				// }
				WithdrawAsset((Here,new_amount).into()),
				BuyExecution { fees: (Here,new_amount).into(), weight_limit: WeightLimit::Unlimited },
				DepositAsset { assets: All.into(), beneficiary:  (AccountId32 {network: None, id: payee.encode().try_into().unwrap()}).into() }
			]);

			<pallet_xcm::Pallet<T>>::send_xcm(Here,Parent,message).map_err(|_| Error::<T>::ErrorSendingXcm)?;

			// Change the status in the payee receipt


            // Event
			Self::deposit_event(
				Event::MessageTransferedToPolkadot
			);

            Ok(())
        }


        pub fn vane_xcm_transfer_assethub_dot() -> DispatchResult{
            Ok(())
        }

        pub fn vane_xcm_confirm_transfer_assethub_dot() -> DispatchResult{

            Ok(())
        }


		// Util functions

		pub fn derive_reference_no(
			payer: T::AccountId,
			payee: T::AccountId,
			multi_id: T::AccountId,
		) -> BoundedVec<u8,MAX_BYTES> {
			let mut buffer = Vec::new();
			buffer.append(&mut payer.using_encoded(blake2_256).to_vec());
			buffer.append(&mut payee.using_encoded(blake2_256).to_vec());
			buffer.append(&mut multi_id.using_encoded(blake2_256).to_vec());

			let reference = blake2_256(&buffer[..]);
			return reference[20..26].to_vec().try_into().unwrap(); // Proper error handling should be done
		}


		pub  fn derive_multi_id(account_object: AccountSigners<T>) -> T::AccountId {

			let (acc1, acc2) = account_object.get_accounts();

			let entropy =  (b"vane/salt", acc1, acc2).using_encoded(blake2_256);

			let multi_account = Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed");

			multi_account
		}

		pub fn create_multi_account(multi_id: T::AccountId) -> DispatchResult {
			let account_info = AccountInfo::<T::Nonce, T::AccountData> { ..Default::default() };

			// Ensure the multi_id account is not yet registered in the storage
			if <frame_system::Pallet<T>>::account_exists(&multi_id) {
				return Ok(());
			} else {
				// Register to frame_system Account Storage item;
				<frame_system::Account<T>>::set(multi_id, account_info);
				Ok(())
			}
		}

		
	}


}

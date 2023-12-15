// Copyright (C) Moondance Labs Ltd.
// This file is part of Tanssi.

// Tanssi is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Tanssi is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Tanssi.  If not, see <http://www.gnu.org/licenses/>
use sp_std::marker::PhantomData;

use {
    super::{
        AccountId, AllPalletsWithSystem, Balances, ParachainInfo, ParachainSystem, PolkadotXcm,
        Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, WeightToFee, XcmpQueue,
    },
    cumulus_primitives_core::ParaId,
    frame_support::{
        parameter_types,
        traits::{Everything, Nothing, PalletInfoAccess},
        weights::Weight,match_types
    },
    frame_system::EnsureRoot,
    pallet_xcm::XcmPassthrough,
    polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery,
    sp_core::ConstU32,
    staging_xcm::latest::prelude::*,
    staging_xcm_builder::{
        AccountId32Aliases, AllowKnownQueryResponses, AllowSubscriptionsFrom,
        AllowTopLevelPaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds,
        IsConcrete, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
        SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
        SovereignSignedViaLocation, TakeWeightCredit, UsingComponents, WithComputedOrigin,
    },
    staging_xcm_executor::XcmExecutor,
};


use polkadot_parachain_primitives::primitives::Sibling;
use staging_xcm_builder::{AllowUnpaidExecutionFrom, AliasForeignAccountId32, WithUniqueTopic};
use vane_xcm_transfer_system::{CurrencyId, MultiCurrencyAsset, MultiCurrencyConverter};
//use orml_xcm_support::IsNativeConcrete;
use sp_runtime::traits::{CheckedConversion, Convert};
use staging_xcm_executor::traits::MatchesFungible;

// parameter_types! {
//     // Self Reserve location, defines the multilocation identifiying the self-reserve currency
//     // This is used to match it also against our Balances pallet when we receive such
//     // a MultiLocation: (Self Balances pallet index)
//     // We use the RELATIVE multilocation
//     pub SelfReserve: MultiLocation = MultiLocation {
//         parents:0,
//         interior: Junctions::X1(
//             PalletInstance(<Balances as PalletInfoAccess>::index() as u8)
//         )
//     };

//     // One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
//     pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);

//     // TODO: revisit
//     pub const RelayNetwork: NetworkId = NetworkId::Westend;

//     // The relay chain Origin type
//     pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();

//     pub const MaxAssetsIntoHolding: u32 = 64;

//     /// Maximum number of instructions in a single XCM fragment. A sanity check against
//     /// weight caculations getting too crazy.
//     pub MaxInstructions: u32 = 100;

//     // The universal location within the global consensus system
//     pub UniversalLocation: InteriorMultiLocation =
//     X2(GlobalConsensus(RelayNetwork::get()), Parachain(ParachainInfo::parachain_id().into()));
// }

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
    pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

// pub type XcmBarrier = (
//     // Weight that is paid for may be consumed.
//     TakeWeightCredit,
//     // Expected responses are OK.
//     AllowKnownQueryResponses<PolkadotXcm>,
//     WithComputedOrigin<
//         (
//             // If the message is one that immediately attemps to pay for execution, then allow it.
//             AllowTopLevelPaidExecutionFrom<Everything>,
//             // Subscriptions for version tracking are OK.
//             AllowSubscriptionsFrom<Everything>,
//         ),
//         UniversalLocation,
//         ConstU32<8>,
//     >,
// );

// /// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
// /// when determining ownership of accounts for asset transacting and when attempting to use XCM
// /// `Transact` in order to determine the dispatch Origin.
// pub type LocationToAccountId = (
//     // The parent (Relay-chain) origin converts to the default `AccountId`.
//     ParentIsPreset<AccountId>,
//     // Sibling parachain origins convert to AccountId via the `ParaId::into`.
//     SiblingParachainConvertsVia<polkadot_parachain_primitives::primitives::Sibling, AccountId>,
//     // If we receive a MultiLocation of type AccountKey20, just generate a native account
//     AccountId32Aliases<RelayNetwork, AccountId>,
//     // Generate remote accounts according to polkadot standards
//     staging_xcm_builder::HashedDescription<
//         AccountId,
//         staging_xcm_builder::DescribeFamily<staging_xcm_builder::DescribeAllTerminal>,
//     >,
// );

// /// Local origins on this chain are allowed to dispatch XCM sends/executions.
// pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

// /// Means for transacting the native currency on this chain.
// pub type CurrencyTransactor = CurrencyAdapter<
//     // Use this currency:
//     Balances,
//     // Use this currency when it is a fungible asset matching the given location or name:
//     IsConcrete<SelfReserve>,
//     // Convert an XCM MultiLocation into a local account id:
//     LocationToAccountId,
//     // Our chain's account ID type (we can't get away without mentioning it explicitly):
//     AccountId,
//     // We don't track any teleports of `Balances`.
//     (),
// >;

// /// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
// /// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
// /// biases the kind of local `Origin` it will become.
// pub type XcmOriginToTransactDispatchOrigin = (
//     // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
//     // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
//     // foreign chains who want to have a local sovereign account on this chain which they control.
//     SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
//     // Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
//     // recognised.
//     RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
//     // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
//     // recognised.
//     SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
//     // Native signed account converter; this just converts an `AccountId32` origin into a normal
//     // `RuntimeOrigin::Signed` origin of the same 32-byte value.
//     SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
//     // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
//     XcmPassthrough<RuntimeOrigin>,
// );

// /// Means for transacting assets on this chain.
// pub type AssetTransactors = CurrencyTransactor;
// pub type XcmWeigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;

// /// The means for routing XCM messages which are not for local execution into the right message
// /// queues.
// pub type XcmRouter = (
//     // Two routers - use UMP to communicate with the relay chain:
//     cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
//     // ..and XCMP to communicate with the sibling chains.
//     XcmpQueue,
// );




// vane config

parameter_types! {
	pub const RelayLocation: MultiLocation = MultiLocation::parent();
	pub const RelayNetwork: Option<NetworkId> = None;
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
	// The parent (Relay-chain) origin converts to the parent `AccountId`.
	ParentIsPreset<AccountId>,
	// Sibling parachain origins convert to AccountId via the `ParaId::into`.
	SiblingParachainConvertsVia<Sibling, AccountId>,
	// Straight up local `AccountId32` origins just alias directly to `AccountId`.
	AccountId32Aliases<RelayNetwork, AccountId>,
);

pub struct IsNativeConcrete<CurrencyId, CurrencyIdConvert>(PhantomData<(CurrencyId, CurrencyIdConvert)>);
impl<CurrencyId, CurrencyIdConvert, Amount> MatchesFungible<Amount> for IsNativeConcrete<CurrencyId, CurrencyIdConvert>
	where
		CurrencyIdConvert: Convert<MultiLocation, Option<CurrencyId>>,
		Amount: TryFrom<u128>,
{
	fn matches_fungible(a: &MultiAsset) -> Option<Amount> {
		if let (Fungible(ref amount), Concrete(ref location)) = (&a.fun, &a.id) {
			if CurrencyIdConvert::convert(*location).is_some() {
				return CheckedConversion::checked_from(*amount);
			}
		}
		None
	}
}

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor =  vane_xcm_transfer_system::VaneMultiCurrencyAdapter<
	MultiCurrencyAsset<Runtime>,
	(), // handler for unknown assets
	IsNativeConcrete<CurrencyId, MultiCurrencyConverter<Runtime>>,
	AccountId,
	LocationToAccountId,
	CurrencyId,
	MultiCurrencyConverter<Runtime>,
	// HandlingFailedDeposits
>;

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location; will convert to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter; this just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::Signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

parameter_types! {
	// One XCM operation is 1_000_000_000 weight - almost certainly a conservative estimate.
	pub UnitWeightCost: Weight = Weight::from_parts(1_000_000_000, 64 * 1024);
	pub const MaxInstructions: u32 = 100;
	pub const MaxAssetsIntoHolding: u32 = 64;
}

match_types! {
	pub type ParentOrParentsExecutivePlurality: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here } |
		MultiLocation { parents: 1, interior: X1(Plurality { id: BodyId::Executive, .. }) }
	};
}

// pub type Barrier = TrailingSetTopicAsId<
// 	DenyThenTry<
// 		DenyReserveTransferToRelayChain,
// 		(
// 			TakeWeightCredit,
// 			WithComputedOrigin<
// 				(
// 					AllowTopLevelPaidExecutionFrom<Everything>,
// 					AllowExplicitUnpaidExecutionFrom<ParentOrParentsExecutivePlurality>,
// 					// ^^^ Parent and its exec plurality get free execution
// 					//AllowUnpaidExecutionFrom<Parent>
// 				),
// 				UniversalLocation,
// 				ConstU32<8>,
// 			>,
// 		),
// 	>,
// >;

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

match_types! {
	pub type SiblingPrefix: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: X1(Parachain(_)) }
	};
	pub type ChildPrefix: impl Contains<MultiLocation> = {
		MultiLocation { parents: 0, interior: X1(Parachain(_)) }
	};
	pub type ParentPrefix: impl Contains<MultiLocation> = {
		MultiLocation { parents: 1, interior: Here }
	};
}


pub type XcmOriginToCallOrigin = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetwork, RuntimeOrigin>,
	XcmPassthrough<RuntimeOrigin>,
);

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = WithUniqueTopic<(
	// Two routers - use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, (), ()>,
	// ..and XCMP to communicate with the sibling chains.
	XcmpQueue,
)>;



pub struct XcmConfig;
impl staging_xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = XcmOriginToCallOrigin;
	type IsReserve = vane_xcm_transfer_system::VaneDerivedAssets; // Custom Asset matcher
	type IsTeleporter = ();
	type Aliasers = AliasForeignAccountId32<ParentPrefix>;
	// Teleporting is disabled.
	type UniversalLocation = UniversalLocation;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type Trader =
		UsingComponents<WeightToFee, RelayLocation, AccountId, Balances, ()>;
	type ResponseHandler = PolkadotXcm;
	type AssetTrap = PolkadotXcm;
	type AssetLocker = ();
	type AssetExchanger = ();
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = RuntimeCall;
	type SafeCallFilter = Everything;
}

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = ();
    type SovereignAccountOf = LocationToAccountId;
    type MaxLockers = ConstU32<8>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();
    // TODO pallet-xcm weights
    type WeightInfo = pallet_xcm::TestWeightInfo;
    #[cfg(feature = "runtime-benchmarks")]
    type ReachableDest = ReachableDest;
    type AdminOrigin = EnsureRoot<AccountId>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ChannelInfo = ParachainSystem;
    type VersionWrapper = PolkadotXcm;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type ControllerOrigin = EnsureRoot<AccountId>;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Self>;
    type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
}

impl cumulus_pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}

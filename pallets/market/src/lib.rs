#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, PalletId};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, AtLeast32BitUnsigned},
    RuntimeDebug,
};
use sp_std::prelude::*;
use sugarfunge_primitives::Balance;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type MarketId = u32;
pub type MarketRateId = u32;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AmountOp {
    Equal(Balance),
    LessThan(Balance),
    LessEqualThan(Balance),
    GreaterThan(Balance),
    GreaterEqualThan(Balance),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RateAmount {
    Credit(Balance),
    Debit(Balance),
    Mint(Balance),
    Burn(Balance),
    Has(AmountOp),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RateTarget<AccountId> {
    Account(AccountId),
    Creator,
    Buyer,
    Seller,
    Market,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetRate<AccountId, ClassId, AssetId> {
    class_id: ClassId,
    asset_id: AssetId,
    amount: RateAmount,
    target: RateTarget<AccountId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct MarketRate<AccountId, ClassId, AssetId> {
    goods: Vec<AssetRate<AccountId, ClassId, AssetId>>,
    price: Vec<AssetRate<AccountId, ClassId, AssetId>>,
    metadata: Vec<u8>,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type MarketId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>;

        type MarketRateId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn markets)]
    pub(super) type Markets<T: Config> = StorageMap<_, Blake2_128, MarketId, Market<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn market_rates)]
    pub(super) type MarketRates<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, MarketId>,
            NMapKey<Blake2_128Concat, MarketRateId>,
        ),
        MarketRate<T::AccountId, T::ClassId, T::AssetId>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created {
            market_id: MarketId,
            who: T::AccountId,
        },
        RateCreated {
            market_id: MarketId,
            market_rate_id: MarketRateId,
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        InvalidMarket,
        InvalidMarketRate,
        InvalidMarketOwner,
        NotAuthorizedToMintAsset,
        MarketExists,
        MarketRateExists,
        InvalidAsset,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market<AccountId> {
    /// The owner of the market
    pub owner: AccountId,
    /// The fund account of the market
    pub vault: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_create_market(owner: &T::AccountId, market_id: MarketId) -> DispatchResult {
        ensure!(
            !Markets::<T>::contains_key(market_id),
            Error::<T>::MarketExists
        );

        let vault: T::AccountId = <T as Config>::PalletId::get().into_sub_account(market_id);

        Markets::<T>::insert(
            market_id,
            Market {
                owner: owner.clone(),
                vault,
            },
        );

        Self::deposit_event(Event::Created {
            market_id,
            who: owner.clone(),
        });

        Ok(())
    }

    pub fn do_create_market_rate(
        who: &T::AccountId,
        market_id: MarketId,
        market_rate_id: MarketRateId,
        market_rate: &MarketRate<T::AccountId, T::ClassId, T::AssetId>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        MarketRates::<T>::insert((market_id, market_rate_id), market_rate);

        Self::deposit_event(Event::RateCreated {
            market_id,
            market_rate_id,
            who: who.clone(),
        });

        Ok(())
    }

    pub fn do_deposit_assets(
        who: &T::AccountId,
        market_id: MarketId,
        market_rate_id: MarketRateId,
        _amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let mut asset_rates: Vec<AssetRate<T::AccountId, T::ClassId, T::AssetId>> =
            Vec::with_capacity(market_rate.goods.len() + market_rate.price.len());
        asset_rates.extend(market_rate.goods.clone());
        asset_rates.extend(market_rate.price.clone());

        // Ensure assets exists
        for asset_rate in asset_rates {
            ensure!(
                sugarfunge_asset::Pallet::<T>::asset_exists(
                    asset_rate.class_id,
                    asset_rate.asset_id
                ),
                Error::<T>::InvalidAsset
            );
        }

        Ok(().into())
    }
}

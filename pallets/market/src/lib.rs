#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{dispatch::DispatchResult, ensure, traits::Get, PalletId};
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
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
        InvalidMarketId,
        InvalidMarketRateId,
        MarketExists,
        MarketRateExists,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market<AccountId> {
    /// The creator of the market
    pub creator: AccountId,
    /// The fund account of the market
    pub vault: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_create_market(creator: &T::AccountId, market_id: MarketId) -> DispatchResult {
        ensure!(
            !Markets::<T>::contains_key(market_id),
            Error::<T>::MarketExists
        );

        let vault: T::AccountId = <T as Config>::PalletId::get().into_sub_account(market_id);

        Markets::<T>::insert(
            market_id,
            Market {
                creator: creator.clone(),
                vault,
            },
        );

        Self::deposit_event(Event::Created {
            market_id,
            who: creator.clone(),
        });

        Ok(())
    }

    pub fn do_create_market_rate(
        creator: &T::AccountId,
        market_id: MarketId,
        market_rate_id: MarketRateId,
        market_rate: &MarketRate<T::AccountId, T::ClassId, T::AssetId>,
    ) -> DispatchResult {
        ensure!(
            Markets::<T>::contains_key(market_id),
            Error::<T>::InvalidMarketId
        );

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        MarketRates::<T>::insert((market_id, market_rate_id), market_rate);

        Self::deposit_event(Event::RateCreated {
            market_id,
            market_rate_id,
            who: creator.clone(),
        });

        Ok(())
    }
}

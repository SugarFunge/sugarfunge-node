#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    PalletId,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, AtLeast32BitUnsigned},
    RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
use sugarfunge_primitives::{Amount, Balance};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum AmountOp {
    Equal(Amount),
    LessThan(Amount),
    LessEqualThan(Amount),
    GreaterThan(Amount),
    GreaterEqualThan(Amount),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RateAmount {
    Credit(Amount),
    Debit(Amount),
    Mint(Amount),
    Burn(Amount),
    Has(AmountOp),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum RateTarget<AccountId> {
    Account(AccountId),
    Buyer,
    Seller,
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

// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
// pub struct AssetExchange<AccountId, ClassId, AssetId> {
//     class_id: ClassId,
//     asset_id: AssetId,
//     debit: Amount,
//     credit: Amount,
//     account: AccountId,
// }

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ExchangeBalance<AccountId, ClassId, AssetId> {
    goods: Vec<AssetRate<AccountId, ClassId, AssetId>>,
    price: Vec<AssetRate<AccountId, ClassId, AssetId>>,
}

type TransactionBalances<AccountId, ClassId, AssetId> =
    BTreeMap<(AccountId, ClassId, AssetId), Amount>;

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
    pub(super) type Markets<T: Config> =
        StorageMap<_, Blake2_128, T::MarketId, Market<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn market_rates)]
    pub(super) type MarketRates<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::MarketId>,
            NMapKey<Blake2_128Concat, T::MarketRateId>,
        ),
        MarketRate<T::AccountId, T::ClassId, T::AssetId>,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created {
            market_id: T::MarketId,
            who: T::AccountId,
        },
        RateCreated {
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            who: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Overflow,
        InsufficientAmount,
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
    pub fn do_create_market(owner: &T::AccountId, market_id: T::MarketId) -> DispatchResult {
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
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        market_rate: &MarketRate<T::AccountId, T::ClassId, T::AssetId>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        // TODO: ensure all amounts are positive

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
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        Ok(().into())
    }

    pub fn add_balance(
        balances: &mut TransactionBalances<T::AccountId, T::ClassId, T::AssetId>,
        account: &T::AccountId,
        class_id: T::ClassId,
        asset_id: T::AssetId,
        amount: Amount,
    ) -> Result<Amount, DispatchError> {
        let amount = if let Some(balance) = balances.get_mut(&(account.clone(), class_id, asset_id))
        {
            *balance = *balance + amount;
            *balance
        } else {
            let balance: i128 =
                sugarfunge_asset::Pallet::<T>::balance_of(account, class_id, asset_id)
                    .try_into()
                    .map_err(|_| Error::<T>::Overflow)?;
            let balance = balance.checked_add(amount).ok_or(Error::<T>::Overflow)?;
            balances.insert((account.clone(), class_id, asset_id), balance);
            balance
        };

        Ok(amount)
    }

    pub fn do_compute_transactions(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> Result<ExchangeBalance<T::AccountId, T::ClassId, T::AssetId>, DispatchError> {
        ensure!(amount > 0, Error::<T>::InsufficientAmount);

        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        let mut exchange_balance = ExchangeBalance {
            goods: vec![],
            price: vec![],
        };

        let get_target_account = |target| match target {
            RateTarget::Account(account) => account,
            RateTarget::Buyer => buyer.clone(),
            RateTarget::Seller => market.owner.clone(),
        };

        let balances: TransactionBalances<T::AccountId, T::ClassId, T::AssetId> = BTreeMap::new();

        let total_amount: i128 = amount.try_into().map_err(|_| Error::<T>::Overflow)?;

        for asset_rate in market_rate.goods {
            let target = get_target_account(asset_rate.target);
            match asset_rate.amount {
                RateAmount::Credit(amount) => {
                    let amount = amount
                        .checked_mul(total_amount)
                        .ok_or(Error::<T>::Overflow)?;
                    let balance: i128 = sugarfunge_asset::Pallet::<T>::balance_of(
                        &market.owner,
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    )
                    .try_into()
                    .map_err(|_| Error::<T>::Overflow)?;
                    let diff = (balance as i128)
                        .checked_sub(amount as i128)
                        .ok_or(Error::<T>::Overflow)?;
                    exchange_balance.goods.push(AssetRate {
                        class_id: asset_rate.class_id,
                        asset_id: asset_rate.asset_id,
                        amount: RateAmount::Credit(if diff < 0 { diff } else { amount }),
                        target: RateTarget::Account(target),
                    });
                }
                _ => (),
            }
        }

        // ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);
        Ok(exchange_balance)
    }
}

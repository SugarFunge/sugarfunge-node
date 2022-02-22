#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    BoundedVec, PalletId,
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

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum AmountOp {
    Equal,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum RateAction {
    Transfer,
    Mint,
    Burn,
    Has(AmountOp),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub enum RateAccount<AccountId> {
    Market,
    Account(AccountId),
    Buyer,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
pub struct AssetRate<AccountId, ClassId, AssetId> {
    class_id: ClassId,
    asset_id: AssetId,
    action: RateAction,
    amount: Amount,
    from: RateAccount<AccountId>,
    to: RateAccount<AccountId>,
}

pub type Rates<T> = BoundedVec<
    AssetRate<
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    >,
    <T as Config>::MaxRates,
>;

pub type RateBalances<AccountId, ClassId, AssetId> =
    BTreeMap<AssetRate<AccountId, ClassId, AssetId>, Amount>;

type TransactionBalances<AccountId, ClassId, AssetId> =
    BTreeMap<(RateAccount<AccountId>, ClassId, AssetId), Amount>;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

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

        /// Max number of rates per market_rate
        #[pallet::constant]
        type MaxRates: Get<u32>;

        /// Max metadata size
        #[pallet::constant]
        type MaxMetadata: Get<u32>;
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
        Rates<T>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn market_rates_metadata)]
    pub(super) type MarketRatesMetadata<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::MarketId>,
            NMapKey<Blake2_128Concat, T::MarketRateId>,
        ),
        BoundedVec<u8, <T as Config>::MaxMetadata>,
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
        Deposit {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            // balances: RateBalances<T::AccountId, T::ClassId, T::AssetId>,
        },
        Exchanged {
            buyer: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            // balances: RateBalances<T::AccountId, T::ClassId, T::AssetId>,
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
        InvalidAssetRate,
        InvalidRateAccount,
        InvalidRateAmount,
        InvalidBurnPrice,
        InvalidBurnBalance,
        InvalidTransferPrice,
        InvalidTransferBalance,
        InvalidBuyer,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_market(
            origin: OriginFor<T>,
            market_id: T::MarketId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_market(&who, market_id)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_market_rate(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            rates: Rates<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_market_rate(&who, market_id, market_rate_id, &rates)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn deposit_assets(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_deposit_assets(&who, market_id, market_rate_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn exchange_assets(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_exchange_assets(&who, market_id, market_rate_id, amount)?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Market<AccountId> {
    /// The owner of the market
    pub owner: AccountId,
    /// The fund account of the market
    pub vault: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_create_market(who: &T::AccountId, market_id: T::MarketId) -> DispatchResult {
        ensure!(
            !Markets::<T>::contains_key(market_id),
            Error::<T>::MarketExists
        );

        let vault: T::AccountId = <T as Config>::PalletId::get().into_sub_account(market_id);

        Markets::<T>::insert(
            market_id,
            Market {
                owner: who.clone(),
                vault,
            },
        );

        Self::deposit_event(Event::Created {
            market_id,
            who: who.clone(),
        });

        Ok(())
    }

    pub fn do_create_market_rate(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        rates: &Rates<T>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        // Ensure rates are valid
        for asset_rate in rates.iter() {
            ensure!(asset_rate.amount >= 0, Error::<T>::InvalidRateAmount);
        }

        MarketRates::<T>::insert((market_id, market_rate_id), rates);

        Self::deposit_event(Event::RateCreated {
            market_id,
            market_rate_id,
            who: who.clone(),
        });

        Ok(())
    }

    pub fn do_compute_deposit(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> Result<(bool, RateBalances<T::AccountId, T::ClassId, T::AssetId>), DispatchError> {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let mut deposit_balances = BTreeMap::new();

        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalances<T::AccountId, T::ClassId, T::AssetId> =
            BTreeMap::new();

        let mut prices: TransactionBalances<T::AccountId, T::ClassId, T::AssetId> = BTreeMap::new();

        let total_amount: i128 = amount.try_into().map_err(|_| Error::<T>::Overflow)?;

        let asset_rates = rates.into_iter().filter(|asset_rate| {
            asset_rate.from == RateAccount::Market
                && (asset_rate.action == RateAction::Transfer
                    || asset_rate.action == RateAction::Burn)
        });

        let asset_rates: Vec<AssetRate<T::AccountId, T::ClassId, T::AssetId>> =
            asset_rates.collect();

        for asset_rate in &asset_rates {
            let balance: i128 = sugarfunge_asset::Pallet::<T>::balance_of(
                &market.owner,
                asset_rate.class_id,
                asset_rate.asset_id,
            )
            .try_into()
            .map_err(|_| Error::<T>::Overflow)?;
            balances.insert(
                (
                    asset_rate.from.clone(),
                    asset_rate.class_id,
                    asset_rate.asset_id,
                ),
                balance,
            );
            let amount = asset_rate
                .amount
                .checked_mul(total_amount)
                .ok_or(Error::<T>::Overflow)?;
            if let Some(price) = prices.get_mut(&(
                asset_rate.from.clone(),
                asset_rate.class_id,
                asset_rate.asset_id,
            )) {
                *price = price.checked_add(amount).ok_or(Error::<T>::Overflow)?;
            } else {
                prices.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    amount,
                );
            }
        }

        // RateAction::Burn - Compute total burns

        for asset_rate in &asset_rates {
            if let RateAction::Burn = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnBalance)?;
                *balance = balance.checked_sub(*price).ok_or(Error::<T>::Overflow)?;
                if *balance < 0 {
                    deposit_balances.insert(asset_rate.clone(), *balance);
                } else {
                    deposit_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Transfer - Compute total transfers

        for asset_rate in &asset_rates {
            if let RateAction::Transfer = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferBalance)?;
                *balance = balance.checked_sub(*price).ok_or(Error::<T>::Overflow)?;
                if *balance < 0 {
                    deposit_balances.insert(asset_rate.clone(), *balance);
                } else {
                    deposit_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        let mut can_do_deposit = true;

        for (_, deposit_balance) in &deposit_balances {
            if *deposit_balance < 0 {
                can_do_deposit = false;
                break;
            }
        }

        Ok((can_do_deposit, deposit_balances))
    }

    pub fn do_deposit_assets(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let (can_do_deposit, deposit_balances) =
            Self::do_compute_deposit(who, market_id, market_rate_id, amount)?;

        ensure!(can_do_deposit, Error::<T>::InsufficientAmount);

        for (asset_rate, amount) in &deposit_balances {
            let amount: u128 = (*amount).try_into().map_err(|_| Error::<T>::Overflow)?;
            sugarfunge_asset::Pallet::<T>::do_transfer_from(
                &market.owner,
                &market.owner,
                &market.vault,
                asset_rate.class_id,
                asset_rate.asset_id,
                amount,
            )?
        }

        Self::deposit_event(Event::Deposit {
            who: who.clone(),
            market_id,
            market_rate_id,
            amount,
            // balances: deposit_balances,
        });

        Ok(().into())
    }

    pub fn do_compute_exchange(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> Result<(bool, RateBalances<T::AccountId, T::ClassId, T::AssetId>), DispatchError> {
        ensure!(amount > 0, Error::<T>::InsufficientAmount);

        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        let mut exchange_balances = BTreeMap::new();

        let mut can_do_exchange = true;

        // RateAction::Has - Prove parties possess non-transferable assets

        for asset_rate in rates.iter() {
            if let RateAction::Has(op) = &asset_rate.action {
                let target_account = match &asset_rate.from {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                let balance: i128 = sugarfunge_asset::Pallet::<T>::balance_of(
                    target_account,
                    asset_rate.class_id,
                    asset_rate.asset_id,
                )
                .try_into()
                .map_err(|_| Error::<T>::Overflow)?;
                let amount = match op {
                    AmountOp::Equal => {
                        if balance == asset_rate.amount {
                            asset_rate.amount
                        } else {
                            can_do_exchange = false;
                            balance - asset_rate.amount
                        }
                    }
                    AmountOp::GreaterEqualThan => {
                        if balance >= asset_rate.amount {
                            asset_rate.amount
                        } else {
                            can_do_exchange = false;
                            balance - asset_rate.amount
                        }
                    }
                    AmountOp::GreaterThan => {
                        if balance > asset_rate.amount {
                            asset_rate.amount
                        } else {
                            can_do_exchange = false;
                            balance - asset_rate.amount
                        }
                    }
                    AmountOp::LessEqualThan => {
                        if balance <= asset_rate.amount {
                            asset_rate.amount
                        } else {
                            can_do_exchange = false;
                            asset_rate.amount - balance
                        }
                    }
                    AmountOp::LessThan => {
                        if balance < asset_rate.amount {
                            asset_rate.amount
                        } else {
                            can_do_exchange = false;
                            asset_rate.amount - balance
                        }
                    }
                };
                exchange_balances.insert(asset_rate.clone(), amount);
            }
        }

        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalances<T::AccountId, T::ClassId, T::AssetId> =
            BTreeMap::new();

        let mut prices: TransactionBalances<T::AccountId, T::ClassId, T::AssetId> = BTreeMap::new();

        let total_amount: i128 = amount.try_into().map_err(|_| Error::<T>::Overflow)?;

        for asset_rate in rates.iter() {
            let balance = match &asset_rate.action {
                RateAction::Transfer | RateAction::Burn => {
                    let target_account = match &asset_rate.from {
                        RateAccount::Account(account) => account,
                        RateAccount::Buyer => buyer,
                        RateAccount::Market => &market.vault,
                    };
                    let balance: i128 = sugarfunge_asset::Pallet::<T>::balance_of(
                        target_account,
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    )
                    .try_into()
                    .map_err(|_| Error::<T>::Overflow)?;
                    Some(balance)
                }
                _ => None,
            };
            if let Some(balance) = balance {
                balances.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    balance,
                );
            }
            let amount = asset_rate
                .amount
                .checked_mul(total_amount)
                .ok_or(Error::<T>::Overflow)?;
            if let Some(price) = prices.get_mut(&(
                asset_rate.from.clone(),
                asset_rate.class_id,
                asset_rate.asset_id,
            )) {
                *price = price.checked_add(amount).ok_or(Error::<T>::Overflow)?;
            } else {
                prices.insert(
                    (
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ),
                    amount,
                );
            }
        }

        // RateAction::Burn - Compute total burns

        for asset_rate in rates.iter() {
            if let RateAction::Burn = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnBalance)?;
                *balance = balance.checked_sub(*price).ok_or(Error::<T>::Overflow)?;
                if *balance < 0 {
                    can_do_exchange = false;
                    exchange_balances.insert(asset_rate.clone(), *balance);
                } else {
                    exchange_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Transfer - Compute total transfers

        for asset_rate in rates.iter() {
            if let RateAction::Transfer = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferPrice)?;
                let balance = balances
                    .get_mut(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferBalance)?;
                *balance = balance.checked_sub(*price).ok_or(Error::<T>::Overflow)?;
                if *balance < 0 {
                    can_do_exchange = false;
                    exchange_balances.insert(asset_rate.clone(), *balance);
                } else {
                    exchange_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Mint - Compute total mints

        for asset_rate in rates.iter() {
            if let RateAction::Mint = &asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                exchange_balances.insert(asset_rate.clone(), *price);
            }
        }

        Ok((can_do_exchange, exchange_balances))
    }

    pub fn do_exchange_assets(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*buyer != market.owner, Error::<T>::InvalidBuyer);
        ensure!(*buyer != market.vault, Error::<T>::InvalidBuyer);

        let (can_do_exchange, exchange_balances) =
            Self::do_compute_exchange(buyer, market_id, market_rate_id, amount)?;

        ensure!(can_do_exchange, Error::<T>::InsufficientAmount);

        for (asset_rate, amount) in &exchange_balances {
            let amount: u128 = (*amount).try_into().map_err(|_| Error::<T>::Overflow)?;
            let from = match &asset_rate.from {
                RateAccount::Account(account) => account,
                RateAccount::Buyer => buyer,
                RateAccount::Market => &market.vault,
            };
            let to = match &asset_rate.to {
                RateAccount::Account(account) => account,
                RateAccount::Buyer => buyer,
                RateAccount::Market => &market.vault,
            };
            match asset_rate.action {
                RateAction::Transfer => sugarfunge_asset::Pallet::<T>::do_transfer_from(
                    &market.owner,
                    from,
                    to,
                    asset_rate.class_id,
                    asset_rate.asset_id,
                    amount,
                )?,
                RateAction::Burn => sugarfunge_asset::Pallet::<T>::do_burn(
                    &market.owner,
                    from,
                    asset_rate.class_id,
                    asset_rate.asset_id,
                    amount,
                )?,
                RateAction::Mint => sugarfunge_asset::Pallet::<T>::do_mint(
                    &market.owner,
                    to,
                    asset_rate.class_id,
                    asset_rate.asset_id,
                    amount,
                )?,
                _ => (),
            }
        }

        Self::deposit_event(Event::Exchanged {
            buyer: buyer.clone(),
            market_id,
            market_rate_id,
            amount,
            // balances: exchange_balances,
        });

        Ok(().into())
    }
}

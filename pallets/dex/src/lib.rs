#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
    PalletId,
};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, Zero},
    RuntimeDebug,
};
use sp_std::{convert::TryInto, prelude::*};
use sugarfunge_primitives::{Balance, CurrencyId};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub type ExchangeId = u32;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + sugarfunge_currency::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type PalletId: Get<PalletId>;

        /// The minimum balance to create exchange
        #[pallet::constant]
        type CreateExchangeDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Exchanges<T: Config> =
        StorageMap<_, Blake2_128, ExchangeId, Exchange<T::ClassId, T::AssetId, T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn total_supplies)]
    pub(super) type TotalSupplies<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ExchangeId,
        Blake2_128Concat,
        T::AssetId,
        Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn currency_reserves)]
    pub(super) type CurrencyReserves<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        ExchangeId,
        Blake2_128Concat,
        T::AssetId,
        Balance,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ExchangeCreated {
            exchange_id: ExchangeId,
            who: T::AccountId,
        },
        CurrencyToAsset {
            exchange_id: ExchangeId,
            who: T::AccountId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts_out: Vec<Balance>,
            currency_amounts_in: Vec<Balance>,
        },
        AssetToCurrency {
            exchange_id: ExchangeId,
            who: T::AccountId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts_in: Vec<Balance>,
            currency_amounts_out: Vec<Balance>,
        },
        LiquidityAdded {
            exchange_id: ExchangeId,
            who: T::AccountId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts: Vec<Balance>,
            currency_amounts: Vec<Balance>,
        },
        LiquidityRemoved {
            exchange_id: ExchangeId,
            who: T::AccountId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts: Vec<Balance>,
            currency_amounts: Vec<Balance>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Overflow,
        InvalidExchange,
        InvalidAssetClass,
        InvalidLiquidityClass,
        InvalidMaxCurrency,
        InsufficientCurrencyAmount,
        InsufficientAssetAmount,
        SameCurrencyAndAsset,
        MaxCurrencyAmountExceeded,
        InvalidCurrencyAmount,
        InsufficientLiquidity,
        NullAssetsBought,
        NullAssetsSold,
        EmptyReserve,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_exchange(
            origin: OriginFor<T>,
            exchange_id: ExchangeId,
            currency_id: CurrencyId,
            asset_class_id: T::ClassId,
            lp_class_id: T::ClassId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            ensure!(
                !Exchanges::<T>::contains_key(exchange_id),
                Error::<T>::InvalidExchange
            );

            ensure!(
                sugarfunge_asset::Pallet::<T>::class_exists(asset_class_id),
                Error::<T>::InvalidAssetClass
            );

            ensure!(
                !sugarfunge_asset::Pallet::<T>::class_exists(lp_class_id),
                Error::<T>::InvalidLiquidityClass
            );

            let fund_account = <T as Config>::PalletId::get().into_sub_account(exchange_id);

            let deposit = T::CreateExchangeDeposit::get();
            <T as Config>::Currency::transfer(&who, &fund_account, deposit, AllowDeath)?;

            sugarfunge_asset::Pallet::<T>::do_create_class(
                &fund_account,
                &fund_account,
                lp_class_id,
                vec![].try_into().unwrap(),
            )?;

            let (currency_class_id, currency_asset_id) =
                sugarfunge_currency::Pallet::<T>::get_currency_asset(currency_id)?;

            let new_exchange = Exchange {
                creator: who.clone(),
                asset_class_id,
                currency_class_id,
                currency_asset_id,
                lp_class_id,
                vault: fund_account,
            };

            Exchanges::<T>::insert(exchange_id, new_exchange);

            Self::deposit_event(Event::ExchangeCreated { exchange_id, who });

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn buy_assets(
            origin: OriginFor<T>,
            exchange_id: ExchangeId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts_out: Vec<Balance>,
            max_currency: Balance,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_buy_assets(
                &who,
                exchange_id,
                asset_ids,
                asset_amounts_out,
                max_currency,
                &to,
            )?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn sell_assets(
            origin: OriginFor<T>,
            exchange_id: ExchangeId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts_in: Vec<Balance>,
            min_currency: Balance,
            to: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_sell_assets(
                &who,
                exchange_id,
                asset_ids,
                asset_amounts_in,
                min_currency,
                &to,
            )?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            exchange_id: ExchangeId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            asset_amounts: Vec<Balance>,
            max_currencies: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_add_liquidity(
                &who,
                exchange_id,
                &to,
                asset_ids,
                asset_amounts,
                max_currencies,
            )?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn remove_liquidity(
            origin: OriginFor<T>,
            exchange_id: ExchangeId,
            to: T::AccountId,
            asset_ids: Vec<T::AssetId>,
            liquidities: Vec<Balance>,
            min_currencies: Vec<Balance>,
            min_assets: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_remove_liquidity(
                &who,
                exchange_id,
                &to,
                asset_ids,
                liquidities,
                min_currencies,
                min_assets,
            )?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Exchange<ClassId, AssetId, AccountId> {
    /// The creator of Exchange
    pub creator: AccountId,
    /// The class of the assets
    pub asset_class_id: ClassId,
    /// The class of the currency
    pub currency_class_id: ClassId,
    /// The asset of the currency class
    pub currency_asset_id: AssetId,
    /// The class of exchange liquidity pool
    pub lp_class_id: ClassId,
    /// The fund account of exchange
    pub vault: AccountId,
}

impl<T: Config> Pallet<T> {
    // currency to asset
    pub fn do_buy_assets(
        who: &T::AccountId,
        exchange_id: ExchangeId,
        asset_ids: Vec<T::AssetId>,
        asset_amounts_out: Vec<Balance>,
        max_currency: Balance,
        to: &T::AccountId,
    ) -> DispatchResult {
        let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchange)?;

        let n = asset_ids.len();
        let mut total_currency = Balance::from(0u128);
        let mut currency_amounts_in = vec![Balance::from(0u128); n];

        let asset_reserves =
            Self::get_asset_reserves(&exchange.vault, exchange.asset_class_id, asset_ids.clone());

        for i in 0..n {
            let asset_id = asset_ids[i];
            let amount_out = asset_amounts_out[i];
            let asset_reserve = asset_reserves[i];

            ensure!(amount_out > Zero::zero(), Error::<T>::NullAssetsBought);

            let currency_reserve = Self::currency_reserves(exchange_id, asset_id);
            let currency_amount = Self::get_buy_price(amount_out, currency_reserve, asset_reserve)?;

            total_currency = total_currency.saturating_add(currency_amount);
            ensure!(
                total_currency <= max_currency,
                Error::<T>::MaxCurrencyAmountExceeded
            );

            currency_amounts_in[i] = currency_amount;

            CurrencyReserves::<T>::try_mutate(
                exchange_id,
                asset_id,
                |currency_reserve| -> DispatchResult {
                    *currency_reserve = currency_reserve
                        .checked_add(currency_amount)
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(())
                },
            )?;
        }

        // Transfer currency asset to exchange vault
        sugarfunge_asset::Pallet::<T>::do_transfer_from(
            who,
            who,
            &exchange.vault,
            exchange.currency_class_id,
            exchange.currency_asset_id,
            total_currency,
        )?;

        // Send all assets purchased
        sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
            &exchange.vault,
            &exchange.vault,
            &to,
            exchange.asset_class_id,
            asset_ids.clone(),
            asset_amounts_out.clone(),
        )?;

        Self::deposit_event(Event::CurrencyToAsset {
            exchange_id,
            who: who.clone(),
            to: to.clone(),
            asset_ids,
            asset_amounts_out,
            currency_amounts_in,
        });

        Ok(())
    }

    // asset to currency
    pub fn do_sell_assets(
        who: &T::AccountId,
        exchange_id: ExchangeId,
        asset_ids: Vec<T::AssetId>,
        asset_amounts_in: Vec<Balance>,
        min_currency: Balance,
        to: &T::AccountId,
    ) -> DispatchResult {
        let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchange)?;

        let n = asset_ids.len();
        let mut total_currency = Balance::from(0u128);
        let mut currency_amounts_out = vec![Balance::from(0u128); n];

        let asset_reserves =
            Self::get_asset_reserves(&exchange.vault, exchange.asset_class_id, asset_ids.clone());

        for i in 0..n {
            let asset_id = asset_ids[i];
            let amount_in = asset_amounts_in[i];
            let asset_reserve = asset_reserves[i];

            ensure!(amount_in > Zero::zero(), Error::<T>::NullAssetsSold);

            let currency_reserve = Self::currency_reserves(exchange_id, asset_id);
            let currency_amount = Self::get_sell_price(
                amount_in,
                asset_reserve.saturating_sub(amount_in),
                currency_reserve,
            )?;

            total_currency = total_currency.saturating_add(currency_amount);
            currency_amounts_out[i] = currency_amount;

            CurrencyReserves::<T>::try_mutate(
                exchange_id,
                asset_id,
                |currency_reserve| -> DispatchResult {
                    *currency_reserve = currency_reserve
                        .checked_sub(currency_amount)
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(())
                },
            )?;
        }

        ensure!(
            total_currency >= min_currency,
            Error::<T>::InsufficientCurrencyAmount
        );

        // Transfer the assets to sell to exchange vault
        sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
            who,
            who,
            &exchange.vault,
            exchange.asset_class_id,
            asset_ids.clone(),
            asset_amounts_in.clone(),
        )?;

        // Transfer currency here
        sugarfunge_asset::Pallet::<T>::do_transfer_from(
            &exchange.vault,
            &exchange.vault,
            &to,
            exchange.currency_class_id,
            exchange.currency_asset_id,
            total_currency,
        )?;

        Self::deposit_event(Event::AssetToCurrency {
            exchange_id,
            who: who.clone(),
            to: to.clone(),
            asset_ids,
            asset_amounts_in,
            currency_amounts_out,
        });

        Ok(())
    }

    // add liquidity
    pub fn do_add_liquidity(
        who: &T::AccountId,
        exchange_id: ExchangeId,
        to: &T::AccountId,
        asset_ids: Vec<T::AssetId>,
        asset_amounts: Vec<Balance>,
        max_currencies: Vec<Balance>,
    ) -> DispatchResult {
        let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchange)?;

        let n = asset_ids.len();
        let mut total_currency = Balance::from(0u128);
        let mut liquidities_to_mint = vec![Balance::from(0u128); n];
        let mut currency_amounts = vec![Balance::from(0u128); n];

        let asset_reserves =
            Self::get_asset_reserves(&exchange.vault, exchange.asset_class_id, asset_ids.clone());

        for i in 0..n {
            let asset_id = asset_ids[i];
            let amount = asset_amounts[i];

            ensure!(
                max_currencies[i] > Zero::zero(),
                Error::<T>::InvalidMaxCurrency
            );
            ensure!(amount > Zero::zero(), Error::<T>::InsufficientAssetAmount);

            if exchange.currency_class_id == exchange.asset_class_id {
                ensure!(
                    exchange.currency_asset_id != asset_id,
                    Error::<T>::SameCurrencyAndAsset
                );
            }

            let total_liquidity = Self::total_supplies(exchange_id, asset_id);

            if total_liquidity > Zero::zero() {
                let currency_reserve = Self::currency_reserves(exchange_id, asset_id);
                let asset_reserve = asset_reserves[i];

                let (currency_amount, rounded) = Self::div_round(
                    U256::from(amount).saturating_mul(U256::from(currency_reserve)),
                    U256::from(asset_reserve).saturating_sub(U256::from(amount)),
                );
                ensure!(
                    max_currencies[i] >= currency_amount,
                    Error::<T>::MaxCurrencyAmountExceeded
                );

                total_currency = total_currency.saturating_add(currency_amount);

                let fixed_currency_amount = if rounded {
                    currency_amount.saturating_sub(1u128)
                } else {
                    currency_amount
                };
                liquidities_to_mint[i] =
                    (fixed_currency_amount.saturating_mul(total_liquidity)) / currency_reserve;
                currency_amounts[i] = currency_amount;

                CurrencyReserves::<T>::try_mutate(
                    exchange_id,
                    asset_id,
                    |currency_reserve| -> DispatchResult {
                        *currency_reserve = currency_reserve
                            .checked_add(currency_amount)
                            .ok_or(Error::<T>::Overflow)?;
                        Ok(())
                    },
                )?;

                TotalSupplies::<T>::try_mutate(
                    exchange_id,
                    asset_id,
                    |total_supply| -> DispatchResult {
                        *total_supply = total_liquidity
                            .checked_add(liquidities_to_mint[i])
                            .ok_or(Error::<T>::Overflow)?;
                        Ok(())
                    },
                )?;
            } else {
                let max_currency = max_currencies[i];

                // Otherwise rounding error could end up being significant on second deposit
                ensure!(
                    max_currency >= Balance::from(1000000000u128),
                    Error::<T>::InvalidCurrencyAmount
                );

                total_currency = total_currency.saturating_add(max_currency);
                liquidities_to_mint[i] = max_currency;
                currency_amounts[i] = max_currency;

                CurrencyReserves::<T>::mutate(exchange_id, asset_id, |currency_reserve| {
                    *currency_reserve = max_currency
                });
                TotalSupplies::<T>::mutate(exchange_id, asset_id, |total_supply| {
                    *total_supply = max_currency
                });
            }
        }

        // Transfer the assets to add to the exchange liquidity pools
        sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
            who,
            who,
            &exchange.vault,
            exchange.asset_class_id,
            asset_ids.clone(),
            asset_amounts.clone(),
        )?;

        // Mint liquidity pool assets
        sugarfunge_asset::Pallet::<T>::do_batch_mint(
            &exchange.vault,
            &to,
            exchange.lp_class_id,
            asset_ids.clone(),
            liquidities_to_mint,
        )?;

        // Transfer all currency to this contract
        sugarfunge_asset::Pallet::<T>::do_transfer_from(
            &who,
            &who,
            &exchange.vault,
            exchange.currency_class_id,
            exchange.currency_asset_id,
            total_currency,
        )?;

        Self::deposit_event(Event::LiquidityAdded {
            exchange_id,
            who: who.clone(),
            to: to.clone(),
            asset_ids,
            asset_amounts,
            currency_amounts,
        });

        Ok(())
    }

    // remove liquidity
    pub fn do_remove_liquidity(
        who: &T::AccountId,
        exchange_id: ExchangeId,
        to: &T::AccountId,
        asset_ids: Vec<T::AssetId>,
        liquidities: Vec<Balance>,
        min_currencies: Vec<Balance>,
        min_assets: Vec<Balance>,
    ) -> DispatchResult {
        let exchange = Exchanges::<T>::get(exchange_id).ok_or(Error::<T>::InvalidExchange)?;

        let n = asset_ids.len();
        let mut total_currency = Balance::from(0u128);
        let mut asset_amounts = vec![Balance::from(0u128); n];
        let mut currency_amounts = vec![Balance::from(0u128); n];

        let asset_reserves =
            Self::get_asset_reserves(&exchange.vault, exchange.asset_class_id, asset_ids.clone());

        for i in 0..n {
            let asset_id = asset_ids[i];
            let liquidity = liquidities[i];
            let asset_reserve = asset_reserves[i];

            let total_liquidity = Self::total_supplies(exchange_id, asset_id);
            ensure!(
                total_liquidity > Zero::zero(),
                Error::<T>::InsufficientLiquidity
            );

            let currency_reserve = Self::currency_reserves(exchange_id, asset_id);

            let currency_amount = U256::from(liquidity)
                .saturating_mul(U256::from(currency_reserve))
                .checked_div(U256::from(total_liquidity))
                .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                .unwrap_or_else(Zero::zero);

            let asset_amount = U256::from(liquidity)
                .saturating_mul(U256::from(asset_reserve))
                .checked_div(U256::from(total_liquidity))
                .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                .unwrap_or_else(Zero::zero);

            ensure!(
                currency_amount >= min_currencies[i],
                Error::<T>::InsufficientCurrencyAmount
            );
            ensure!(
                asset_amount >= min_assets[i],
                Error::<T>::InsufficientAssetAmount
            );

            total_currency = total_currency.saturating_add(currency_amount);
            asset_amounts[i] = asset_amount;
            currency_amounts[i] = currency_amount;

            CurrencyReserves::<T>::try_mutate(
                exchange_id,
                asset_id,
                |currency_reserve| -> DispatchResult {
                    *currency_reserve = currency_reserve
                        .checked_sub(currency_amount)
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(())
                },
            )?;

            TotalSupplies::<T>::try_mutate(
                exchange_id,
                asset_id,
                |total_supply| -> DispatchResult {
                    *total_supply = total_liquidity
                        .checked_sub(liquidity)
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(())
                },
            )?;
        }

        // Transfer the liquidity pool assets to burn to exchange vault
        sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
            who,
            who,
            &exchange.vault,
            exchange.lp_class_id,
            asset_ids.clone(),
            liquidities.clone(),
        )?;

        // Burn liquidity pool assets for offchain supplies
        sugarfunge_asset::Pallet::<T>::do_batch_burn(
            &exchange.vault,
            &exchange.vault,
            exchange.lp_class_id,
            asset_ids.clone(),
            liquidities,
        )?;

        // Transfer total currency
        sugarfunge_asset::Pallet::<T>::do_transfer_from(
            &exchange.vault,
            &exchange.vault,
            &to,
            exchange.currency_class_id,
            exchange.currency_asset_id,
            total_currency,
        )?;

        // Transfer all assets ids
        sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
            &exchange.vault,
            &exchange.vault,
            &to,
            exchange.asset_class_id,
            asset_ids.clone(),
            asset_amounts.clone(),
        )?;

        Self::deposit_event(Event::LiquidityRemoved {
            exchange_id,
            who: who.clone(),
            to: to.clone(),
            asset_ids,
            asset_amounts,
            currency_amounts,
        });

        Ok(())
    }

    /// Pricing function used for converting between currency asset to assets.
    ///
    /// - `amount_out`: Amount of assets being bought.
    /// - `reserve_in`: Amount of currency assets in exchange reserves.
    /// - `reserve_out`: Amount of assets in exchange reserves.
    /// Return the price Amount of currency assets to send to dex.
    pub fn get_buy_price(
        amount_out: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, DispatchError> {
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T>::EmptyReserve
        );

        let numerator: U256 = U256::from(reserve_in)
            .saturating_mul(U256::from(amount_out))
            .saturating_mul(U256::from(1000u128));
        let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(amount_out)))
            .saturating_mul(U256::from(995u128));

        ensure!(
            denominator > U256::from(0),
            Error::<T>::InsufficientLiquidity
        );

        let (amount_in, _) = Self::div_round(numerator, denominator);

        Ok(amount_in)
    }

    /// Pricing function used for converting assets to currency asset.
    ///
    /// - `amount_in`: Amount of assets being sold.
    /// - `reserve_in`: Amount of assets in exchange reserves.
    /// - `reserve_out`: Amount of currency assets in exchange reserves.
    /// Return the price Amount of currency assets to receive from dex.
    pub fn get_sell_price(
        amount_in: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, DispatchError> {
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T>::EmptyReserve
        );

        let amount_in_with_fee: U256 = U256::from(amount_in).saturating_mul(U256::from(995u128));
        let numerator: U256 =
            U256::from(amount_in_with_fee).saturating_mul(U256::from(reserve_out));
        let denominator: U256 = (U256::from(reserve_in).saturating_mul(U256::from(1000u128)))
            .saturating_add(amount_in_with_fee);

        let amount_out = numerator
            .checked_div(denominator)
            .and_then(|n| TryInto::<Balance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero);

        Ok(amount_out)
    }

    fn get_asset_reserves(
        vault: &T::AccountId,
        class_id: T::ClassId,
        asset_ids: Vec<T::AssetId>,
    ) -> Vec<Balance> {
        let n = asset_ids.len();

        if n == 1 {
            let mut asset_reserves = vec![Balance::from(0u128); n];
            asset_reserves[0] =
                sugarfunge_asset::Pallet::<T>::balance_of(vault, class_id, asset_ids[0]);
            asset_reserves
        } else {
            let vaults = vec![vault.clone(); n];
            let asset_reserves =
                sugarfunge_asset::Pallet::<T>::balance_of_batch(&vaults, class_id, asset_ids)
                    .unwrap();
            asset_reserves
        }
    }

    /// Divides two numbers and add 1 if there is a rounding error
    fn div_round(numerator: U256, denominator: U256) -> (Balance, bool) {
        let remainder = numerator.checked_rem(denominator).unwrap();
        if remainder.is_zero() {
            (
                numerator
                    .checked_div(denominator)
                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                    .unwrap_or_else(Zero::zero),
                false,
            )
        } else {
            (
                numerator
                    .checked_div(denominator)
                    .and_then(|r| r.checked_add(U256::one()))
                    .and_then(|n| TryInto::<Balance>::try_into(n).ok())
                    .unwrap_or_else(Zero::zero),
                true,
            )
        }
    }
}

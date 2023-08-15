// SBP-M1 review: use cargo fmt
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::Get,
    BoundedVec, PalletId,
};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
    traits::{AccountIdConversion, AtLeast32BitUnsigned, Zero},
    RuntimeDebug,
};
use sp_std::{collections::btree_map::BTreeMap, prelude::*};
// SBP-M1 review: should amount not be a u128 rather than signed?
use sugarfunge_primitives::{Amount, Balance};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// SBP-M1 review: consider moving types to separate module
// SBP-M1 review: add doc comments

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
pub enum AmountOp {
    Equal,
    LessThan,
    LessEqualThan,
    GreaterThan,
    GreaterEqualThan,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
pub enum AMM {
    Constant,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
pub enum RateAction<ClassId, AssetId> {
    Transfer(Amount),
    MarketTransfer(AMM, ClassId, AssetId),
    Mint(Amount),
    Burn(Amount),
    Has(AmountOp, Amount),
}

impl<ClassId, AssetId> RateAction<ClassId, AssetId> {
    fn get_amount(&self) -> Amount {
        match *self {
            // SBP-M1 review: merge arm patterns with same body
            // SBP-M1 review: use Self instead of RateAction
            RateAction::Burn(amount) => amount,
            RateAction::Mint(amount) => amount,
            RateAction::Transfer(amount) => amount,
            // SBP-M1 review: explicit variant handling preferred > 'RateAction::MarketTransfer(..) | RateAction::Has(..) => 0,'
            _ => 0 as Amount,
        }
    }
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
pub enum RateAccount<AccountId> {
    Market,
    Account(AccountId),
    Buyer,
}

#[derive(
    Encode,
    Decode,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    RuntimeDebug,
    TypeInfo,
    MaxEncodedLen,
)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
// SBP-M1 review: name misleading, consider renaming to something like Instruction based on description in simple_market_rates() test helper
pub struct AssetRate<AccountId, ClassId, AssetId> {
    class_id: ClassId,
    asset_id: AssetId,
    action: RateAction<ClassId, AssetId>,
    from: RateAccount<AccountId>,
    to: RateAccount<AccountId>,
}

// SBP-M1 review: consider pub(crate) or less, depending on usage
pub type Rates<T> = BoundedVec<
    AssetRate<
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    >,
    <T as Config>::MaxRates,
>;

// SBP-M1 review: consider pub(crate) or less, depending on usage
pub type RateBalances<T> = BTreeMap<
    AssetRate<
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    >,
    Amount,
>;

type TransactionBalances<T> = BTreeMap<
    (
        RateAccount<<T as frame_system::Config>::AccountId>,
        // SBP-M1 review: consider defining these types on Config type of this pallet, configured via runtime
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
    ),
    Amount,
>;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, RuntimeDebug, TypeInfo)]
// SBP-M1 review: consider pub(crate) or less, depending on usage
pub struct RateBalance<AccountId, ClassId, AssetId> {
    rate: AssetRate<AccountId, ClassId, AssetId>,
    balance: Amount,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // SBP-M1 review: remove comment
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    // SBP-M1 review: loose pallet coupling preferable, can sugarfunge_asset dependency be brought in via trait (associated type on this pallets Config trait)?
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // SBP-M1 review: missing doc comment
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        // SBP-M1 review: missing doc comment
        type MarketId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        // SBP-M1 review: missing doc comment
        type MarketRateId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        /// Max number of rates per market_rate
        #[pallet::constant]
        type MaxRates: Get<u32>;

        /// Max metadata size
        #[pallet::constant]
        type MaxMetadata: Get<u32>;
    }

    #[pallet::pallet]
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

    // SBP-M1 review: add doc comments
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
        LiquidityAdded {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            class_ids: Vec<T::ClassId>,
            asset_ids: Vec<Vec<T::AssetId>>,
            amounts: Vec<Vec<Balance>>,
        },
        LiquidityRemoved {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            class_ids: Vec<T::ClassId>,
            asset_ids: Vec<Vec<T::AssetId>>,
            amounts: Vec<Vec<Balance>>,
        },
        Deposit {
            who: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            balances: Vec<RateBalance<T::AccountId, T::ClassId, T::AssetId>>,
            success: bool,
        },
        Exchanged {
            buyer: T::AccountId,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            balances: Vec<RateBalance<T::AccountId, T::ClassId, T::AssetId>>,
            success: bool,
        },
    }

    // SBP-M1 review: add doc comments
    #[pallet::error]
    pub enum Error<T> {
        // SBP-M1 review: can use ArithmeticError::Overflow instead of custom overflow type
        Overflow,
        InsufficientAmount,
        InsufficientLiquidity,
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
        InvalidArrayLength,
    }

    // SBP-M1 review: remove template comment
    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // SBP-M1 review: add doc comments
        #[pallet::call_index(0)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_market(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: how are markets removed? Can a deposit be taken to create a market to incentivize owner to remove market when no longer required?
            // SBP-M1 review: market rate removal also needs consideration
            // SBP-M1 review: could simplify and just return value from Self::do_create_market(..)
            Self::do_create_market(&who, market_id)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: add doc comments
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create_market_rate(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            rates: Rates<T>,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_create_market_rate(..)
            Self::do_create_market_rate(&who, market_id, market_rate_id, &rates)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: add doc comments
        #[pallet::call_index(2)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn deposit(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_deposit(..)
            Self::do_deposit(&who, market_id, market_rate_id, amount)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: add doc comments
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function - static weight definition probably underweight and no proof_size, necessary when parachain
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn exchange_assets(
            origin: OriginFor<T>,
            market_id: T::MarketId,
            market_rate_id: T::MarketRateId,
            amount: Balance,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_exchange_assets(..)
            Self::do_exchange_assets(&who, market_id, market_rate_id, amount)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }
    }
}

// SBP-M1 review: add doc comment and consider relocating closer to other structs
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Market<AccountId> {
    /// The owner of the market
    pub owner: AccountId,
    /// The fund account of the market
    pub vault: AccountId,
}

// SBP-M1 review: verify pub visibility on all functions. Wrap in traits to signal if they should be accessible to other pallets
impl<T: Config> Pallet<T> {
    // SBP-M1 review: probably doesnt need to be public, consider pub(crate), pub(super)...
    pub fn do_create_market(who: &T::AccountId, market_id: T::MarketId) -> DispatchResult {
        ensure!(
            !Markets::<T>::contains_key(market_id),
            Error::<T>::MarketExists
        );

        // SBP-M1 review: vault account may require existential deposit (especially for native currency), could take from owner here at creation and return once market removed/closed
        // SBP-M1 review: may apply for each asset depending on implementation in asset pallet
        let vault: T::AccountId =
            <T as Config>::PalletId::get().into_sub_account_truncating(market_id);

        Markets::<T>::insert(
            market_id,
            Market {
                owner: who.clone(),
                vault,
            },
        );

        Self::deposit_event(Event::Created {
            market_id,
            // SBP-M1 review: unnecessary clone
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
        // SBP-M1 review: can market not be auto-created if it doesnt exist? Currently requires one transaction to create market and then another to submit rates
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            !MarketRates::<T>::contains_key((market_id, market_rate_id)),
            Error::<T>::MarketRateExists
        );

        // Ensure rates are valid
        for asset_rate in rates.iter() {
            // SBP-M1 review: duplicate code, use asset_rate.action.get_amount()
            let amount = match asset_rate.action {
                // SBP-M1 review: merge arms > RateAction::Burn(amount) | RateAction::Mint(amount) ...
                RateAction::Burn(amount) => amount,
                RateAction::Mint(amount) => amount,
                RateAction::Transfer(amount) => amount,
                // SBP-M1 review: wildcard match will also match any future variants, prefer being explicit
                _ => 0 as Amount,
            };
            // SBP-M1 review: can be simplified to ensure!(asset_rate.action.get_amount() >= 0,...)
            // SBP-M1 review: Amount type being signed is questionable, using u128 removes the need for this check
            ensure!(amount >= 0, Error::<T>::InvalidRateAmount);
        }

        MarketRates::<T>::insert((market_id, market_rate_id), rates);

        Self::deposit_event(Event::RateCreated {
            market_id,
            market_rate_id,
            // SBP-M1 review: unnecessary clone
            who: who.clone(),
        });

        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    // SBP-M1 review: who uses this? Appears to be no usage in project. Wrap in a well documented trait so other pallets can call via trait
    pub fn add_liquidity(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        // SBP-M1 review: vectors should be bounded
        // SBP-M1 review: use Vec<(T::ClassId, Vec<T::AssetId>, Vec<Balance>)> to avoid needing to check sizes of each vector match
        class_ids: Vec<T::ClassId>,
        asset_ids: Vec<Vec<T::AssetId>>,
        amounts: Vec<Vec<Balance>>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used, used .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move above market rate check to prevent unnecessary read if caller is not market owner
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            asset_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        for (idx, class_id) in class_ids.iter().enumerate() {
            ensure!(
                // SBP-M1 review: whilst size checked above, using .get() is preferred over indexing due to possibility of panic
                asset_ids[idx].len() == amounts[idx].len(),
                Error::<T>::InvalidArrayLength
            );

            // SBP-M1 review: should use trait, defined as associated type on Config trait of pallet (loose coupling)
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &market.owner,
                &market.owner,
                &market.vault,
                *class_id,
                // SBP-M1 review: indexing may panic, consider .get() instead
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::LiquidityAdded {
            // SBP-M1 review: unnecessary clone
            who: who.clone(),
            market_id,
            market_rate_id,
            class_ids,
            asset_ids,
            amounts,
        });

        // SBP-M1 review: unnecessary .into()
        Ok(().into())
    }

    // SBP-M1 review: too many lines, refactor
    // SBP-M1 review: who uses this? Appears to be no usage in project. Wrap in a well documented trait so other pallets can call via trait
    pub fn remove_liquidity(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        // SBP-M1 review: vectors should be bounded
        // SBP-M1 review: use Vec<(T::ClassId, Vec<T::AssetId>, Vec<Balance>)> to avoid needing to check sizes of each vector match
        class_ids: Vec<T::ClassId>,
        asset_ids: Vec<Vec<T::AssetId>>,
        amounts: Vec<Vec<Balance>>,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used, used .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move above market rate check to prevent unnecessary read if caller is not market owner
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            asset_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        for (idx, class_id) in class_ids.iter().enumerate() {
            ensure!(
                // SBP-M1 review: whilst size checked above, using .get() is preferred over indexing due to possibility of panic
                asset_ids[idx].len() == amounts[idx].len(),
                Error::<T>::InvalidArrayLength
            );

            // SBP-M1 review: should use trait, defined as associated type on Config trait of pallet (loose coupling)
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &market.owner,
                &market.vault,
                &market.owner,
                *class_id,
                // SBP-M1 review: indexing may panic, consider .get() instead
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::LiquidityRemoved {
            // SBP-M1 review: unnecessary clone
            who: who.clone(),
            market_id,
            market_rate_id,
            class_ids,
            asset_ids,
            amounts,
        });

        // SBP-M1 review: unnecessary .into()
        Ok(().into())
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_quote_deposit(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
        // SBP-M1 review: consider changing to Result<RateBalances<T>, DispatchError> and simply returning error if !can_do_deposit
    ) -> Result<(bool, RateBalances<T>), DispatchError> {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move above rates check, inefficient to read all rates to then find caller is not owner when that data already in memory from previous call
        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let mut deposit_balances = BTreeMap::new();

        // SBP-M1 review: consider grouping each 'phase' of algorithm into blocks: aggregate, compute burns, compute transfers
        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalances<T> = BTreeMap::new();

        let mut prices: TransactionBalances<T> = BTreeMap::new();

        // SBP-M1 review: why use a signed integer?
        let total_amount: i128 = amount.try_into().map_err(|_| Error::<T>::Overflow)?;

        // SBP-M1 review: filter and collect in single statement > rates.into_iter().filter(..).collect();
        let asset_rates = rates.into_iter().filter(|asset_rate| {
            // SBP-M1 review: refactor into .quotable() function on enum to simplify to asset_rate.from == RateAccount::Market && asset_rate.action.quotable()
            // SBP-M1 review: alternatively use matches!
            let quotable = match asset_rate.action {
                RateAction::Transfer(_) | RateAction::Burn(_) => true,
                // SBP-M1 review: wildcard will match future added variants
                _ => false,
            };
            asset_rate.from == RateAccount::Market && quotable
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
                .action
                .get_amount()
                .checked_mul(total_amount)
                .ok_or(Error::<T>::Overflow)?;
            // SBP-M1 review: duplicate code, refactor into reusable function
            // SBP-M1 review: consider BTreeMap.entry() api
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
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Burn(_) = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidBurnPrice)?;
                // SBP-M1 review: consider .entry() api
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
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Transfer(_) = asset_rate.action {
                let price = prices
                    .get(&(
                        asset_rate.from.clone(),
                        asset_rate.class_id,
                        asset_rate.asset_id,
                    ))
                    .ok_or(Error::<T>::InvalidTransferPrice)?;
                // SBP-M1 review: consider .entry() api
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

        // SBP-M1 review: use deposit_balances.values()
        for (_, deposit_balance) in &deposit_balances {
            if *deposit_balance < 0 {
                // SBP-M1 review: consider return Err(Error::CannotDeposit);
                can_do_deposit = false;
                break;
            }
        }

        // SBP-M1 review: consider Ok(deposit_balances)
        Ok((can_do_deposit, deposit_balances))
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_deposit(
        who: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used therefore inefficient, use .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        ensure!(*who == market.owner, Error::<T>::InvalidMarketOwner);

        let (can_do_deposit, deposit_balances) =
            Self::do_quote_deposit(who, market_id, market_rate_id, amount)?;

        // SBP-M1 review: consider returning an error rather than signalling that caller can not deposit via success attribute of Deposit event
        if can_do_deposit {
            // SBP-M1 review: each transfer request results in separate state change, consider grouping by class/asset if applicable
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
        }

        let balances = deposit_balances
            .iter()
            .map(|(rate, balance)| RateBalance {
                rate: rate.clone(),
                balance: *balance,
            })
            .collect();

        // SBP-M1 review: consider emitting an error if the deposit could not be performed rather than success: can_do_deposit
        Self::deposit_event(Event::Deposit {
            // SBP-M1 review: unnecessary clone()
            who: who.clone(),
            market_id,
            market_rate_id,
            amount,
            balances,
            success: can_do_deposit,
        });

        // SBP-M1 review: .into() unnecessary
        Ok(().into())
    }

    pub fn get_vault(market_id: T::MarketId) -> Option<T::AccountId> {
        // SBP-M1 review: use .map() instead of .and_then()
        Markets::<T>::get(market_id).and_then(|market| Some(market.vault))
    }

    pub fn balance(
        market: &Market<T::AccountId>,
        class_id: T::ClassId,
        asset_id: T::AssetId,
    ) -> Balance {
        sugarfunge_asset::Pallet::<T>::balance_of(&market.vault, class_id, asset_id)
    }

    // SBP-M1 review: typo - incomming > incoming
    /// Pricing function used for converting between outgoing asset to incomming asset.
    ///
    /// - `amount_out`: Amount of outgoing asset being bought.
    /// - `reserve_in`: Amount of incomming asset in reserves.
    /// - `reserve_out`: Amount of outgoing asset in reserves.
    /// Return the price Amount of incomming asset to send to vault.
    // SBP-M1 review: unused, no test
    pub fn get_buy_price(
        amount_out: Balance,
        reserve_in: Balance,
        reserve_out: Balance,
    ) -> Result<Balance, DispatchError> {
        ensure!(
            reserve_in > Zero::zero() && reserve_out > Zero::zero(),
            Error::<T>::InsufficientLiquidity
        );

        let numerator: U256 = U256::from(reserve_in)
            .saturating_mul(U256::from(amount_out))
            .saturating_mul(U256::from(1000u128));
        let denominator: U256 = (U256::from(reserve_out).saturating_sub(U256::from(amount_out)))
            .saturating_mul(U256::from(995u128));

        let amount_in = numerator
            .checked_div(denominator)
            .and_then(|r| r.checked_add(U256::one())) // add 1 to correct possible losses caused by remainder discard
            .and_then(|n| TryInto::<Balance>::try_into(n).ok())
            .unwrap_or_else(Zero::zero);

        Ok(amount_in)
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_quote_exchange(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
        // SBP-M1 review: consider changing to Result<RateBalances<T>, DispatchError> and simply returning error if !can_do_exchange
    ) -> Result<(bool, RateBalances<T>), DispatchError> {
        ensure!(amount > 0, Error::<T>::InsufficientAmount);

        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        let rates = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        let mut exchange_balances = BTreeMap::new();

        let mut can_do_exchange = true;

        // SBP-M1 review: consider grouping each 'phase' of algorithm into blocks: prove, aggregate, burn, transfer, mint
        // RateAction::Has - Prove parties possess non-transferable assets

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Has(op, amount) = asset_rate.action {
                // SBP-M1 review: consider .target_account() helper function on enum
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
                // SBP-M1 review: refactor into helper function - e.g. op.evaluate(balance, amount)
                let amount = match op {
                    AmountOp::Equal => {
                        if balance == amount {
                            amount
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount
                        }
                    }
                    AmountOp::GreaterEqualThan => {
                        if balance >= amount {
                            amount
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount
                        }
                    }
                    AmountOp::GreaterThan => {
                        if balance > amount {
                            amount
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            balance - amount
                        }
                    }
                    AmountOp::LessEqualThan => {
                        if balance <= amount {
                            amount
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            amount - balance
                        }
                    }
                    AmountOp::LessThan => {
                        if balance < amount {
                            amount
                        } else {
                            // SBP-M1 review: return early?
                            can_do_exchange = false;
                            // SBP-M1 review: use safe math
                            amount - balance
                        }
                    }
                };
                exchange_balances.insert(asset_rate.clone(), amount);
            }
        }

        // RateAction::Transfer|Burn - Aggregate transferable prices and balances

        let mut balances: TransactionBalances<T> = BTreeMap::new();

        let mut prices: TransactionBalances<T> = BTreeMap::new();

        let total_amount: i128 = amount.try_into().map_err(|_| Error::<T>::Overflow)?;

        for asset_rate in rates.iter() {
            let balance = match &asset_rate.action {
                RateAction::Transfer(_) | RateAction::Burn(_) => {
                    // SBP-M1 review: refactor into .target_account() helper function on enum
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
                // SBP-M1 review: prefer explict matches, wildcard will match any future added variants
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
                .action
                .get_amount()
                .checked_mul(total_amount)
                .ok_or(Error::<T>::Overflow)?;
            // SBP-M1 review: duplicate code, refactor into reusable function
            // SBP-M1 review: consider BTreeMap.entry() api
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
            // SBP-M1 review: use let-else continue to reduce nesting
            if let RateAction::Burn(_) = &asset_rate.action {
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
                // SBP-M1 review: duplicate code
                if *balance < 0 {
                    // SBP-M1 review: return early
                    can_do_exchange = false;
                    exchange_balances.insert(asset_rate.clone(), *balance);
                } else {
                    exchange_balances.insert(asset_rate.clone(), *price);
                }
            }
        }

        // RateAction::Transfer - Compute total transfers

        for asset_rate in rates.iter() {
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Transfer(_) = &asset_rate.action {
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
                // SBP-M1 review: duplicate code
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
            // SBP-M1 review: use let-else with continue to reduce nesting
            if let RateAction::Mint(_) = &asset_rate.action {
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

        // SBP-M1 review: consider Ok(exchange_balances), returning error early once !can_do_exchange
        Ok((can_do_exchange, exchange_balances))
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_exchange_assets(
        buyer: &T::AccountId,
        market_id: T::MarketId,
        market_rate_id: T::MarketRateId,
        amount: Balance,
    ) -> DispatchResult {
        let market = Markets::<T>::get(market_id).ok_or(Error::<T>::InvalidMarket)?;
        // SBP-M1 review: value not used therefore inefficient, use .contains_key() with ensure!
        let _market_rate = MarketRates::<T>::get((market_id, market_rate_id))
            .ok_or(Error::<T>::InvalidMarketRate)?;

        // SBP-M1 review: move before market rate check to avoid unnecessary read if buyer is owner/vault
        ensure!(*buyer != market.owner, Error::<T>::InvalidBuyer);
        ensure!(*buyer != market.vault, Error::<T>::InvalidBuyer);

        let (can_do_exchange, exchange_balances) =
            Self::do_quote_exchange(buyer, market_id, market_rate_id, amount)?;

        // SBP-M1 review: consider returning an error rather than signalling that caller can not exchange via success attribute of Exchanged event
        if can_do_exchange {
            for (asset_rate, amount) in &exchange_balances {
                let amount: u128 = (*amount).try_into().map_err(|_| Error::<T>::Overflow)?;
                // SBP-M1 review: use .from() helper method on enum
                let from = match &asset_rate.from {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                // SBP-M1 review: use .to() helper method on enum
                let to = match &asset_rate.to {
                    RateAccount::Account(account) => account,
                    RateAccount::Buyer => buyer,
                    RateAccount::Market => &market.vault,
                };
                match asset_rate.action {
                    RateAction::Transfer(_) => sugarfunge_asset::Pallet::<T>::do_transfer_from(
                        &market.owner,
                        from,
                        to,
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount,
                    )?,
                    RateAction::Burn(_) => sugarfunge_asset::Pallet::<T>::do_burn(
                        &market.owner,
                        from,
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount,
                    )?,
                    RateAction::Mint(_) => sugarfunge_asset::Pallet::<T>::do_mint(
                        &market.owner,
                        to,
                        asset_rate.class_id,
                        asset_rate.asset_id,
                        amount,
                    )?,
                    // SBP-M1 review: use actual enum variants, wildcard will match future added variants
                    _ => (),
                }
            }
        }

        let balances = exchange_balances
            .iter()
            .map(|(rate, balance)| RateBalance {
                rate: rate.clone(),
                balance: *balance,
            })
            .collect();

        // SBP-M1 review: consider emitting an error if the exchange could not be performed rather than success: can_do_exchange
        Self::deposit_event(Event::Exchanged {
            // SBP-M1 review: unnecessary clone()
            buyer: buyer.clone(),
            market_id,
            market_rate_id,
            amount,
            balances,
            success: can_do_exchange,
        });

        // SBP-M1 review: .into() unnecessary
        Ok(().into())
    }
}

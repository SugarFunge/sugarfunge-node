#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, ReservableCurrency},
    BoundedVec, PalletId,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Hash},
    RuntimeDebug,
};
use sp_std::{fmt::Debug, prelude::*};
use sugarfunge_primitives::Balance;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type BundleSchema<T> = (
    BoundedVec<<T as sugarfunge_asset::Config>::ClassId, <T as Config>::MaxAssets>,
    BoundedVec<
        BoundedVec<<T as sugarfunge_asset::Config>::AssetId, <T as Config>::MaxAssets>,
        <T as Config>::MaxAssets,
    >,
    BoundedVec<BoundedVec<Balance, <T as Config>::MaxAssets>, <T as Config>::MaxAssets>,
);

pub type BundleId = sugarfunge_primitives::Hash;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub fn argsort<T: Ord>(data: &[T]) -> Vec<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &data[i]);
    indices
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + sugarfunge_escrow::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The minimum balance to create bundle
        #[pallet::constant]
        type CreateBundleDeposit: Get<BalanceOf<Self>>;

        /// Max number of assets in a bundle
        #[pallet::constant]
        type MaxAssets: Get<u32>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn bundles)]
    pub(super) type Bundles<T: Config> =
        StorageMap<_, Blake2_128Concat, BundleId, Bundle<BundleSchema<T>, T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn balances)]
    pub(super) type Balances<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, BundleId>,
        ),
        (T::AccountId, Balance),
        ValueQuery,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // Bundle created who, escrow, balance
        Created(BundleId, T::AccountId, T::AccountId, Balance),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Bundle already exists
        BundleExists,
        /// Number Overflow
        NumOverflow,
        /// Array is of wrong length
        InvalidArrayLength,
        /// Insufficient asset balance
        InsufficientBalance,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Bundle<
    BundleSchema,
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq + TypeInfo,
> {
    /// Creator
    creator: AccountId,
    /// Bundle metadata
    metadata: Vec<u8>,
    /// Schema
    schema: BundleSchema,
}

impl<T: Config> Pallet<T> {
    pub fn do_register_bundle(
        creator: &T::AccountId,
        schema: &BundleSchema<T>,
        metadata: Vec<u8>,
    ) -> Result<BundleId, DispatchError> {
        let bundle_id: BundleId = BlakeTwo256::hash_of(&schema);

        ensure!(
            !Bundles::<T>::contains_key(bundle_id),
            Error::<T>::BundleExists
        );

        Bundles::<T>::insert(
            &bundle_id,
            &Bundle::<BundleSchema<T>, T::AccountId> {
                creator: creator.clone(),
                schema: schema.clone(),
                metadata,
            },
        );

        Ok(bundle_id)
    }

    pub fn do_create_bundles(
        who: &T::AccountId,
        schema: &BundleSchema<T>,
        amount: Balance,
    ) -> Result<(BundleId, T::AccountId), DispatchError> {
        let bundle_id: BundleId = BlakeTwo256::hash_of(&schema);

        let (class_ids, asset_ids, amounts) = schema;
        ensure!(
            class_ids.len() == asset_ids.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        for (class_idx, class_id) in class_ids.iter().enumerate() {
            let balances = sugarfunge_asset::Pallet::<T>::balance_of_single_owner_batch(
                who,
                *class_id,
                asset_ids[class_idx].to_vec(),
            )?;
            let amounts = amounts[class_idx]
                .iter()
                .map(|balance| balance.saturating_mul(amount))
                .collect::<Vec<u128>>();
            for (balance_idx, balance) in balances.iter().enumerate() {
                ensure!(
                    *balance >= amounts[balance_idx],
                    Error::<T>::InsufficientBalance
                );
            }
        }

        if !Bundles::<T>::contains_key(bundle_id) {
            Bundles::<T>::insert(
                &bundle_id,
                &Bundle::<BundleSchema<T>, T::AccountId> {
                    creator: who.clone(),
                    schema: schema.clone(),
                    metadata: vec![],
                },
            );
        }

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account();

        let escrow = sugarfunge_escrow::Pallet::<T>::do_create_escrow(&operator, &operator)?;

        for (idx, class_id) in class_ids.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &who,
                &who,
                &escrow,
                *class_id,
                asset_ids[idx].to_vec(),
                amounts[idx]
                    .iter()
                    .map(|balance| balance.saturating_mul(amount))
                    .collect(),
            )?;
        }

        Balances::<T>::try_mutate((who, bundle_id), |balance| -> DispatchResult {
            balance.0 = escrow.clone();
            balance.1 = balance
                .1
                .checked_add(amount)
                .ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::Created(
            bundle_id,
            who.clone(),
            escrow.clone(),
            amount,
        ));

        Ok((bundle_id, escrow))
    }
}

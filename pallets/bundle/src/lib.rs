#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, Get, ReservableCurrency},
    BoundedVec, PalletId,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, Hash},
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

pub type BundleSchema<T> = (
    BoundedVec<<T as sugarfunge_asset::Config>::ClassId, <T as Config>::MaxAssets>,
    BoundedVec<
        BoundedVec<<T as sugarfunge_asset::Config>::AssetId, <T as Config>::MaxAssets>,
        <T as Config>::MaxAssets,
    >,
    BoundedVec<BoundedVec<Balance, <T as Config>::MaxAssets>, <T as Config>::MaxAssets>,
);

pub type BundleId = sugarfunge_primitives::Hash;

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
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

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
    pub(super) type Bundles<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BundleId,
        Bundle<T::ClassId, T::AssetId, BundleSchema<T>, T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn asset_bundles)]
    pub(super) type AssetBundles<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::AssetId>,
        ),
        BundleId,
    >;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Mint {
            bundle_id: BundleId,
            who: T::AccountId,
            amount: Balance,
        },
        Burn {
            bundle_id: BundleId,
            who: T::AccountId,
            amount: Balance,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Bundle hash does not match bundle id
        InvalidBundleIdForBundle,
        /// Bundle already exists
        BundleExists,
        /// Bundle does not exists
        BundleNotFound,
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
pub struct Bundle<ClassId, AssetId, BundleSchema, AccountId> {
    /// Creator
    creator: AccountId,
    /// IOU asset class
    class_id: ClassId,
    /// IOU asset id
    asset_id: AssetId,
    /// Bundle metadata
    metadata: Vec<u8>,
    /// Schema
    schema: BundleSchema,
    /// Vault
    vault: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_register_bundle(
        creator: &T::AccountId,
        class_id: T::ClassId,
        asset_id: T::AssetId,
        bundle_id: BundleId,
        schema: &BundleSchema<T>,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        ensure!(
            BlakeTwo256::hash_of(&schema) == bundle_id,
            Error::<T>::InvalidBundleIdForBundle
        );

        let bundle_id: BundleId = BlakeTwo256::hash_of(&schema);

        ensure!(
            !Bundles::<T>::contains_key(bundle_id),
            Error::<T>::BundleExists
        );

        let operator = <T as Config>::PalletId::get().into_account();

        sugarfunge_asset::Pallet::<T>::do_create_class(
            &creator,
            &operator,
            class_id,
            metadata.clone(),
        )?;

        let vault: T::AccountId = <T as Config>::PalletId::get().into_sub_account(bundle_id);

        Bundles::<T>::insert(
            &bundle_id,
            &Bundle {
                creator: creator.clone(),
                class_id,
                asset_id,
                schema: schema.clone(),
                metadata,
                vault,
            },
        );

        AssetBundles::<T>::insert((class_id, asset_id), bundle_id);

        Ok(())
    }

    pub fn do_mint_bundles(
        who: &T::AccountId,
        bundle_id: BundleId,
        amount: Balance,
    ) -> DispatchResult {
        let bundle = Bundles::<T>::get(bundle_id).ok_or(Error::<T>::BundleNotFound)?;

        let (class_ids, asset_ids, amounts) = bundle.schema;
        ensure!(
            class_ids.len() == asset_ids.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        // Ensure creator has enough assets to create bundle
        for (class_idx, class_id) in class_ids.iter().enumerate() {
            let balances = sugarfunge_asset::Pallet::<T>::balance_of_single_owner_batch(
                who,
                *class_id,
                asset_ids[class_idx].to_vec(),
            )?;
            let amounts = amounts[class_idx]
                .iter()
                .map(|balance| balance.saturating_mul(amount))
                .collect::<Vec<Balance>>();
            for (balance_idx, balance) in balances.iter().enumerate() {
                ensure!(
                    *balance >= amounts[balance_idx],
                    Error::<T>::InsufficientBalance
                );
            }
        }

        // Transfer creator assets to bundle vault
        for (idx, class_id) in class_ids.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &who,
                &who,
                &bundle.vault,
                *class_id,
                asset_ids[idx].to_vec(),
                amounts[idx]
                    .iter()
                    .map(|balance| balance.saturating_mul(amount))
                    .collect(),
            )?;
        }

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account();

        // Mint IOU assets to creator for each bundle created
        sugarfunge_asset::Pallet::<T>::do_mint(
            &operator,
            who,
            bundle.class_id,
            bundle.asset_id,
            amount,
        )?;

        Self::deposit_event(Event::Mint {
            bundle_id,
            who: who.clone(),
            amount,
        });

        Ok(())
    }

    pub fn do_burn_bundles(
        who: &T::AccountId,
        bundle_id: BundleId,
        amount: Balance,
    ) -> DispatchResult {
        let bundle = Bundles::<T>::get(bundle_id).ok_or(Error::<T>::BundleNotFound)?;

        let (class_ids, asset_ids, amounts) = bundle.schema;
        ensure!(
            class_ids.len() == asset_ids.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        // Ensure enough IOU assets to recover bundle assets
        let iou_balance =
            sugarfunge_asset::Pallet::<T>::balances((who, bundle.class_id, bundle.asset_id));
        ensure!(iou_balance >= amount, Error::<T>::InsufficientBalance);

        // Ensure enough bundle assets in vault to cover IOU
        for (class_idx, class_id) in class_ids.iter().enumerate() {
            let balances = sugarfunge_asset::Pallet::<T>::balance_of_single_owner_batch(
                &bundle.vault,
                *class_id,
                asset_ids[class_idx].to_vec(),
            )?;
            let amounts = amounts[class_idx]
                .iter()
                .map(|balance| balance.saturating_mul(amount))
                .collect::<Vec<Balance>>();
            for (balance_idx, balance) in balances.iter().enumerate() {
                ensure!(
                    *balance >= amounts[balance_idx],
                    Error::<T>::InsufficientBalance
                );
            }
        }

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account();

        // Transfer bundle assets from reserves to IOU owner
        for (idx, class_id) in class_ids.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &operator,
                &operator,
                &who,
                *class_id,
                asset_ids[idx].to_vec(),
                amounts[idx]
                    .iter()
                    .map(|balance| balance.saturating_mul(amount))
                    .collect(),
            )?;
        }

        // Burn IOU assets
        sugarfunge_asset::Pallet::<T>::do_burn(
            &operator,
            who,
            bundle.class_id,
            bundle.asset_id,
            amount,
        )?;

        Self::deposit_event(Event::Burn {
            bundle_id,
            who: who.clone(),
            amount,
        });

        Ok(())
    }
}

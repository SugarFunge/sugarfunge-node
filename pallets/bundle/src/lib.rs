#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
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
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Max number of asset classes and per asset_id in a bundle
        #[pallet::constant]
        type MaxAssets: Get<u32>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    pub type BundleMetadataOf<T> =
        BoundedVec<u8, <T as sugarfunge_asset::Config>::MaxClassMetadata>;
    pub type BundleOf<T> = Bundle<
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
        BundleSchema<T>,
        <T as frame_system::Config>::AccountId,
        BundleMetadataOf<T>,
    >;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn bundles)]
    pub(super) type Bundles<T: Config> = StorageMap<_, Blake2_128Concat, BundleId, BundleOf<T>>;

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
        Register {
            bundle_id: BundleId,
            who: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
        },
        Mint {
            bundle_id: BundleId,
            who: T::AccountId,
            from: T::AccountId,
            to: T::AccountId,
            amount: Balance,
        },
        Burn {
            bundle_id: BundleId,
            who: T::AccountId,
            from: T::AccountId,
            to: T::AccountId,
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
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn register_bundle(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            bundle_id: BundleId,
            schema: BundleSchema<T>,
            metadata: BundleMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_register_bundle(&who, class_id, asset_id, bundle_id, &schema, metadata)?;

            Ok(().into())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn mint_bundle(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            bundle_id: BundleId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_mint_bundles(&who, &from, &to, bundle_id, amount)?;

            Ok(().into())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn burn_bundle(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            bundle_id: BundleId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_burn_bundles(&who, &from, &to, bundle_id, amount)?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Bundle<ClassId, AssetId, BundleSchema, AccountId, BundleMetadataOf> {
    /// Creator
    creator: AccountId,
    /// IOU asset class
    class_id: ClassId,
    /// IOU asset id
    asset_id: AssetId,
    /// Bundle metadata
    metadata: BundleMetadataOf,
    /// Schema
    schema: BundleSchema,
    /// Vault
    vault: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_register_bundle(
        who: &T::AccountId,
        class_id: T::ClassId,
        asset_id: T::AssetId,
        bundle_id: BundleId,
        schema: &BundleSchema<T>,
        metadata: BundleMetadataOf<T>,
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

        let operator = <T as Config>::PalletId::get().into_account_truncating();

        sugarfunge_asset::Pallet::<T>::do_create_class(
            &who,
            &operator,
            class_id,
            metadata.clone(),
        )?;

        let vault: T::AccountId =
            <T as Config>::PalletId::get().into_sub_account_truncating(bundle_id);

        Bundles::<T>::insert(
            &bundle_id,
            &BundleOf::<T> {
                creator: who.clone(),
                class_id,
                asset_id,
                schema: schema.clone(),
                metadata,
                vault,
            },
        );

        AssetBundles::<T>::insert((class_id, asset_id), bundle_id);

        Self::deposit_event(Event::Register {
            bundle_id,
            who: who.clone(),
            class_id,
            asset_id,
        });

        Ok(())
    }

    pub fn do_mint_bundles(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
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

        // Ensure from has enough assets to create bundle
        for (class_idx, class_id) in class_ids.iter().enumerate() {
            let balances = sugarfunge_asset::Pallet::<T>::balance_of_single_owner_batch(
                from,
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

        // Transfer from assets to bundle vault
        for (idx, class_id) in class_ids.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &who,
                &from,
                &bundle.vault,
                *class_id,
                asset_ids[idx].to_vec(),
                amounts[idx]
                    .iter()
                    .map(|balance| balance.saturating_mul(amount))
                    .collect(),
            )?;
        }

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Mint IOU assets for each bundle created
        sugarfunge_asset::Pallet::<T>::do_mint(
            &operator,
            to,
            bundle.class_id,
            bundle.asset_id,
            amount,
        )?;

        Self::deposit_event(Event::Mint {
            bundle_id,
            who: who.clone(),
            from: from.clone(),
            to: to.clone(),
            amount,
        });

        Ok(())
    }

    pub fn do_burn_bundles(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
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
            sugarfunge_asset::Pallet::<T>::balances((from, bundle.class_id, bundle.asset_id));
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

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Transfer bundle assets from reserves to IOU owner
        for (idx, class_id) in class_ids.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &operator,
                &operator,
                &to,
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
            from: from.clone(),
            to: to.clone(),
            amount,
        });

        Ok(())
    }
}

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, ReservableCurrency},
    BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, One, Zero},
    RuntimeDebug,
};
use sp_std::prelude::*;
use sugarfunge_primitives::Balance;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Class<AccountId, ClassMetadataOf> {
    owner: AccountId,
    metadata: ClassMetadataOf,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Asset<ClassId, AccountId, AssetMetadataOf> {
    class_id: ClassId,
    creator: AccountId,
    metadata: AssetMetadataOf,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The minimum balance to create class
        #[pallet::constant]
        type CreateAssetClassDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type ClassId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        type AssetId: Member
            + Parameter
            + HasCompact
            + AtLeast32BitUnsigned
            + MaybeSerializeDeserialize
            + Default
            + Copy
            + From<u64>
            + Into<u64>
            + MaxEncodedLen;

        #[pallet::constant]
        type MaxClassMetadata: Get<u32>;

        #[pallet::constant]
        type MaxAssetMetadata: Get<u32>;
    }

    pub type ClassMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxClassMetadata>;
    pub type ClassOf<T> = Class<<T as frame_system::Config>::AccountId, ClassMetadataOf<T>>;

    pub type AssetMetadataOf<T> = BoundedVec<u8, <T as Config>::MaxAssetMetadata>;
    pub type AssetOf<T> =
        Asset<<T as Config>::ClassId, <T as frame_system::Config>::AccountId, AssetMetadataOf<T>>;

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Classes<T: Config> = StorageMap<_, Blake2_128Concat, T::ClassId, ClassOf<T>>;

    #[pallet::storage]
    pub(super) type Assets<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, T::ClassId, Blake2_128, T::AssetId, AssetOf<T>>;

    #[pallet::storage]
    #[pallet::getter(fn asset_count)]
    pub(super) type AssetCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ClassId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn balances)]
    pub(super) type Balances<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>,
            NMapKey<Blake2_128Concat, T::ClassId>,
            NMapKey<Blake2_128Concat, T::AssetId>,
        ),
        Balance,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClassCreated {
            class_id: T::ClassId,
            who: T::AccountId,
        },
        AssetCreated {
            class_id: T::ClassId,
            asset_id: T::AssetId,
            who: T::AccountId,
        },
        AssetMetadataUpdated {
            class_id: T::ClassId,
            asset_id: T::AssetId,
            who: T::AccountId,
            metadata: Vec<u8>,
        },
        Mint {
            who: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        },
        BatchMint {
            who: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        },
        Burn {
            who: T::AccountId,
            from: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        },
        BatchBurn {
            who: T::AccountId,
            from: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        },
        Transferred {
            who: T::AccountId,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        },
        BatchTransferred {
            who: T::AccountId,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        },
        OperatorApprovalForAll {
            who: T::AccountId,
            operator: T::AccountId,
            class_id: T::ClassId,
            approved: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        Unknown,
        InUse,
        InvalidAssetId,
        InsufficientBalance,
        NumOverflow,
        InvalidArrayLength,
        Overflow,
        InvalidClassId,
        NoPermission,
        ClassNotFound,
        AssetNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_class(
            origin: OriginFor<T>,
            owner: T::AccountId,
            class_id: T::ClassId,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_class(&who, &owner, class_id, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_asset(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            metadata: AssetMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_create_asset(&who, class_id, asset_id, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_transfer_from(&who, &from, &to, class_id, asset_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_batch_transfer_from(&who, &from, &to, class_id, asset_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_mint(&who, &to, class_id, asset_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_batch_mint(&who, &to, class_id, asset_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_burn(&who, &from, class_id, asset_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::maybe_check_owner(&who, class_id)?;

            Self::do_batch_burn(&who, &from, class_id, asset_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn update_class_metadata(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_update_class_metadata(&who, class_id, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn update_asset_metadata(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            metadata: AssetMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_update_asset_metadata(&who, class_id, asset_id, metadata)?;

            Ok(().into())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_create_class(
            who: &T::AccountId,
            owner: &T::AccountId,
            class_id: T::ClassId,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResult {
            ensure!(
                !Classes::<T>::contains_key(class_id),
                Error::<T>::InvalidClassId
            );

            let deposit = T::CreateAssetClassDeposit::get();
            T::Currency::reserve(&who, deposit.clone())?;

            let class = ClassOf::<T> {
                owner: owner.clone(),
                metadata,
            };

            Classes::<T>::insert(class_id, class);

            Self::deposit_event(Event::ClassCreated {
                class_id,
                who: who.clone(),
            });

            Ok(())
        }

        pub fn do_create_asset(
            who: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            metadata: AssetMetadataOf<T>,
        ) -> DispatchResult {
            Self::maybe_check_owner(who, class_id)?;

            ensure!(
                !Assets::<T>::contains_key(class_id, asset_id),
                Error::<T>::InUse
            );

            Assets::<T>::insert(
                class_id,
                asset_id,
                AssetOf::<T> {
                    class_id,
                    creator: who.clone(),
                    metadata,
                },
            );

            AssetCount::<T>::try_mutate(class_id, |count| -> DispatchResult {
                *count = count
                    .checked_add(One::one())
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::AssetCreated {
                class_id,
                asset_id,
                who: who.clone(),
            });

            Ok(())
        }

        pub fn do_update_class_metadata(
            who: &T::AccountId,
            class_id: T::ClassId,
            metadata: ClassMetadataOf<T>,
        ) -> DispatchResult {
            Self::maybe_check_owner(who, class_id)?;
            ensure!(
                Classes::<T>::contains_key(class_id),
                Error::<T>::InvalidClassId
            );
            Classes::<T>::try_mutate(class_id, |class| -> DispatchResult {
                if let Some(class) = class {
                    class.metadata = metadata.clone();
                }
                Ok(())
            })?;
            Ok(())
        }

        pub fn do_update_asset_metadata(
            who: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            metadata: AssetMetadataOf<T>,
        ) -> DispatchResult {
            Self::maybe_check_owner(who, class_id)?;
            ensure!(
                Assets::<T>::contains_key(class_id, asset_id),
                Error::<T>::InvalidAssetId
            );
            Assets::<T>::try_mutate(class_id, asset_id, |asset| -> DispatchResult {
                if let Some(asset) = asset {
                    asset.metadata = metadata.clone();
                }
                Ok(())
            })?;

            Self::deposit_event(Event::AssetMetadataUpdated {
                class_id,
                asset_id,
                who: who.clone(),
                metadata: metadata.to_vec(),
            });

            Ok(())
        }

        pub fn class_exists(class_id: T::ClassId) -> bool {
            Classes::<T>::contains_key(class_id)
        }

        pub fn asset_exists(class_id: T::ClassId, asset_id: T::AssetId) -> bool {
            Assets::<T>::contains_key(class_id, asset_id)
        }

        pub fn do_mint(
            who: &T::AccountId,
            to: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResult {
            Self::add_balance_to(to, class_id, asset_id, amount)?;

            Self::deposit_event(Event::Mint {
                who: who.clone(),
                to: to.clone(),
                class_id,
                asset_id,
                amount,
            });

            Ok(())
        }

        pub fn do_batch_mint(
            who: &T::AccountId,
            to: &T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResult {
            ensure!(
                asset_ids.len() == amounts.len(),
                Error::<T>::InvalidArrayLength
            );

            let n = asset_ids.len();
            for i in 0..n {
                let asset_id = asset_ids[i];
                let amount = amounts[i];
                Self::add_balance_to(to, class_id, asset_id, amount)?;
            }

            Self::deposit_event(Event::BatchMint {
                who: who.clone(),
                to: to.clone(),
                class_id,
                asset_ids,
                amounts,
            });

            Ok(())
        }

        pub fn do_burn(
            who: &T::AccountId,
            from: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResult {
            Self::remove_balance_from(from, class_id, asset_id, amount)?;

            Self::deposit_event(Event::Burn {
                who: who.clone(),
                from: from.clone(),
                class_id,
                asset_id,
                amount,
            });

            Ok(())
        }

        pub fn do_batch_burn(
            who: &T::AccountId,
            from: &T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResult {
            ensure!(
                asset_ids.len() == amounts.len(),
                Error::<T>::InvalidArrayLength
            );

            let n = asset_ids.len();
            for i in 0..n {
                let asset_id = asset_ids[i];
                let amount = amounts[i];

                Self::remove_balance_from(from, class_id, asset_id, amount)?;
            }

            Self::deposit_event(Event::BatchBurn {
                who: who.clone(),
                from: from.clone(),
                class_id,
                asset_ids,
                amounts,
            });

            Ok(())
        }

        pub fn do_transfer_from(
            who: &T::AccountId,
            from: &T::AccountId,
            to: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResult {
            if from == to || amount == Zero::zero() {
                return Ok(());
            }

            Self::remove_balance_from(from, class_id, asset_id, amount)?;

            Self::add_balance_to(to, class_id, asset_id, amount)?;

            Self::deposit_event(Event::Transferred {
                who: who.clone(),
                from: from.clone(),
                to: to.clone(),
                class_id,
                asset_id,
                amount,
            });

            Ok(())
        }

        pub fn do_batch_transfer_from(
            who: &T::AccountId,
            from: &T::AccountId,
            to: &T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
            amounts: Vec<Balance>,
        ) -> DispatchResult {
            if from == to {
                return Ok(());
            }

            ensure!(
                asset_ids.len() == amounts.len(),
                Error::<T>::InvalidArrayLength
            );

            let n = asset_ids.len();
            for i in 0..n {
                let asset_id = asset_ids[i];
                let amount = amounts[i];

                Self::remove_balance_from(from, class_id, asset_id, amount)?;

                Self::add_balance_to(to, class_id, asset_id, amount)?;
            }

            Self::deposit_event(Event::BatchTransferred {
                who: who.clone(),
                from: from.clone(),
                to: to.clone(),
                class_id,
                asset_ids,
                amounts,
            });

            Ok(())
        }

        pub fn balance_of(
            owner: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
        ) -> Balance {
            Self::balances((owner, class_id, asset_id))
        }

        pub fn balance_of_batch(
            owners: &Vec<T::AccountId>,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
        ) -> Result<Vec<Balance>, DispatchError> {
            ensure!(
                owners.len() == asset_ids.len(),
                Error::<T>::InvalidArrayLength
            );

            let mut batch_balances = Vec::new();

            for _i in 0..owners.len() {
                batch_balances.push(Balance::from(0u32));
            }

            let n = owners.len();
            for i in 0..n {
                let owner = &owners[i];
                let asset_id = asset_ids[i];
                batch_balances[i] = Self::balances((owner, class_id, asset_id));
            }

            Ok(batch_balances)
        }

        pub fn balance_of_single_owner_batch(
            owner: &T::AccountId,
            class_id: T::ClassId,
            asset_ids: Vec<T::AssetId>,
        ) -> Result<Vec<Balance>, DispatchError> {
            let mut batch_balances = Vec::new();

            for _i in 0..asset_ids.len() {
                batch_balances.push(Balance::from(0u32));
            }

            let n = asset_ids.len();
            for i in 0..n {
                let owner = owner.clone();
                let asset_id = asset_ids[i];

                batch_balances[i] = Self::balances((owner, class_id, asset_id));
            }

            Ok(batch_balances)
        }

        pub fn balances_of_owner(
            owner: &T::AccountId,
        ) -> Result<Vec<(T::ClassId, T::AssetId, Balance)>, DispatchError> {
            let mut balances = Vec::new();
            let assets = Balances::<T>::iter_key_prefix((owner,));
            for (class_id, asset_id) in assets {
                let balance = Balances::<T>::get((owner, class_id, asset_id));
                balances.push((class_id, asset_id, balance));
            }
            Ok(balances)
        }

        pub fn class_balances_of_owner(
            owner: &T::AccountId,
            class_id: T::ClassId,
        ) -> Result<Vec<(T::AssetId, Balance)>, DispatchError> {
            let mut balances = Vec::new();
            let assets = Balances::<T>::iter_key_prefix((owner, class_id));
            for asset_id in assets {
                let balance = Balances::<T>::get((owner, class_id, asset_id));
                balances.push((asset_id, balance));
            }
            Ok(balances)
        }

        fn add_balance_to(
            to: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResult {
            Balances::<T>::try_mutate((to, class_id, asset_id), |balance| -> DispatchResult {
                *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Ok(())
        }

        fn remove_balance_from(
            from: &T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            amount: Balance,
        ) -> DispatchResult {
            Balances::<T>::try_mutate((from, class_id, asset_id), |balance| -> DispatchResult {
                *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Ok(())
        }

        fn maybe_check_owner(who: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
            let class = Classes::<T>::get(class_id).ok_or(Error::<T>::InvalidClassId)?;
            ensure!(*who == class.owner, Error::<T>::NoPermission);
            Ok(())
        }
    }
}

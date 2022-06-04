#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency},
    PalletId,
};
use scale_info::TypeInfo;
use sp_arithmetic::traits::UniqueSaturatedInto;
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

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

        type PalletId: Get<PalletId>;

        /// Max number of owners
        #[pallet::constant]
        type MaxOwners: Get<u32>;

        /// The minimum balance to create bag
        #[pallet::constant]
        type CreateBagDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Bags<T: Config> =
        StorageMap<_, Blake2_128, T::ClassId, Bag<T::AccountId, T::ClassId>>;

    #[pallet::storage]
    pub(super) type BagAccounts<T: Config> =
        StorageMap<_, Blake2_128, T::AccountId, BagAccount<T::AccountId, T::ClassId, T::AssetId>>;

    #[pallet::storage]
    pub(super) type NextBagId<T: Config> = StorageMap<_, Blake2_128, T::ClassId, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Register {
            who: T::AccountId,
            class_id: T::ClassId,
        },
        Created {
            bag: T::AccountId,
            who: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            owners: Vec<T::AccountId>,
        },
        Deposit {
            bag: T::AccountId,
            who: T::AccountId,
        },
        Sweep {
            bag: T::AccountId,
            who: T::AccountId,
            to: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        BagClassExists,
        BagAccountExists,
        InvalidBagClass,
        InvalidBagAccount,
        InvalidBagOperator,
        InvalidBagOwner,
        InvalidArrayLength,
        InsufficientShares,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn register(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            metadata: sugarfunge_asset::ClassMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_register(&who, class_id, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            owners: Vec<T::AccountId>,
            shares: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create(&who, class_id, &owners, &shares)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn deposit(
            origin: OriginFor<T>,
            bag: T::AccountId,
            class_ids: Vec<T::ClassId>,
            asset_ids: Vec<Vec<T::AssetId>>,
            amounts: Vec<Vec<Balance>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_deposit(&who, &bag, class_ids, asset_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn sweep(
            origin: OriginFor<T>,
            to: T::AccountId,
            bag: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_sweep(&who, &to, &bag)?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Bag<AccountId, ClassId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting claims
    pub class_id: ClassId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BagAccount<AccountId, ClassId, AssetId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting shares
    pub class_id: ClassId,
    /// The asset_id for minting shares
    pub asset_id: AssetId,
    /// Total number of shares
    pub total_shares: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn do_register(
        who: &T::AccountId,
        class_id: T::ClassId,
        metadata: sugarfunge_asset::ClassMetadataOf<T>,
    ) -> DispatchResult {
        ensure!(
            !Bags::<T>::contains_key(&class_id),
            Error::<T>::BagClassExists
        );

        let owner = <T as Config>::PalletId::get().into_account_truncating();
        sugarfunge_asset::Pallet::<T>::do_create_class(&who, &owner, class_id, metadata.clone())?;

        let bag = Bag {
            operator: who.clone(),
            class_id,
        };

        Bags::<T>::insert(class_id, &bag);

        Self::deposit_event(Event::Register {
            who: who.clone(),
            class_id,
        });

        Ok(())
    }

    pub fn do_create(
        who: &T::AccountId,
        class_id: T::ClassId,
        owners: &Vec<T::AccountId>,
        shares: &Vec<Balance>,
    ) -> Result<T::AccountId, DispatchError> {
        ensure!(
            Bags::<T>::contains_key(&class_id),
            Error::<T>::InvalidBagClass
        );

        ensure!(owners.len() == shares.len(), Error::<T>::InvalidArrayLength);

        let bag_id = NextBagId::<T>::try_mutate(&class_id, |id| -> Result<u64, DispatchError> {
            let current_id = *id;
            *id = *id + 1;
            Ok(current_id)
        })?;

        let block_number: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number as u64, class_id.into(), bag_id];
        let bag = <T as Config>::PalletId::get().into_sub_account_truncating(sub);

        ensure!(
            !BagAccounts::<T>::contains_key(&bag),
            Error::<T>::BagAccountExists
        );

        let deposit = T::CreateBagDeposit::get();
        <T as Config>::Currency::transfer(who, &bag, deposit, AllowDeath)?;

        let asset_id: T::AssetId = bag_id.into();

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Mint shares for each owner
        for (idx, owner) in owners.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_mint(
                &operator,
                &owner,
                class_id,
                asset_id,
                shares[idx],
            )?;
        }

        let new_bag = BagAccount {
            operator: operator.clone(),
            class_id,
            asset_id,
            total_shares: shares.iter().sum(),
        };

        BagAccounts::<T>::insert(&bag, &new_bag);

        Self::deposit_event(Event::Created {
            bag: bag.clone(),
            who: who.clone(),
            class_id,
            asset_id,
            owners: owners.clone(),
        });

        Ok(bag.clone())
    }

    pub fn do_deposit(
        who: &T::AccountId,
        bag: &T::AccountId,
        class_ids: Vec<T::ClassId>,
        asset_ids: Vec<Vec<T::AssetId>>,
        amounts: Vec<Vec<Balance>>,
    ) -> DispatchResult {
        ensure!(
            BagAccounts::<T>::contains_key(&bag),
            Error::<T>::InvalidBagAccount
        );

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
                asset_ids[idx].len() == amounts[idx].len(),
                Error::<T>::InvalidArrayLength
            );

            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &who,
                &who,
                &bag,
                *class_id,
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Deposit {
            bag: bag.clone(),
            who: who.clone(),
        });

        Ok(().into())
    }

    pub fn do_sweep(
        who: &T::AccountId,
        to: &T::AccountId,
        bag: &T::AccountId,
    ) -> Result<(Vec<T::ClassId>, Vec<Vec<T::AssetId>>, Vec<Vec<Balance>>), DispatchError> {
        let bag_info = BagAccounts::<T>::get(bag).ok_or(Error::<T>::InvalidBagAccount)?;

        let shares =
            sugarfunge_asset::Pallet::<T>::balance_of(who, bag_info.class_id, bag_info.asset_id);
        ensure!(
            shares == bag_info.total_shares,
            Error::<T>::InsufficientShares
        );

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Burn bag shares
        sugarfunge_asset::Pallet::<T>::do_burn(
            &operator,
            who,
            bag_info.class_id,
            bag_info.asset_id,
            bag_info.total_shares,
        )?;

        let balances = sugarfunge_asset::Pallet::<T>::balances_of_owner(&bag)?;
        let balances = balances.iter().fold(
            (
                Vec::<T::ClassId>::new(),
                Vec::<Vec<T::AssetId>>::new(),
                Vec::<Vec<Balance>>::new(),
            ),
            |(mut class_ids, mut asset_ids, mut balances), (class_id, asset_id, balance)| {
                let class_idx = if let Some(class_idx) =
                    class_ids.iter().position(|class| *class == *class_id)
                {
                    class_idx
                } else {
                    let class_idx = class_ids.len();
                    class_ids.push(*class_id);
                    class_idx
                };
                if asset_ids.len() <= class_idx {
                    asset_ids.resize(class_idx + 1, vec![]);
                }
                asset_ids[class_idx].push(*asset_id);
                if balances.len() <= class_idx {
                    balances.resize(class_idx + 1, vec![]);
                }
                balances[class_idx].push(*balance);
                (class_ids, asset_ids, balances)
            },
        );

        for (idx, class_id) in balances.0.iter().enumerate() {
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                &bag,
                &bag,
                to,
                *class_id,
                balances.1[idx].clone(),
                balances.2[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Sweep {
            bag: bag.clone(),
            who: who.clone(),
            to: to.clone(),
        });

        Ok(balances)
    }
}

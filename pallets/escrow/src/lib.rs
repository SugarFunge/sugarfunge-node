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

        /// The minimum balance to create escrow
        #[pallet::constant]
        type CreateEscrowDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Escrows<T: Config> =
        StorageMap<_, Blake2_128, T::AccountId, Escrow<T::AccountId>>;

    #[pallet::storage]
    pub(super) type NextEscrowId<T: Config> =
        StorageMap<_, Blake2_128, (T::ClassId, T::AssetId), u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created {
            escrow: T::AccountId,
            operator: T::AccountId,
            owner: T::AccountId,
        },
        Deposit {
            escrow: T::AccountId,
            operator: T::AccountId,
            owner: T::AccountId,
        },
        Refund {
            escrow: T::AccountId,
            operator: T::AccountId,
            owner: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
        EscrowAccountExists,
        InvalidEscrowAccount,
        InvalidEscrowOperator,
        InvalidEscrowOwner,
        InvalidArrayLength,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_escrow(
            origin: OriginFor<T>,
            owner: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_escrow(&who, &owner, class_id, asset_id)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn deposit_assets(
            origin: OriginFor<T>,
            escrow: T::AccountId,
            class_ids: Vec<T::ClassId>,
            asset_ids: Vec<Vec<T::AssetId>>,
            amounts: Vec<Vec<Balance>>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_deposit_assets(&who, &escrow, class_ids, asset_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn refund_assets(
            origin: OriginFor<T>,
            escrow: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_refund_assets(&who, &escrow)?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Escrow<AccountId> {
    /// The operator of the escrow
    pub operator: AccountId,
    /// The owner of the assets
    pub owner: AccountId,
}

impl<T: Config> Pallet<T> {
    pub fn do_create_escrow(
        operator: &T::AccountId,
        owner: &T::AccountId,
        class_id: T::ClassId,
        asset_id: T::AssetId,
    ) -> Result<T::AccountId, DispatchError> {
        let next_id = NextEscrowId::<T>::try_mutate(
            (class_id, asset_id),
            |id| -> Result<u32, DispatchError> {
                let current_id = *id;
                *id = *id + 1;
                Ok(current_id)
            },
        )?;

        let block_number: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number as u64, class_id.into(), next_id as u64];
        let escrow = <T as Config>::PalletId::get().into_sub_account(sub);

        ensure!(
            !Escrows::<T>::contains_key(&escrow),
            Error::<T>::EscrowAccountExists
        );

        let deposit = T::CreateEscrowDeposit::get();
        <T as Config>::Currency::transfer(operator, &escrow, deposit, AllowDeath)?;

        let new_escrow = Escrow {
            operator: operator.clone(),
            owner: owner.clone(),
        };

        Escrows::<T>::insert(&escrow, new_escrow);

        Self::deposit_event(Event::Created {
            escrow: escrow.clone(),
            operator: operator.clone(),
            owner: owner.clone(),
        });

        Ok(escrow.clone())
    }

    pub fn do_deposit_assets(
        who: &T::AccountId,
        escrow: &T::AccountId,
        class_ids: Vec<T::ClassId>,
        asset_ids: Vec<Vec<T::AssetId>>,
        amounts: Vec<Vec<Balance>>,
    ) -> DispatchResult {
        let escrow_info = Escrows::<T>::get(escrow).ok_or(Error::<T>::InvalidEscrowAccount)?;

        ensure!(escrow_info.owner == *who, Error::<T>::InvalidEscrowOwner);
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
                &escrow,
                *class_id,
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Deposit {
            escrow: escrow.clone(),
            operator: escrow_info.operator.clone(),
            owner: escrow_info.owner.clone(),
        });

        Ok(().into())
    }

    pub fn do_refund_assets(
        who: &T::AccountId,
        escrow: &T::AccountId,
    ) -> Result<(Vec<T::ClassId>, Vec<Vec<T::AssetId>>, Vec<Vec<Balance>>), DispatchError> {
        let escrow_info = Escrows::<T>::get(escrow).ok_or(Error::<T>::InvalidEscrowAccount)?;

        ensure!(
            escrow_info.operator == *who,
            Error::<T>::InvalidEscrowOperator
        );

        let balances = sugarfunge_asset::Pallet::<T>::balances_of_owner(&escrow)?;
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
                &escrow,
                &escrow,
                &escrow_info.owner,
                *class_id,
                balances.1[idx].clone(),
                balances.2[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Refund {
            escrow: escrow.clone(),
            operator: escrow_info.operator.clone(),
            owner: escrow_info.owner.clone(),
        });

        Ok(balances)
    }
}

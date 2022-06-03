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
        StorageMap<_, Blake2_128, T::ClassId, Escrow<T::AccountId, T::ClassId>>;

    #[pallet::storage]
    pub(super) type EscrowAccounts<T: Config> = StorageMap<
        _,
        Blake2_128,
        T::AccountId,
        EscrowAccount<T::AccountId, T::ClassId, T::AssetId>,
    >;

    #[pallet::storage]
    pub(super) type NextEscrowId<T: Config> =
        StorageMap<_, Blake2_128, T::ClassId, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Register {
            who: T::AccountId,
            class_id: T::ClassId,
        },
        AccountCreated {
            escrow: T::AccountId,
            who: T::AccountId,
            class_id: T::ClassId,
            asset_id: T::AssetId,
            owners: Vec<T::AccountId>,
        },
        Deposit {
            escrow: T::AccountId,
            who: T::AccountId,
        },
        Sweep {
            escrow: T::AccountId,
            who: T::AccountId,
            to: T::AccountId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        EscrowClassExists,
        EscrowAccountExists,
        InvalidEscrowClass,
        InvalidEscrowAccount,
        InvalidEscrowOperator,
        InvalidEscrowOwner,
        InvalidArrayLength,
        InsufficientShares,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn register_escrow(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            metadata: sugarfunge_asset::ClassMetadataOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_register_escrow(&who, class_id, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_account(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            owners: Vec<T::AccountId>,
            shares: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_account(&who, class_id, &owners, &shares)?;

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
        pub fn sweep_assets(
            origin: OriginFor<T>,
            to: T::AccountId,
            escrow: T::AccountId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_sweep_assets(&who, &to, &escrow)?;

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Escrow<AccountId, ClassId> {
    /// The operator of the escrow
    pub operator: AccountId,
    /// The class_id for minting claims
    pub class_id: ClassId,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct EscrowAccount<AccountId, ClassId, AssetId> {
    /// The operator of the escrow
    pub operator: AccountId,
    /// The class_id for minting shares
    pub class_id: ClassId,
    /// The asset_id for minting shares
    pub asset_id: AssetId,
    /// Total number of shares
    pub total_shares: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn do_register_escrow(
        who: &T::AccountId,
        class_id: T::ClassId,
        metadata: sugarfunge_asset::ClassMetadataOf<T>,
    ) -> DispatchResult {
        ensure!(
            !Escrows::<T>::contains_key(&class_id),
            Error::<T>::EscrowClassExists
        );

        let owner = <T as Config>::PalletId::get().into_account_truncating();
        sugarfunge_asset::Pallet::<T>::do_create_class(&who, &owner, class_id, metadata.clone())?;

        let escrow = Escrow {
            operator: who.clone(),
            class_id,
        };

        Escrows::<T>::insert(class_id, &escrow);

        Self::deposit_event(Event::Register {
            who: who.clone(),
            class_id,
        });

        Ok(())
    }

    pub fn do_create_account(
        who: &T::AccountId,
        class_id: T::ClassId,
        owners: &Vec<T::AccountId>,
        shares: &Vec<Balance>,
    ) -> Result<T::AccountId, DispatchError> {
        ensure!(
            Escrows::<T>::contains_key(&class_id),
            Error::<T>::InvalidEscrowClass
        );

        ensure!(owners.len() == shares.len(), Error::<T>::InvalidArrayLength);

        let escrow_id =
            NextEscrowId::<T>::try_mutate(&class_id, |id| -> Result<u64, DispatchError> {
                let current_id = *id;
                *id = *id + 1;
                Ok(current_id)
            })?;

        let block_number: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number as u64, class_id.into(), escrow_id];
        let escrow = <T as Config>::PalletId::get().into_sub_account_truncating(sub);

        ensure!(
            !EscrowAccounts::<T>::contains_key(&escrow),
            Error::<T>::EscrowAccountExists
        );

        let deposit = T::CreateEscrowDeposit::get();
        <T as Config>::Currency::transfer(who, &escrow, deposit, AllowDeath)?;

        let asset_id: T::AssetId = escrow_id.into();

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

        let new_escrow = EscrowAccount {
            operator: operator.clone(),
            class_id,
            asset_id,
            total_shares: shares.iter().sum(),
        };

        EscrowAccounts::<T>::insert(&escrow, &new_escrow);

        Self::deposit_event(Event::AccountCreated {
            escrow: escrow.clone(),
            who: who.clone(),
            class_id,
            asset_id,
            owners: owners.clone(),
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
        ensure!(
            EscrowAccounts::<T>::contains_key(&escrow),
            Error::<T>::InvalidEscrowAccount
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
                &escrow,
                *class_id,
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Deposit {
            escrow: escrow.clone(),
            who: who.clone(),
        });

        Ok(().into())
    }

    pub fn do_sweep_assets(
        who: &T::AccountId,
        to: &T::AccountId,
        escrow: &T::AccountId,
    ) -> Result<(Vec<T::ClassId>, Vec<Vec<T::AssetId>>, Vec<Vec<Balance>>), DispatchError> {
        let escrow_info =
            EscrowAccounts::<T>::get(escrow).ok_or(Error::<T>::InvalidEscrowAccount)?;

        let shares = sugarfunge_asset::Pallet::<T>::balance_of(
            who,
            escrow_info.class_id,
            escrow_info.asset_id,
        );
        ensure!(
            shares == escrow_info.total_shares,
            Error::<T>::InsufficientShares
        );

        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Burn escrow shares
        sugarfunge_asset::Pallet::<T>::do_burn(
            &operator,
            who,
            escrow_info.class_id,
            escrow_info.asset_id,
            escrow_info.total_shares,
        )?;

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
                to,
                *class_id,
                balances.1[idx].clone(),
                balances.2[idx].clone(),
            )?;
        }

        Self::deposit_event(Event::Sweep {
            escrow: escrow.clone(),
            who: who.clone(),
            to: to.clone(),
        });

        Ok(balances)
    }
}

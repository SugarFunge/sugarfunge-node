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

// SBP-M1 review: move within pallet module as only used there
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    // SBP-M1 review: remove template comment
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    // SBP-M1 review: loose pallet coupling preferable, can sugarfunge_asset dependency be brought in via trait (associated type on this pallets Config trait)?
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        // SBP-M1 review: add doc comment
        type PalletId: Get<PalletId>;

        /// Max number of owners
        #[pallet::constant]
        type MaxOwners: Get<u32>;

        /// The minimum balance to create bag
        #[pallet::constant]
        type CreateBagDeposit: Get<BalanceOf<Self>>;

        // SBP-M1 review: add doc comment
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type BagClasses<T: Config> =
        StorageMap<_, Blake2_128, T::ClassId, BagClass<T::AccountId, T::ClassId>>;

    #[pallet::storage]
    pub(super) type Bags<T: Config> =
        StorageMap<_, Blake2_128, T::AccountId, Bag<T::AccountId, T::ClassId, T::AssetId>>;

    #[pallet::storage]
    pub(super) type NextBagId<T: Config> = StorageMap<_, Blake2_128, T::ClassId, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    // SBP-M1 review: add doc comments
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

    // SBP-M1 review: remove template comment
    // Errors inform users that something went wrong.
    #[pallet::error]
    // SBP-M1 review: add doc comments
    pub enum Error<T> {
        BagClassExists,
        BagExists,
        InvalidBagClass,
        InvalidBag,
        InvalidBagOperator,
        InvalidBagOwner,
        InvalidArrayLength,
        InsufficientShares,
    }

    // SBP-M1 review: remove template comment
    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // SBP-M1 review: add doc comments
        #[pallet::call_index(0)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn register(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            // SBP-M1 review: redefine this type on this pallets Config to avoid tight pallet coupling - runtime can reuse same types to configure as appropriate
            metadata: sugarfunge_asset::ClassMetadataOf<T>,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_register(..)
            Self::do_register(&who, class_id, metadata)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: whilst a deposit is taken for a bag, anyone can call this to create bags and mint tokens. Consider whether this should perhaps be limited to the class owner or governance?
        // SBP-M1 review: add doc comments
        // SBP-M1 review: register(..) and create(..) names too similar, consider improving naming to be more explicit
        #[pallet::call_index(1)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn create(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            // SBP-M1 review: use bounded vectors
            // SBP-M1 review: use tuple - e.g. `shares: BoundedVec<(T::AccountId, T::Balance), T::MaxShares>`
            owners: Vec<T::AccountId>,
            // SBP-M1 review: consider the regulatory impact of the term shares
            shares: Vec<Balance>,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_create(..)
            Self::do_create(&who, class_id, &owners, &shares)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: add doc comments
        #[pallet::call_index(2)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn deposit(
            origin: OriginFor<T>,
            bag: T::AccountId,
            // SBP-M1 review: use bounded vectors
            // SBP-M1 review: use tuple to collect corresponding values together - e.g. BoundedVec<(T::ClassId, BoundedVec<(T::AssetId, T::Balance), Max>), Max>
            class_ids: Vec<T::ClassId>,
            asset_ids: Vec<Vec<T::AssetId>>,
            amounts: Vec<Vec<Balance>>,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_deposit(..)
            Self::do_deposit(&who, &bag, class_ids, asset_ids, amounts)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }

        // SBP-M1 review: add doc comments
        #[pallet::call_index(3)]
        // SBP-M1 review: implement benchmark and use resulting weight function
        // SBP-M1 review: unnecessary cast
        #[pallet::weight(Weight::from_parts(10_000 as u64, 0))]
        pub fn sweep(
            origin: OriginFor<T>,
            // SBP-M1 review: consider reordering so 'to' is last (from > to)
            to: T::AccountId,
            bag: T::AccountId,
            // SBP-M1 review: can just be DispatchResult as no PostDispatchInfo used
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            // SBP-M1 review: could simplify and just return value from Self::do_sweep(..)
            Self::do_sweep(&who, &to, &bag)?;

            // SBP-M1 review: unnecessary .into() once return type changed
            Ok(().into())
        }
    }
}

// SBP-M1 review: add doc comment for struct
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct BagClass<AccountId, ClassId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting claims
    pub class_id: ClassId,
}

// SBP-M1 review: add doc comment for struct
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Bag<AccountId, ClassId, AssetId> {
    /// The operator of the bag
    pub operator: AccountId,
    /// The class_id for minting shares
    pub class_id: ClassId,
    /// The asset_id for minting shares
    pub asset_id: AssetId,
    /// Total number of shares
    // SBP-M1 review: consider making Balance a type on Config - runtime can configure it with type from primitives
    pub total_shares: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn do_register(
        who: &T::AccountId,
        class_id: T::ClassId,
        // SBP-M1 review: not consumed, can pass by reference
        metadata: sugarfunge_asset::ClassMetadataOf<T>,
    ) -> DispatchResult {
        ensure!(
            // SBP-M1 review: needless borrow
            !BagClasses::<T>::contains_key(&class_id),
            Error::<T>::BagClassExists
        );

        // SBP-M1 review: refactor into account_id() function
        let owner = <T as Config>::PalletId::get().into_account_truncating();
        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass
        // SBP-M1 review: unnecessary clone and needless borrows
        sugarfunge_asset::Pallet::<T>::do_create_class(&who, &owner, class_id, metadata.clone())?;

        let bag_class = BagClass {
            operator: who.clone(),
            class_id,
        };

        // SBP-M1 review: BagClasses are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: BagClasses removal should also remove NextBagId entry
        BagClasses::<T>::insert(class_id, &bag_class);

        Self::deposit_event(Event::Register {
            // SBP-M1 review: unnecessary clone
            who: who.clone(),
            class_id,
        });

        Ok(())
    }

    // SBP-M1 review: too many lines, refactor
    pub fn do_create(
        who: &T::AccountId,
        class_id: T::ClassId,
        // SBP-M1 review: use bounded vectors
        // SBP-M1 review: combine into single parameter using tuple
        owners: &Vec<T::AccountId>,
        shares: &Vec<Balance>,
        // SBP-M1 review: return type not used anywhere, use DispatchResult
    ) -> Result<T::AccountId, DispatchError> {
        ensure!(
            // SBP-M1 review: needless borrow
            BagClasses::<T>::contains_key(&class_id),
            Error::<T>::InvalidBagClass
        );

        // SBP-M1 review: using bounded vector of tuples avoids this
        ensure!(owners.len() == shares.len(), Error::<T>::InvalidArrayLength);

        // SBP-M1 review: try_mutate unnecessary as no error returned, use .mutate(). Safe math fix will require try_mutate however, returning ArithmeticError::Overflow depending on id value
        // SBP-M1 review: needless borrow
        let bag_id = NextBagId::<T>::try_mutate(&class_id, |id| -> Result<u64, DispatchError> {
            let current_id = *id;
            // SBP-M1 review: use safe math
            *id = *id + 1;
            Ok(current_id)
        })?;

        // SBP-M1 review: convert directly into u64 to save a cast
        let block_number: u32 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let sub = vec![block_number as u64, class_id.into(), bag_id];
        let bag = <T as Config>::PalletId::get().into_sub_account_truncating(sub);

        ensure!(!Bags::<T>::contains_key(&bag), Error::<T>::BagExists);

        // SBP-M1 review: consider whether bag deposit should increase based on number of shares. A static deposit value may not account for very large share structures.
        let deposit = T::CreateBagDeposit::get();
        <T as Config>::Currency::transfer(who, &bag, deposit, AllowDeath)?;

        let asset_id: T::AssetId = bag_id.into();

        // SBP-M1 review: refactor into account_id() function
        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Mint shares for each owner
        // SBP-M1 review: iteration should be bounded to complete within block limits. Each iteration is adding state and should be benchmarked accordingly.
        for (idx, owner) in owners.iter().enumerate() {
            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass + Mint
            sugarfunge_asset::Pallet::<T>::do_mint(
                &operator,
                // SBP-M1 review: needless borrow
                &owner,
                class_id,
                asset_id,
                // SBP-M1 review: indexing may panic, .get() preferred. Eliminated by using tuple as suggested above
                shares[idx],
            )?;
        }

        let new_bag = Bag {
            // SBP-M1 review: unnecessary clone
            operator: operator.clone(),
            class_id,
            asset_id,
            total_shares: shares.iter().sum(),
        };

        // SBP-M1 review: Bags are never removed, consider mechanism for avoiding state bloat
        // SBP-M1 review: corresponding bag accounts need cleanup when bag removed, with deposit returned
        Bags::<T>::insert(&bag, &new_bag);

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Created {
            bag: bag.clone(),
            who: who.clone(),
            class_id,
            asset_id,
            owners: owners.clone(),
        });

        // SBP-M1 review: unnecessary clone and return type not used
        Ok(bag.clone())
    }

    pub fn do_deposit(
        who: &T::AccountId,
        bag: &T::AccountId,
        // SBP-M1 review: combine values into tuple to eliminate length checking and simplify iteration
        class_ids: Vec<T::ClassId>,
        asset_ids: Vec<Vec<T::AssetId>>,
        amounts: Vec<Vec<Balance>>,
    ) -> DispatchResult {
        // SBP-M1 review: needless borrow
        ensure!(Bags::<T>::contains_key(&bag), Error::<T>::InvalidBag);

        // SBP-M1 review: can be eliminated by using tuple
        ensure!(
            class_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );
        ensure!(
            asset_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        // SBP-M1 review: iteration should be bounded to complete within block limits. Each iteration is changing state and should be benchmarked accordingly.
        for (idx, class_id) in class_ids.iter().enumerate() {
            // SBP-M1 review: improve parameter data type to avoid this
            ensure!(
                // SBP-M1 review: indexing may panic, .get() preferred. Can be eliminated by improving parameter type
                asset_ids[idx].len() == amounts[idx].len(),
                Error::<T>::InvalidArrayLength
            );

            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : CreateClass + Mint + BatchTransfer
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                // SBP-M1 review: needless borrows
                &who,
                &who,
                &bag,
                *class_id,
                // SBP-M1 review: indexing may panic, .get() preferred. Can be eliminated by improving parameter type
                asset_ids[idx].clone(),
                amounts[idx].clone(),
            )?;
        }

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Deposit {
            bag: bag.clone(),
            who: who.clone(),
        });

        // SBP-M1 review: unnecessary .into()
        Ok(().into())
    }

    // SBP-M1 review: should sweep not remove bag from Bags storage item?
    // SBP-M1 review: too many lines, refactor
    pub fn do_sweep(
        who: &T::AccountId,
        to: &T::AccountId,
        bag: &T::AccountId,
        // SBP-M1 review: return type not used anywhere, use DispatchResult
    ) -> Result<(Vec<T::ClassId>, Vec<Vec<T::AssetId>>, Vec<Vec<Balance>>), DispatchError> {
        let bag_info = Bags::<T>::get(bag).ok_or(Error::<T>::InvalidBag)?;

        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Inspect
        let shares =
            sugarfunge_asset::Pallet::<T>::balance_of(who, bag_info.class_id, bag_info.asset_id);
        ensure!(
            // SBP-M1 review: shares >= bag_info.total_shares ?
            shares == bag_info.total_shares,
            Error::<T>::InsufficientShares
        );

        // SBP-M1 review: refactor into account_id() function
        let operator: T::AccountId = <T as Config>::PalletId::get().into_account_truncating();

        // Burn bag shares
        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Burn
        sugarfunge_asset::Pallet::<T>::do_burn(
            &operator,
            who,
            bag_info.class_id,
            bag_info.asset_id,
            bag_info.total_shares,
        )?;

        // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : Inspect
        // SBP-M1 review: needless borrow
        let balances = sugarfunge_asset::Pallet::<T>::balances_of_owner(&bag)?;
        // SBP-M1 review: iteration should be bounded to complete within block limits.
        // SBP-M1 review: destructure into clearer variable names
        let balances = balances.iter().fold(
            (
                // SBP-M1 review: simplify using tuple
                Vec::<T::ClassId>::new(),
                Vec::<Vec<T::AssetId>>::new(),
                Vec::<Vec<Balance>>::new(),
            ),
            |(mut class_ids, mut asset_ids, mut balances), (class_id, asset_id, balance)| {
                // SBP-M1 review: use .map_or_else()
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
                    // SBP-M1 review: use safe math
                    asset_ids.resize(class_idx + 1, vec![]);
                }
                // SBP-M1 review: indexing may panic, prefer .get()
                asset_ids[class_idx].push(*asset_id);
                if balances.len() <= class_idx {
                    // SBP-M1 review: use safe math
                    balances.resize(class_idx + 1, vec![]);
                }
                // SBP-M1 review: indexing may panic, prefer .get()
                balances[class_idx].push(*balance);
                (class_ids, asset_ids, balances)
            },
        );

        // SBP-M1 review: iteration should be bounded to complete within block limits.
        for (idx, class_id) in balances.0.iter().enumerate() {
            // SBP-M1 review: access via trait bounds on Config item - e.g. type Asset : BatchTransfer
            sugarfunge_asset::Pallet::<T>::do_batch_transfer_from(
                // SBP-M1 review: needless borrows
                &bag,
                &bag,
                to,
                *class_id,
                // SBP-M1 review: indexing may panic, prefer .get()
                balances.1[idx].clone(),
                balances.2[idx].clone(),
            )?;
        }

        // SBP-M1 review: unnecessary clones
        Self::deposit_event(Event::Sweep {
            bag: bag.clone(),
            who: who.clone(),
            to: to.clone(),
        });

        // SBP-M1 review: return type not used anywhere
        Ok(balances)
    }
}

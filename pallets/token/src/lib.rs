#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, HasCompact};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, ReservableCurrency},
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, One, Zero},
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Collection<AccountId> {
    owner: AccountId,
    metadata: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Token<CollectionId, AccountId> {
    collection_id: CollectionId,
    creator: AccountId,
    metadata: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ApprovalKey<AccountId> {
    owner: AccountId,
    operator: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// The minimum balance to create collection
        #[pallet::constant]
        type CreateTokenCollectionDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type TokenId: Member + Parameter + Default + Copy + HasCompact + From<u64> + Into<u64>;

        type CollectionId: Member
            + Parameter
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + From<u64>
            + Into<u64>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Collections<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CollectionId, Collection<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn next_collection_id)]
    pub(super) type NextCollectionId<T: Config> = StorageValue<_, T::CollectionId, ValueQuery>;

    #[pallet::storage]
    pub(super) type Tokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128,
        T::TokenId,
        Token<T::CollectionId, T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn token_count)]
    pub(super) type TokenCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CollectionId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn balances)]
    pub(super) type Balances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        (T::CollectionId, T::TokenId),
        Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn operator_approvals)]
    pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        ApprovalKey<T::AccountId>,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CollectionCreated(T::CollectionId, T::AccountId),
        TokenCreated(T::CollectionId, T::TokenId, T::AccountId),
        Mint(T::AccountId, T::CollectionId, T::TokenId, Balance),
        BatchMint(T::AccountId, T::CollectionId, Vec<T::TokenId>, Vec<Balance>),
        Burn(T::AccountId, T::CollectionId, T::TokenId, Balance),
        BatchBurn(T::AccountId, T::CollectionId, Vec<T::TokenId>, Vec<Balance>),
        Transferred(
            T::AccountId,
            T::AccountId,
            T::CollectionId,
            T::TokenId,
            Balance,
        ),
        BatchTransferred(
            T::AccountId,
            T::AccountId,
            T::CollectionId,
            Vec<T::TokenId>,
            Vec<Balance>,
        ),
        ApprovalForAll(T::AccountId, T::AccountId, T::CollectionId, bool),
    }

    #[pallet::error]
    pub enum Error<T> {
        Unknown,
        InUse,
        InvalidTokenId,
        InsufficientBalance,
        NumOverflow,
        InvalidArrayLength,
        Overflow,
        NoAvailableCollectionId,
        InvalidCollectionId,
        NoPermission,
        CollectionNotFound,
        TokenNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_collection(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_collection(&who, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_token(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
            metadata: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_token(&who, collection_id, token_id, metadata)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn set_approval_for_all(
            origin: OriginFor<T>,
            operator: T::AccountId,
            collection_id: T::CollectionId,
            approved: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_set_approval_for_all(&who, &operator, collection_id, approved)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_transfer_from(&who, &from, &to, collection_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            collection_id: T::CollectionId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_transfer_from(&who, &from, &to, collection_id, token_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_mint(&who, &to, collection_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            collection_id: T::CollectionId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_mint(&who, &to, collection_id, token_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_burn(&who, &from, collection_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            collection_id: T::CollectionId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_burn(&who, &from, collection_id, token_ids, amounts)?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_create_collection(
        who: &T::AccountId,
        metadata: Vec<u8>,
    ) -> Result<T::CollectionId, DispatchError> {
        let deposit = T::CreateTokenCollectionDeposit::get();
        T::Currency::reserve(&who, deposit.clone())?;

        let collection_id =
            NextCollectionId::<T>::try_mutate(|id| -> Result<T::CollectionId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableCollectionId)?;
                Ok(current_id)
            })?;

        let collection = Collection {
            owner: who.clone(),
            metadata,
        };

        Collections::<T>::insert(collection_id, collection);

        Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));

        Ok(collection_id)
    }

    pub fn do_create_token(
        who: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, collection_id)?;
        ensure!(
            !Tokens::<T>::contains_key(collection_id, token_id),
            Error::<T>::InUse
        );

        Tokens::<T>::insert(
            collection_id,
            token_id,
            Token {
                collection_id,
                creator: who.clone(),
                metadata,
            },
        );

        TokenCount::<T>::try_mutate(collection_id, |count| -> DispatchResult {
            *count = count
                .checked_add(One::one())
                .ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::TokenCreated(collection_id, token_id, who.clone()));
        Ok(())
    }

    pub fn do_set_approval_for_all(
        who: &T::AccountId,
        operator: &T::AccountId,
        collection_id: T::CollectionId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(
            Collections::<T>::contains_key(collection_id),
            Error::<T>::CollectionNotFound
        );

        let key = ApprovalKey {
            owner: who.clone(),
            operator: operator.clone(),
        };
        OperatorApprovals::<T>::try_mutate(collection_id, &key, |status| -> DispatchResult {
            *status = approved;
            Ok(())
        })?;

        Self::deposit_event(Event::ApprovalForAll(
            who.clone(),
            operator.clone(),
            collection_id,
            approved,
        ));

        Ok(())
    }

    pub fn do_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, collection_id)?;

        Self::add_balance_to(to, collection_id, token_id, amount)?;

        Self::deposit_event(Event::Mint(to.clone(), collection_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, collection_id)?;
        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            Self::add_balance_to(to, collection_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchMint(
            to.clone(),
            collection_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn do_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, collection_id)?;

        Self::remove_balance_from(from, collection_id, token_id, amount)?;

        Self::deposit_event(Event::Burn(from.clone(), collection_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        collection_id: T::CollectionId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, collection_id)?;
        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            Self::remove_balance_from(from, collection_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchBurn(
            from.clone(),
            collection_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn do_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        ensure!(
            Self::approved_or_owner(who, from, collection_id),
            Error::<T>::NoPermission
        );

        if from == to || amount == Zero::zero() {
            return Ok(());
        }

        Self::remove_balance_from(from, collection_id, token_id, amount)?;

        Self::add_balance_to(to, collection_id, token_id, amount)?;

        Self::deposit_event(Event::Transferred(
            from.clone(),
            to.clone(),
            collection_id,
            token_id,
            amount,
        ));

        Ok(())
    }

    pub fn do_batch_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        ensure!(
            Self::approved_or_owner(who, from, collection_id),
            Error::<T>::NoPermission
        );

        if from == to {
            return Ok(());
        }

        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            Self::remove_balance_from(from, collection_id, token_id, amount)?;

            Self::add_balance_to(to, collection_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchTransferred(
            from.clone(),
            to.clone(),
            collection_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn approved_or_owner(
        who: &T::AccountId,
        account: &T::AccountId,
        collection_id: T::CollectionId,
    ) -> bool {
        *who == *account || Self::is_approved_for_all(account, who, collection_id)
    }

    pub fn is_approved_for_all(
        owner: &T::AccountId,
        operator: &T::AccountId,
        collection_id: T::CollectionId,
    ) -> bool {
        let key = ApprovalKey {
            owner: owner.clone(),
            operator: operator.clone(),
        };
        Self::operator_approvals(collection_id, &key)
    }

    pub fn balance_of(
        owner: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
    ) -> Balance {
        Self::balances(owner, (collection_id, token_id))
    }

    pub fn balance_of_batch(
        owners: &Vec<T::AccountId>,
        collection_id: T::CollectionId,
        token_ids: Vec<T::TokenId>,
    ) -> Result<Vec<Balance>, DispatchError> {
        ensure!(
            owners.len() == token_ids.len(),
            Error::<T>::InvalidArrayLength
        );

        let mut batch_balances = vec![Balance::from(0u32); owners.len()];

        let n = owners.len();
        for i in 0..n {
            let owner = &owners[i];
            let token_id = token_ids[i];

            batch_balances[i] = Self::balances(owner, (collection_id, token_id));
        }

        Ok(batch_balances)
    }

    pub fn balance_of_single_owner_batch(
        owner: &T::AccountId,
        collection_id: T::CollectionId,
        token_ids: Vec<T::TokenId>,
    ) -> Result<Vec<Balance>, DispatchError> {
        let mut batch_balances = vec![Balance::from(0u32); token_ids.len()];

        let n = token_ids.len();
        for i in 0..n {
            let owner = owner.clone();
            let token_id = token_ids[i];

            batch_balances[i] = Self::balances(owner, (collection_id, token_id));
        }

        Ok(batch_balances)
    }

    fn add_balance_to(
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Balances::<T>::try_mutate(to, (collection_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Ok(())
    }

    fn remove_balance_from(
        from: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Balances::<T>::try_mutate(from, (collection_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Ok(())
    }

    fn maybe_check_owner(who: &T::AccountId, collection_id: T::CollectionId) -> DispatchResult {
        let collection = Collections::<T>::get(collection_id).ok_or(Error::<T>::InvalidCollectionId)?;
        ensure!(*who == collection.owner, Error::<T>::NoPermission);

        Ok(())
    }
}

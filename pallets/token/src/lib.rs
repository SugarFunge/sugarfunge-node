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
pub struct Class<AccountId> {
    owner: AccountId,
    metadata: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Token<ClassId, AccountId> {
    class_id: ClassId,
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

        /// The minimum balance to create class
        #[pallet::constant]
        type CreateTokenClassDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type TokenId: Member + Parameter + Default + Copy + HasCompact + From<u64> + Into<u64>;

        type ClassId: Member
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
    pub(super) type Classs<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ClassId, Class<T::AccountId>>;

    #[pallet::storage]
    #[pallet::getter(fn next_class_id)]
    pub(super) type NextClassId<T: Config> = StorageValue<_, T::ClassId, ValueQuery>;

    #[pallet::storage]
    pub(super) type Tokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::ClassId,
        Blake2_128,
        T::TokenId,
        Token<T::ClassId, T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn token_count)]
    pub(super) type TokenCount<T: Config> =
        StorageMap<_, Blake2_128Concat, T::ClassId, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn balances)]
    pub(super) type Balances<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        (T::ClassId, T::TokenId),
        Balance,
        ValueQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn operator_approvals)]
    pub(super) type OperatorApprovals<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::ClassId,
        Blake2_128Concat,
        ApprovalKey<T::AccountId>,
        bool,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ClassCreated(T::ClassId, T::AccountId),
        TokenCreated(T::ClassId, T::TokenId, T::AccountId),
        Mint(T::AccountId, T::ClassId, T::TokenId, Balance),
        BatchMint(T::AccountId, T::ClassId, Vec<T::TokenId>, Vec<Balance>),
        Burn(T::AccountId, T::ClassId, T::TokenId, Balance),
        BatchBurn(T::AccountId, T::ClassId, Vec<T::TokenId>, Vec<Balance>),
        Transferred(
            T::AccountId,
            T::AccountId,
            T::ClassId,
            T::TokenId,
            Balance,
        ),
        BatchTransferred(
            T::AccountId,
            T::AccountId,
            T::ClassId,
            Vec<T::TokenId>,
            Vec<Balance>,
        ),
        ApprovalForAll(T::AccountId, T::AccountId, T::ClassId, bool),
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
        NoAvailableClassId,
        InvalidClassId,
        NoPermission,
        ClassNotFound,
        TokenNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_class(&who, metadata)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn create_token(
            origin: OriginFor<T>,
            class_id: T::ClassId,
            token_id: T::TokenId,
            metadata: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_token(&who, class_id, token_id, metadata)?;
            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn set_approval_for_all(
            origin: OriginFor<T>,
            operator: T::AccountId,
            class_id: T::ClassId,
            approved: bool,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_set_approval_for_all(&who, &operator, class_id, approved)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_transfer_from(&who, &from, &to, class_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_transfer_from(
            origin: OriginFor<T>,
            from: T::AccountId,
            to: T::AccountId,
            class_id: T::ClassId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_transfer_from(&who, &from, &to, class_id, token_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            class_id: T::ClassId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_mint(&who, &to, class_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_mint(
            origin: OriginFor<T>,
            to: T::AccountId,
            class_id: T::ClassId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_mint(&who, &to, class_id, token_ids, amounts)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            class_id: T::ClassId,
            token_id: T::TokenId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_burn(&who, &from, class_id, token_id, amount)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn batch_burn(
            origin: OriginFor<T>,
            from: T::AccountId,
            class_id: T::ClassId,
            token_ids: Vec<T::TokenId>,
            amounts: Vec<Balance>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_batch_burn(&who, &from, class_id, token_ids, amounts)?;

            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn do_create_class(
        who: &T::AccountId,
        metadata: Vec<u8>,
    ) -> Result<T::ClassId, DispatchError> {
        let deposit = T::CreateTokenClassDeposit::get();
        T::Currency::reserve(&who, deposit.clone())?;

        let class_id =
            NextClassId::<T>::try_mutate(|id| -> Result<T::ClassId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableClassId)?;
                Ok(current_id)
            })?;

        let class = Class {
            owner: who.clone(),
            metadata,
        };

        Classs::<T>::insert(class_id, class);

        Self::deposit_event(Event::ClassCreated(class_id, who.clone()));

        Ok(class_id)
    }

    pub fn do_create_token(
        who: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        metadata: Vec<u8>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, class_id)?;
        ensure!(
            !Tokens::<T>::contains_key(class_id, token_id),
            Error::<T>::InUse
        );

        Tokens::<T>::insert(
            class_id,
            token_id,
            Token {
                class_id,
                creator: who.clone(),
                metadata,
            },
        );

        TokenCount::<T>::try_mutate(class_id, |count| -> DispatchResult {
            *count = count
                .checked_add(One::one())
                .ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Self::deposit_event(Event::TokenCreated(class_id, token_id, who.clone()));
        Ok(())
    }

    pub fn do_set_approval_for_all(
        who: &T::AccountId,
        operator: &T::AccountId,
        class_id: T::ClassId,
        approved: bool,
    ) -> DispatchResult {
        ensure!(
            Classs::<T>::contains_key(class_id),
            Error::<T>::ClassNotFound
        );

        let key = ApprovalKey {
            owner: who.clone(),
            operator: operator.clone(),
        };
        OperatorApprovals::<T>::try_mutate(class_id, &key, |status| -> DispatchResult {
            *status = approved;
            Ok(())
        })?;

        Self::deposit_event(Event::ApprovalForAll(
            who.clone(),
            operator.clone(),
            class_id,
            approved,
        ));

        Ok(())
    }

    pub fn do_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, class_id)?;

        Self::add_balance_to(to, class_id, token_id, amount)?;

        Self::deposit_event(Event::Mint(to.clone(), class_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_mint(
        who: &T::AccountId,
        to: &T::AccountId,
        class_id: T::ClassId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, class_id)?;
        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            Self::add_balance_to(to, class_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchMint(
            to.clone(),
            class_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn do_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, class_id)?;

        Self::remove_balance_from(from, class_id, token_id, amount)?;

        Self::deposit_event(Event::Burn(from.clone(), class_id, token_id, amount));

        Ok(())
    }

    pub fn do_batch_burn(
        who: &T::AccountId,
        from: &T::AccountId,
        class_id: T::ClassId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        Self::maybe_check_owner(who, class_id)?;
        ensure!(
            token_ids.len() == amounts.len(),
            Error::<T>::InvalidArrayLength
        );

        let n = token_ids.len();
        for i in 0..n {
            let token_id = token_ids[i];
            let amount = amounts[i];

            Self::remove_balance_from(from, class_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchBurn(
            from.clone(),
            class_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn do_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        ensure!(
            Self::approved_or_owner(who, from, class_id),
            Error::<T>::NoPermission
        );

        if from == to || amount == Zero::zero() {
            return Ok(());
        }

        Self::remove_balance_from(from, class_id, token_id, amount)?;

        Self::add_balance_to(to, class_id, token_id, amount)?;

        Self::deposit_event(Event::Transferred(
            from.clone(),
            to.clone(),
            class_id,
            token_id,
            amount,
        ));

        Ok(())
    }

    pub fn do_batch_transfer_from(
        who: &T::AccountId,
        from: &T::AccountId,
        to: &T::AccountId,
        class_id: T::ClassId,
        token_ids: Vec<T::TokenId>,
        amounts: Vec<Balance>,
    ) -> DispatchResult {
        ensure!(
            Self::approved_or_owner(who, from, class_id),
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

            Self::remove_balance_from(from, class_id, token_id, amount)?;

            Self::add_balance_to(to, class_id, token_id, amount)?;
        }

        Self::deposit_event(Event::BatchTransferred(
            from.clone(),
            to.clone(),
            class_id,
            token_ids,
            amounts,
        ));

        Ok(())
    }

    pub fn approved_or_owner(
        who: &T::AccountId,
        account: &T::AccountId,
        class_id: T::ClassId,
    ) -> bool {
        *who == *account || Self::is_approved_for_all(account, who, class_id)
    }

    pub fn is_approved_for_all(
        owner: &T::AccountId,
        operator: &T::AccountId,
        class_id: T::ClassId,
    ) -> bool {
        let key = ApprovalKey {
            owner: owner.clone(),
            operator: operator.clone(),
        };
        Self::operator_approvals(class_id, &key)
    }

    pub fn balance_of(
        owner: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
    ) -> Balance {
        Self::balances(owner, (class_id, token_id))
    }

    pub fn balance_of_batch(
        owners: &Vec<T::AccountId>,
        class_id: T::ClassId,
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

            batch_balances[i] = Self::balances(owner, (class_id, token_id));
        }

        Ok(batch_balances)
    }

    pub fn balance_of_single_owner_batch(
        owner: &T::AccountId,
        class_id: T::ClassId,
        token_ids: Vec<T::TokenId>,
    ) -> Result<Vec<Balance>, DispatchError> {
        let mut batch_balances = vec![Balance::from(0u32); token_ids.len()];

        let n = token_ids.len();
        for i in 0..n {
            let owner = owner.clone();
            let token_id = token_ids[i];

            batch_balances[i] = Self::balances(owner, (class_id, token_id));
        }

        Ok(batch_balances)
    }

    fn add_balance_to(
        to: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Balances::<T>::try_mutate(to, (class_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_add(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Ok(())
    }

    fn remove_balance_from(
        from: &T::AccountId,
        class_id: T::ClassId,
        token_id: T::TokenId,
        amount: Balance,
    ) -> DispatchResult {
        Balances::<T>::try_mutate(from, (class_id, token_id), |balance| -> DispatchResult {
            *balance = balance.checked_sub(amount).ok_or(Error::<T>::NumOverflow)?;
            Ok(())
        })?;

        Ok(())
    }

    fn maybe_check_owner(who: &T::AccountId, class_id: T::ClassId) -> DispatchResult {
        let class = Classs::<T>::get(class_id).ok_or(Error::<T>::InvalidClassId)?;
        ensure!(*who == class.owner, Error::<T>::NoPermission);

        Ok(())
    }
}

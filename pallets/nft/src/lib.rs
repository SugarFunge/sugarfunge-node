#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, Get, ReservableCurrency},
};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{One, Zero, AtLeast32BitUnsigned, CheckedAdd, CheckedSub},
    RuntimeDebug,
};
use sp_std::prelude::*;

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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
        type CreateCollectionDeposit: Get<BalanceOf<Self>>;

        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

        type CollectionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + Into<u128>;

        type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + Into<u128> + From<u128>;

        type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + One + Into<u128> + Into<Self::TokenId> + Into<BalanceOf<Self>>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type Collections<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CollectionId, CollectionInfo<T::AccountId, BalanceOf<T>>>;

    #[pallet::storage]
    #[pallet::getter(fn next_collection_id)]
    pub(super) type NextCollectionId<T: Config> = StorageValue<_, T::CollectionId, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub(super) type Tokens<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::CollectionId,
        Blake2_128Concat,
        T::TokenId,
        TokenInfo<T::AccountId>,
    >;

    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    pub(super) type NextTokenId<T: Config> =
        StorageMap<_, Blake2_128Concat, T::CollectionId, T::TokenId, ValueQuery>;

    /// Token existence check by owner
    #[pallet::storage]
    #[pallet::getter(fn token_owner_set)]
    pub type TokenOwnerSet<T: Config> = StorageNMap<
        _,
        (
            NMapKey<Blake2_128Concat, T::AccountId>, // owner
            NMapKey<Blake2_128Concat, T::CollectionId>,
            NMapKey<Blake2_128Concat, T::TokenId>,
        ),
        (),
        ValueQuery,
    >;

    /// Tokens by owner
    // #[pallet::storage]
    // #[pallet::getter(fn tokens_by_owner)]
    // pub type TokensByOwner<T: Config> =
    //     StorageMap<_, Blake2_128Concat, T::AccountId, (T::CollectionId, T::TokenId)>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        CollectionCreated(T::CollectionId, T::AccountId),
        TokenMint(T::CollectionId, Vec<T::TokenId>, T::AccountId),
        TokenBurned(T::CollectionId, T::TokenId, T::AccountId),
        TokenTransferred(T::CollectionId, T::TokenId, T::AccountId, T::AccountId),
        CollectionDestroyed(T::CollectionId, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        NumOverflow,
        NoAvailableCollectionId,
        CollectionNotFound,
        NoAvailableTokenId,
        TokenNotFound,
        InvalidQuantity,
        NoPermission,
        CannotDestroyCollection,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_collection(
            origin: OriginFor<T>,
            properties: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_create_collection(&who, properties)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
            metadata: Vec<u8>,
            quantity: T::Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_mint(&who, collection_id, metadata, quantity)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn transfer(
            origin: OriginFor<T>,
            to: T::AccountId,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_transfer_from(&who, &to, collection_id, token_id)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
            token_id: T::TokenId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_burn(&who, collection_id, token_id)?;

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn destroy_collection(
            origin: OriginFor<T>,
            collection_id: T::CollectionId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            Self::do_destroy_collection(&who, collection_id)?;

            Ok(().into())
        }
    }
}

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct CollectionInfo<AccountId, Balance> {
    /// Class owner
    pub owner: AccountId,
    /// Total issuance for the class
    pub total_supply: Balance,
    /// Minimum balance to create a collection
    pub deposit: Balance,
    /// Metadata from ipfs
    pub properties: Vec<u8>,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TokenInfo<AccountId> {
    /// Token owner
    pub owner: AccountId,
    /// Metadata from ipfs
    pub metadata: Vec<u8>,
}

impl<T: Config> Pallet<T> {
    pub fn do_create_collection(
        who: &T::AccountId,
        properties: Vec<u8>,
    ) -> Result<T::CollectionId, DispatchError> {
        let collection_id =
            NextCollectionId::<T>::try_mutate(|id| -> Result<T::CollectionId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableCollectionId)?;
                Ok(current_id)
            })?;

        let deposit = T::CreateCollectionDeposit::get();
        T::Currency::reserve(who, deposit.clone())?;

        let collection_info = CollectionInfo {
            owner: who.clone(),
            total_supply: Default::default(),
            deposit,
            properties,
        };

        Collections::<T>::insert(collection_id, collection_info);

        Self::deposit_event(Event::CollectionCreated(collection_id, who.clone()));
        Ok(collection_id)
    }

    pub fn do_mint(
        to: &T::AccountId,
        collection_id: T::CollectionId,
        metadata: Vec<u8>,
        quantity: T::Balance,
    ) -> Result<Vec<T::TokenId>, DispatchError> {
        NextTokenId::<T>::try_mutate(collection_id, |id| -> Result<Vec<T::TokenId>, DispatchError> {
            ensure!(quantity >= One::one(), Error::<T>::InvalidQuantity);
            let next_id = *id;
            *id = id
                .checked_add(&quantity.into())
                .ok_or(Error::<T>::NoAvailableTokenId)?;

            let mut token_ids: Vec<T::TokenId> = Vec::new();
            Collections::<T>::try_mutate(collection_id, |collection_info| -> DispatchResult {
                let info = collection_info
                    .as_mut()
                    .ok_or(Error::<T>::CollectionNotFound)?;

                ensure!(*to == info.owner, Error::<T>::NoPermission);

                let token_info = TokenInfo {
                    owner: to.clone(),
                    metadata,
                };

                for i in 0..quantity.into() {
                    let token_id = T::TokenId::from(next_id.into() + i);
                    token_ids.push(token_id);

                    Tokens::<T>::insert(collection_id, token_id, token_info.clone());
                    TokenOwnerSet::<T>::insert((to, collection_id, token_id), ());
                }

                info.total_supply = info
                    .total_supply
                    .checked_add(&quantity.into())
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::TokenMint(
                collection_id,
                token_ids.clone(),
                to.clone(),
            ));
            Ok(token_ids)
        })
    }

    pub fn do_burn(
        who: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
    ) -> DispatchResult {
        Tokens::<T>::try_mutate_exists(collection_id, token_id, |token_info| -> DispatchResult {
            let info = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(info.owner == *who, Error::<T>::NoPermission);

            Collections::<T>::try_mutate(collection_id, |collection_info| -> DispatchResult {
                let info = collection_info
                    .as_mut()
                    .ok_or(Error::<T>::CollectionNotFound)?;
                info.total_supply = info
                    .total_supply
                    .checked_sub(&One::one())
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            TokenOwnerSet::<T>::remove((who, collection_id, token_id));

            Self::deposit_event(Event::TokenBurned(collection_id, token_id, who.clone()));
            Ok(())
        })
    }

    pub fn do_transfer_from(
        from: &T::AccountId,
        to: &T::AccountId,
        collection_id: T::CollectionId,
        token_id: T::TokenId,
    ) -> DispatchResult {
        Tokens::<T>::try_mutate(collection_id, token_id, |token_info| -> DispatchResult {
            let info = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(info.owner == *from, Error::<T>::NoPermission);
            if from == to {
                return Ok(());
            }

            info.owner = to.clone();

            TokenOwnerSet::<T>::remove((from, collection_id, token_id));
            TokenOwnerSet::<T>::insert((to, collection_id, token_id), ());

            Self::deposit_event(Event::TokenTransferred(
                collection_id,
                token_id,
                from.clone(),
                to.clone(),
            ));

            Ok(())
        })
    }

    pub fn do_destroy_collection(
        who: &T::AccountId,
        collection_id: T::CollectionId,
    ) -> DispatchResult {
        Collections::<T>::try_mutate_exists(collection_id, |collection_info| -> DispatchResult {
            let info = collection_info
                .take()
                .ok_or(Error::<T>::CollectionNotFound)?;
            ensure!(info.owner == *who, Error::<T>::NoPermission);
            ensure!(
                info.total_supply == Zero::zero(),
                Error::<T>::CannotDestroyCollection
            );

            NextTokenId::<T>::remove(collection_id);

            T::Currency::unreserve(who, info.deposit);

            Self::deposit_event(Event::CollectionDestroyed(collection_id, who.clone()));

            Ok(())
        })
    }
}

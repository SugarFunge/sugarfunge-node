//! # Non Fungible Token
//! The module provides implementations for non-fungible-token.
//!
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//! - [`Module`](./struct.Module.html)
//!
//! ## Overview
//!
//! This module provides basic functions to create and manager
//! NFT(non fungible token) such as `create_collection`, `transfer`, `mint`, `burn`.

//! ### Module Functions
//!
//! - `create_collection` - Create NFT(non fungible token) collection
//! - `transfer` - Transfer NFT(non fungible token) to another account.
//! - `mint` - Mint NFT(non fungible token)
//! - `burn` - Burn NFT(non fungible token)
//! - `destroy_collection` - Destroy NFT(non fungible token) collection

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::*, Parameter};
use sp_runtime::{
    traits::{
        AtLeast32BitUnsigned, CheckedAdd, CheckedSub, MaybeSerializeDeserialize, Member, One, Zero,
    },
    DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::vec::Vec;

mod mock;
mod tests;

/// Collection info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct CollectionInfo<TokenId, AccountId, Data> {
    /// Collection metadata
    pub metadata: Vec<u8>,
    /// Total issuance for the collection
    pub total_issuance: TokenId,
    /// Collection owner
    pub owner: AccountId,
    /// Collection Properties
    pub data: Data,
}

/// Token info
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TokenInfo<AccountId, Data> {
    /// Token metadata
    pub metadata: Vec<u8>,
    /// Token owner
    pub owner: AccountId,
    /// Token Properties
    pub data: Data,
}

pub use module::*;

#[frame_support::pallet]
pub mod module {
    use super::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The collection ID type
        type CollectionId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The token ID type
        type TokenId: Parameter + Member + AtLeast32BitUnsigned + Default + Copy;
        /// The collection properties type
        type CollectionData: Parameter + Member + MaybeSerializeDeserialize;
        /// The token properties type
        type TokenData: Parameter + Member + MaybeSerializeDeserialize;
    }

    pub type CollectionInfoOf<T> = CollectionInfo<
        <T as Config>::TokenId,
        <T as frame_system::Config>::AccountId,
        <T as Config>::CollectionData,
    >;
    pub type TokenInfoOf<T> =
        TokenInfo<<T as frame_system::Config>::AccountId, <T as Config>::TokenData>;

    pub type GenesisTokenData<T> = (
        <T as frame_system::Config>::AccountId, // Token owner
        Vec<u8>,                                // Token metadata
        <T as Config>::TokenData,
    );
    pub type GenesisTokens<T> = (
        <T as frame_system::Config>::AccountId, // Token collection owner
        Vec<u8>,                                // Token collection metadata
        <T as Config>::CollectionData,
        Vec<GenesisTokenData<T>>, // Vector of tokens belonging to this collection
    );

    /// Error for non-fungible-token module.
    #[pallet::error]
    pub enum Error<T> {
        /// No available collection ID
        NoAvailableCollectionId,
        /// No available token ID
        NoAvailableTokenId,
        /// Token(CollectionId, TokenId) not found
        TokenNotFound,
        /// Collection not found
        CollectionNotFound,
        /// The operator is not the owner of the token and has no permission
        NoPermission,
        /// Arithmetic calculation overflow
        NumOverflow,
        /// Can not destroy collection
        /// Total issuance is not 0
        CannotDestroyCollection,
    }

    /// Next available collection ID.
    #[pallet::storage]
    #[pallet::getter(fn next_collection_id)]
    pub type NextCollectionId<T: Config> = StorageValue<_, T::CollectionId, ValueQuery>;

    /// Next available token ID.
    #[pallet::storage]
    #[pallet::getter(fn next_token_id)]
    pub type NextTokenId<T: Config> =
        StorageMap<_, Twox64Concat, T::CollectionId, T::TokenId, ValueQuery>;

    /// Store collection info.
    ///
    /// Returns `None` if collection info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn collectiones)]
    pub type Collections<T: Config> =
        StorageMap<_, Twox64Concat, T::CollectionId, CollectionInfoOf<T>>;

    /// Store token info.
    ///
    /// Returns `None` if token info not set or removed.
    #[pallet::storage]
    #[pallet::getter(fn tokens)]
    pub type Tokens<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::CollectionId,
        Twox64Concat,
        T::TokenId,
        TokenInfoOf<T>,
    >;

    /// Token existence check by owner and collection ID.
    #[pallet::storage]
    #[pallet::getter(fn tokens_by_owner)]
    pub type TokensByOwner<T: Config> = StorageDoubleMap<
        _,
        Twox64Concat,
        T::AccountId,
        Twox64Concat,
        (T::CollectionId, T::TokenId),
        (),
        ValueQuery,
    >;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub tokens: Vec<GenesisTokens<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            GenesisConfig { tokens: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.tokens.iter().for_each(|token_collection| {
                let collection_id = Pallet::<T>::create_collection(
                    &token_collection.0,
                    token_collection.1.to_vec(),
                    token_collection.2.clone(),
                )
                .expect("Create collection cannot fail while building genesis");
                for (account_id, token_metadata, token_data) in &token_collection.3 {
                    Pallet::<T>::mint(
                        &account_id,
                        collection_id,
                        token_metadata.to_vec(),
                        token_data.clone(),
                    )
                    .expect("Token mint cannot fail during genesis");
                }
            })
        }
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}

impl<T: Config> Pallet<T> {
    /// Create NFT(non fungible token) collection
    pub fn create_collection(
        owner: &T::AccountId,
        metadata: Vec<u8>,
        data: T::CollectionData,
    ) -> Result<T::CollectionId, DispatchError> {
        let collection_id =
            NextCollectionId::<T>::try_mutate(|id| -> Result<T::CollectionId, DispatchError> {
                let current_id = *id;
                *id = id
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NoAvailableCollectionId)?;
                Ok(current_id)
            })?;

        let info = CollectionInfo {
            metadata,
            total_issuance: Default::default(),
            owner: owner.clone(),
            data,
        };
        Collections::<T>::insert(collection_id, info);

        Ok(collection_id)
    }

    /// Transfer NFT(non fungible token) from `from` account to `to` account
    pub fn transfer(
        from: &T::AccountId,
        to: &T::AccountId,
        token: (T::CollectionId, T::TokenId),
    ) -> DispatchResult {
        Tokens::<T>::try_mutate(token.0, token.1, |token_info| -> DispatchResult {
            let mut info = token_info.as_mut().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(info.owner == *from, Error::<T>::NoPermission);
            if from == to {
                // no change needed
                return Ok(());
            }

            info.owner = to.clone();

            TokensByOwner::<T>::remove(from, token);
            TokensByOwner::<T>::insert(to, token, ());

            Ok(())
        })
    }

    /// Mint NFT(non fungible token) to `owner`
    pub fn mint(
        owner: &T::AccountId,
        collection_id: T::CollectionId,
        metadata: Vec<u8>,
        data: T::TokenData,
    ) -> Result<T::TokenId, DispatchError> {
        NextTokenId::<T>::try_mutate(collection_id, |id| -> Result<T::TokenId, DispatchError> {
            let token_id = *id;
            *id = id
                .checked_add(&One::one())
                .ok_or(Error::<T>::NoAvailableTokenId)?;

            Collections::<T>::try_mutate(collection_id, |collection_info| -> DispatchResult {
                let info = collection_info
                    .as_mut()
                    .ok_or(Error::<T>::CollectionNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_add(&One::one())
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            let token_info = TokenInfo {
                metadata,
                owner: owner.clone(),
                data,
            };
            Tokens::<T>::insert(collection_id, token_id, token_info);
            TokensByOwner::<T>::insert(owner, (collection_id, token_id), ());

            Ok(token_id)
        })
    }

    /// Burn NFT(non fungible token) from `owner`
    pub fn burn(owner: &T::AccountId, token: (T::CollectionId, T::TokenId)) -> DispatchResult {
        Tokens::<T>::try_mutate_exists(token.0, token.1, |token_info| -> DispatchResult {
            let t = token_info.take().ok_or(Error::<T>::TokenNotFound)?;
            ensure!(t.owner == *owner, Error::<T>::NoPermission);

            Collections::<T>::try_mutate(token.0, |collection_info| -> DispatchResult {
                let info = collection_info
                    .as_mut()
                    .ok_or(Error::<T>::CollectionNotFound)?;
                info.total_issuance = info
                    .total_issuance
                    .checked_sub(&One::one())
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            TokensByOwner::<T>::remove(owner, token);

            Ok(())
        })
    }

    /// Destroy NFT(non fungible token) collection
    pub fn destroy_collection(
        owner: &T::AccountId,
        collection_id: T::CollectionId,
    ) -> DispatchResult {
        Collections::<T>::try_mutate_exists(collection_id, |collection_info| -> DispatchResult {
            let info = collection_info
                .take()
                .ok_or(Error::<T>::CollectionNotFound)?;
            ensure!(info.owner == *owner, Error::<T>::NoPermission);
            ensure!(
                info.total_issuance == Zero::zero(),
                Error::<T>::CannotDestroyCollection
            );

            NextTokenId::<T>::remove(collection_id);

            Ok(())
        })
    }

    pub fn is_owner(account: &T::AccountId, token: (T::CollectionId, T::TokenId)) -> bool {
        TokensByOwner::<T>::contains_key(account, token)
    }
}

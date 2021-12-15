#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    traits::Get,
    PalletId,
};
use orml_traits::{MultiCurrency, MultiCurrencyExtended};
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, RuntimeDebug};
use sp_std::{fmt::Debug, prelude::*};
use sugarfunge_primitives::{Balance, CurrencyId};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + sugarfunge_asset::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type Currency: MultiCurrencyExtended<
            Self::AccountId,
            CurrencyId = CurrencyId,
            Balance = Balance,
        >;

        #[pallet::constant]
        type CreateCurrencyClassDeposit: Get<Balance>;

        #[pallet::constant]
        type GetNativeCurrencyId: Get<CurrencyId>;
    }

    pub type GenesisInstance<T> = (<T as frame_system::Config>::AccountId, Vec<u8>);

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub class: GenesisInstance<T>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                class: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            Pallet::<T>::create_class(&self.class.0, self.class.1.to_vec())
                .expect("Create class cannot fail while building genesis");
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn currency_class)]
    pub(super) type CurrencyClass<T: Config> = StorageValue<_, T::ClassId, OptionQuery>;

    #[pallet::storage]
    pub(super) type CurrencyAssets<T: Config> =
        StorageMap<_, Blake2_128Concat, CurrencyId, AssetInfo<T::ClassId, T::AssetId, Balance>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        AssetCreated(CurrencyId, T::AccountId),
        AssetMint(CurrencyId, Balance, T::AccountId),
        AssetBurn(CurrencyId, Balance, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        Unknown,
        NumOverflow,
        CurrencyClassNotCreated,
        CurrencyAssetNotFound,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn mint(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let class_id = CurrencyClass::<T>::get().ok_or(Error::<T>::CurrencyClassNotCreated)?;

            let module_account = Self::account_id();
            <T as Config>::Currency::transfer(currency_id, &who, &module_account, amount)?;

            if !CurrencyAssets::<T>::contains_key(currency_id) {
                let asset_id = Self::convert_to_asset_id(currency_id);
                sugarfunge_asset::Pallet::<T>::do_create_asset(
                    &module_account,
                    class_id,
                    asset_id,
                    [].to_vec(),
                )?;

                let asset_info = AssetInfo {
                    class_id,
                    asset_id: asset_id.clone(),
                    total_supply: Default::default(),
                };

                CurrencyAssets::<T>::insert(currency_id, asset_info);
            }

            CurrencyAssets::<T>::try_mutate(currency_id, |asset_info| -> DispatchResult {
                let info = asset_info.as_mut().ok_or(Error::<T>::Unknown)?;

                sugarfunge_asset::Pallet::<T>::do_mint(
                    &module_account,
                    &who,
                    class_id,
                    info.asset_id,
                    amount,
                )?;

                info.total_supply = info
                    .total_supply
                    .checked_add(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::AssetMint(currency_id, amount, who));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            CurrencyAssets::<T>::try_mutate(currency_id, |asset_info| -> DispatchResult {
                let info = asset_info
                    .as_mut()
                    .ok_or(Error::<T>::CurrencyAssetNotFound)?;

                let class_id =
                    CurrencyClass::<T>::get().ok_or(Error::<T>::CurrencyClassNotCreated)?;

                let module_account = Self::account_id();
                <T as Config>::Currency::transfer(currency_id, &module_account, &who, amount)?;

                sugarfunge_asset::Pallet::<T>::do_burn(
                    &module_account,
                    &who,
                    class_id,
                    info.asset_id,
                    amount,
                )?;

                info.total_supply = info
                    .total_supply
                    .checked_sub(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::AssetBurn(currency_id, amount, who));

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetInfo<
    ClassId: Encode + Decode + Clone + Debug + Eq + PartialEq,
    AssetId: Encode + Decode + Clone + Debug + Eq + PartialEq,
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    class_id: ClassId,
    asset_id: AssetId,
    total_supply: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn create_class(who: &T::AccountId, data: Vec<u8>) -> DispatchResult {
        let module_account = Self::account_id();
        let native_currency_id = T::GetNativeCurrencyId::get();
        let amount = T::CreateCurrencyClassDeposit::get();

        <T as Config>::Currency::transfer(native_currency_id, &who, &module_account, amount)?;

        let class_id = sugarfunge_asset::Pallet::<T>::do_create_class(&module_account, data)?;
        CurrencyClass::<T>::put(class_id);

        Ok(())
    }

    pub fn get_currency_asset(
        currency_id: CurrencyId,
    ) -> Result<(T::ClassId, T::AssetId), DispatchError> {
        let asset_info =
            CurrencyAssets::<T>::get(currency_id).ok_or(Error::<T>::CurrencyAssetNotFound)?;
        Ok((asset_info.class_id, asset_info.asset_id))
    }

    pub fn convert_to_asset_id(id: CurrencyId) -> T::AssetId {
        let n: u64 = id.into();
        n.into()
    }
}

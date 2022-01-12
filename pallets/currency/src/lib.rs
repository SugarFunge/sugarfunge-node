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

    pub type GenesisInstance<T> = (
        <T as frame_system::Config>::AccountId,
        <T as sugarfunge_asset::Config>::ClassId,
        <T as sugarfunge_asset::Config>::AssetId,
        Vec<u8>,
    );

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
            Pallet::<T>::create_currency(
                &self.class.0,
                self.class.1,
                self.class.2,
                self.class.3.to_vec(),
            )
            .expect("Create class cannot fail while building genesis");
        }
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    pub(super) type CurrencyAssets<T: Config> =
        StorageMap<_, Blake2_128Concat, CurrencyId, AssetInfo<Balance>>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Mint(CurrencyId, Balance, T::AccountId),
        Burn(CurrencyId, Balance, T::AccountId),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        Unknown,
        NumOverflow,
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

            let CurrencyId(class_id, asset_id) = currency_id;

            let module_account = Self::account_id();
            <T as Config>::Currency::transfer(currency_id, &who, &module_account, amount)?;

            if !CurrencyAssets::<T>::contains_key(currency_id) {
                sugarfunge_asset::Pallet::<T>::do_create_asset(
                    &module_account,
                    class_id.into(),
                    asset_id.into(),
                    [].to_vec(),
                )?;

                let asset_info = AssetInfo {
                    total_supply: Default::default(),
                };

                CurrencyAssets::<T>::insert(currency_id, asset_info);
            }

            CurrencyAssets::<T>::try_mutate(currency_id, |asset_info| -> DispatchResult {
                let info = asset_info.as_mut().ok_or(Error::<T>::Unknown)?;

                sugarfunge_asset::Pallet::<T>::do_mint(
                    &module_account,
                    &who,
                    class_id.into(),
                    asset_id.into(),
                    amount,
                )?;

                info.total_supply = info
                    .total_supply
                    .checked_add(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Mint(currency_id, amount, who));

            Ok(().into())
        }

        #[pallet::weight(10_000)]
        pub fn burn(
            origin: OriginFor<T>,
            currency_id: CurrencyId,
            amount: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let CurrencyId(class_id, asset_id) = currency_id;

            CurrencyAssets::<T>::try_mutate(currency_id, |asset_info| -> DispatchResult {
                let info = asset_info
                    .as_mut()
                    .ok_or(Error::<T>::CurrencyAssetNotFound)?;

                let module_account = Self::account_id();
                <T as Config>::Currency::transfer(currency_id, &module_account, &who, amount)?;

                sugarfunge_asset::Pallet::<T>::do_burn(
                    &module_account,
                    &who,
                    class_id.into(),
                    asset_id.into(),
                    amount,
                )?;

                info.total_supply = info
                    .total_supply
                    .checked_sub(amount)
                    .ok_or(Error::<T>::NumOverflow)?;
                Ok(())
            })?;

            Self::deposit_event(Event::Burn(currency_id, amount, who));

            Ok(().into())
        }
    }
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct AssetInfo<Balance: Encode + Decode + Clone + Debug + Eq + PartialEq> {
    total_supply: Balance,
}

impl<T: Config> Pallet<T> {
    pub fn account_id() -> T::AccountId {
        T::PalletId::get().into_account()
    }

    pub fn create_currency(
        who: &T::AccountId,
        class_id: T::ClassId,
        asset_id: T::AssetId,
        data: Vec<u8>,
    ) -> DispatchResult {
        let module_account = Self::account_id();
        let native_currency_id = T::GetNativeCurrencyId::get();
        let amount = T::CreateCurrencyClassDeposit::get();

        <T as Config>::Currency::transfer(native_currency_id, &who, &module_account, amount)?;

        sugarfunge_asset::Pallet::<T>::do_create_class(
            &module_account,
            &module_account,
            class_id,
            data.clone(),
        )?;
        sugarfunge_asset::Pallet::<T>::do_create_asset(
            &module_account,
            class_id,
            asset_id,
            data.clone(),
        )?;

        let currency_id = CurrencyId(class_id.into(), asset_id.into());
        CurrencyAssets::<T>::insert(currency_id, AssetInfo { total_supply: 0 });

        Ok(())
    }

    pub fn get_currency_asset(
        currency_id: CurrencyId,
    ) -> Result<(T::ClassId, T::AssetId), DispatchError> {
        let CurrencyId(class_id, asset_id) = currency_id;
        let _ = CurrencyAssets::<T>::get(currency_id).ok_or(Error::<T>::CurrencyAssetNotFound)?;
        Ok((class_id.into(), asset_id.into()))
    }
}

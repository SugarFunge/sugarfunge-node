#![cfg(feature = "runtime-benchmarks")]
use super::*;

use crate::Pallet as Assets;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller};
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

const BASE_SUGAR: u128 = 100000000000000000000000000;
const MINT_VALUE: u128 = 1000000000000;
const BURN_VALUE: u128 = 500000000000;
const TRANSFER_VALUE: u128 = 500;
const CLASS_ID: u64 = 100;
const ASSET_ID: u64 = 100;
const SEED: u32 = 0;

fn vec_from_u64_to_asset_id<T: Config>(vec: Vec<u64>) -> Vec<T::AssetId> {
    let mut result = Vec::new();
    for value in vec {
        result.push(value.into());
    }
    return result;
}

fn assert_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {

  create_class {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap())
  verify {
        assert_event::<T>(Event::ClassCreated { class_id: CLASS_ID.into(), who: caller }.into());
  }

  create_asset {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class before the asset
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

  }: _(RawOrigin::Signed(caller.clone()), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap())
  verify {
        assert_event::<T>(Event::AssetCreated { class_id: CLASS_ID.into(),asset_id: ASSET_ID.into(), who: caller }.into());
  }

  mint {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CLASS_ID.into(), ASSET_ID.into(), MINT_VALUE.into())
  verify {
        assert_event::<T>(Event::Mint { class_id: CLASS_ID.into(),asset_id: ASSET_ID.into(), who: caller.clone(), to: caller, amount: MINT_VALUE.into() }.into());
  }

  batch_mint {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Set the asset to be minted
    let asset_ids: Vec<u64> = [100,200,300].to_vec();
    let mint_amounts: Vec<u128>= [ 1000, 2000, 3000].to_vec();

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    for value in asset_ids.to_vec() {
      let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), value.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    }

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CLASS_ID.into(), vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), mint_amounts.to_vec())
  verify {
        assert_event::<T>(Event::BatchMint { who: caller.clone(), to: caller, class_id: CLASS_ID.into(),asset_ids: vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), amounts: mint_amounts.to_vec() }.into());
  }

  burn {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

    //Mint the assets
    let _= Assets::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), ASSET_ID.into(), MINT_VALUE.into());

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CLASS_ID.into(), ASSET_ID.into(), BURN_VALUE.into())
  verify {
        assert_event::<T>(Event::Burn { class_id: CLASS_ID.into(),asset_id: ASSET_ID.into(), who: caller.clone(), from: caller, amount: BURN_VALUE.into() }.into());
  }

  batch_burn {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Set the asset to be handle
    let asset_ids: Vec<u64> = [100,200,300].to_vec();

    //Set the amounts to be minted
    let mint_amounts: Vec<u128>= [ 1000, 2000, 3000].to_vec();

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    for value in asset_ids.to_vec() {
      let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), value.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    }

    //mint_batch
    let _ = Assets::<T>::batch_mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), mint_amounts.to_vec());

    //Set the amounts to be burned
    let burn_amounts: Vec<u128>= [ 100, 200, 300].to_vec();

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), CLASS_ID.into(), vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), burn_amounts.to_vec())
  verify {
        assert_event::<T>(Event::BatchBurn { who: caller.clone(), from: caller, class_id: CLASS_ID.into(),asset_ids: vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), amounts: burn_amounts.to_vec() }.into());
  }


  transfer_from {
    //Fund the accounts
    let caller: T::AccountId = whitelisted_caller();
    let reciever: T::AccountId = account("1", 0, SEED);
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

    //Mint the assets
    let _= Assets::<T>::mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), ASSET_ID.into(), MINT_VALUE.into());

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), reciever.clone(), CLASS_ID.into(), ASSET_ID.into(), TRANSFER_VALUE.into())
  verify {
        assert_event::<T>(Event::Transferred { who: caller.clone(), from: caller, to: reciever, class_id: CLASS_ID.into(),asset_id: ASSET_ID.into(), amount: TRANSFER_VALUE.into()}.into());
  }

  batch_transfer_from {
    //Fund the account
    let caller: T::AccountId = whitelisted_caller();
    let reciever: T::AccountId = account("1", 0, SEED);
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Set the asset to be handle
    let asset_ids: Vec<u64> = [100,200,300].to_vec();

    //Set the amounts to be minted
    let mint_amounts: Vec<u128>= [ 1000, 2000, 3000].to_vec();

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    for value in asset_ids.to_vec() {
      let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), value.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    }

    //mint_batch
    let _ = Assets::<T>::batch_mint(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), mint_amounts.to_vec());

    //Set the amounts to be burned
    let transfer_amounts: Vec<u128>= [ 100, 200, 300].to_vec();

  }: _(RawOrigin::Signed(caller.clone()), caller.clone(), reciever.clone(), CLASS_ID.into(), vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), transfer_amounts.to_vec())
  verify {
        assert_event::<T>(Event::BatchTransferred { who: caller.clone(), from: caller, to: reciever,class_id: CLASS_ID.into(),asset_ids: vec_from_u64_to_asset_id::<T>(asset_ids.to_vec()), amounts: transfer_amounts.to_vec() }.into());
  }


  update_class_metadata {
    //Fund the accounts
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

  }: _(RawOrigin::Signed(caller.clone()), CLASS_ID.into(), BoundedVec::try_from([1].to_vec()).unwrap())

  update_asset_metadata {
    //Fund the accounts
    let caller: T::AccountId = whitelisted_caller();
    let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
    T::Currency::make_free_balance_be(&caller, sugar_value);

    //Create the class and the asset before minting
    let _ = Assets::<T>::create_class(RawOrigin::Signed(caller.clone()).into(), caller.clone(), CLASS_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());
    let _ = Assets::<T>::create_asset(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([0].to_vec()).unwrap());

  }: _(RawOrigin::Signed(caller.clone()), CLASS_ID.into(), ASSET_ID.into(), BoundedVec::try_from([1].to_vec()).unwrap())

  impl_benchmark_test_suite!(Assets, crate::mock::new_test_ext(), crate::mock::Test);
}

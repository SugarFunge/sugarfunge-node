#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Bundle;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller};
use frame_support::sp_runtime::SaturatedConversion;
use frame_support::storage::bounded_vec::BoundedVec;
use frame_support::BoundedVec;
use frame_system::RawOrigin;

const BASE_SUGAR: u128 = 100000000000000000000000000;
const MINT_VALUE: u128 = 1000000000000;
const BURN_VALUE: u128 = 500000000000;
const TRANSFER_VALUE: u128 = 500;
const CLASS_ID: u64 = 100;
const ASSET_ID: u64 = 100;
const SEED: u32 = 0;

fn assert_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_has_event(generic_event.into());
}

benchmarks! {

    register_bundle {
        // Fund the account
        let caller: T::AccountId = whitelisted_caller();
        let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
        T::Currency::make_free_balance_be(&caller, sugar_value);

        //create the schema
        let schema: BoundedVec<BoundedVec<u64>, BoundedVec<BoundedVec<u64>>, BoundedVec<BoundedVec<u128>>> =
            BoundedVec::try_from([
                BoundedVec::try_from([1].to_vec()).unwrap(),
                BoundedVec::try_from([
                    BoundedVec::try_from([1].to_vec()).unwrap()
                ].to_vec()).unwrap(),
                BoundedVec::try_from([
                    BoundedVec::try_from([2].to_vec()).unwrap()
                ].to_vec()).unwrap(),
            ].to_vec()).unwrap();

        let bundle_id = schema;

    }: _(RawOrigin::Signed(caller.clone()), CLASS_ID.into(),ASSET_ID.into(),bundle_id, schema, BoundedVec::try_from([0].to_vec()).unwrap())
    verify {
          assert_event::<T>(Event::Register { bundle_id: bundleid , class_id: CLASS_ID.into(), asset_id: ASSET_ID.into(), who: caller }.into());
    }

    mint_bundle {
        //Fund the account
        let caller: T::AccountId = whitelisted_caller();
        let reciever: T::AccountId = account("1", 0, SEED);
        let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
        T::Currency::make_free_balance_be(&caller, sugar_value);

        //Create the schema
        let schema: BoundedVec<BoundedVec<u64>, BoundedVec<BoundedVec<u64>>, BoundedVec<BoundedVec<u128>>> =
          BoundedVec::try_from([
              BoundedVec::try_from([1].to_vec()).unwrap(),
              BoundedVec::try_from([
                  BoundedVec::try_from([1].to_vec()).unwrap()
              ].to_vec()).unwrap(),
              BoundedVec::try_from([
                  BoundedVec::try_from([2].to_vec()).unwrap()
              ].to_vec()).unwrap(),
          ].to_vec()).unwrap();

        let bundle_id = schema;

        //Create the bundle
        let _ = Bundle::<T>::register_bundle(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(),ASSET_ID.into(),bundle_id, schema, BoundedVec::try_from([0].to_vec()).unwrap());

      }: _(RawOrigin::Signed(caller.clone()), caller.clone(), reciever.clone(), bundle_id, MINT_VALUE.into())
      verify {
            assert_event::<T>(Event::Mint {who: caller.clone(), from: caller, to: reciever, bundle_id: bundle_id, amount: MINT_VALUE.into()}.into());
      }

      burn_bundle {
        //Fund the account
        let caller: T::AccountId = whitelisted_caller();
        let reciever: T::AccountId = account("1", 0, SEED);
        let sugar_value: BalanceOf<T> = BASE_SUGAR.saturated_into::<BalanceOf<T>>();
        T::Currency::make_free_balance_be(&caller, sugar_value);

        //Create the schema
        let schema: BoundedVec<BoundedVec<u64>, BoundedVec<BoundedVec<u64>>, BoundedVec<BoundedVec<u128>>> =
          BoundedVec::try_from([
              BoundedVec::try_from([1].to_vec()).unwrap(),
              BoundedVec::try_from([
                  BoundedVec::try_from([1].to_vec()).unwrap()
              ].to_vec()).unwrap(),
              BoundedVec::try_from([
                  BoundedVec::try_from([2].to_vec()).unwrap()
              ].to_vec()).unwrap(),
          ].to_vec()).unwrap();

        let bundle_id = schema;

        //Create the bundle and mint it
        let _ = Bundle::<T>::register_bundle(RawOrigin::Signed(caller.clone()).into(), CLASS_ID.into(),ASSET_ID.into(),bundle_id, schema, BoundedVec::try_from([0].to_vec()).unwrap());
        let _ = Bundle::<T>::mint_bundle(RawOrigin::Signed(caller.clone()).into(), caller.clone(), reciever.clone(), bundle_id, MINT_VALUE.into());
      }: _(RawOrigin::Signed(caller.clone()), caller.clone(), reciever.clone(), bundle_id, BURN_VALUE.into())
      verify {
            assert_event::<T>(Event::Burn {who: caller.clone(), from: caller, to: reciever, bundle_id: bundle_id, amount: MINT_VALUE.into()}.into());
      }
    impl_benchmark_test_suite!(Bundle, crate::mock::new_test_ext(), crate::mock::Test);
}

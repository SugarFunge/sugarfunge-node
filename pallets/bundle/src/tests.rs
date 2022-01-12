use crate::{mock::*, BundleSchema, Error};
use frame_support::{assert_err, assert_ok, BoundedVec};
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::prelude::*;

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_bundle() {
    run_to_block(10);

    assert_ok!(Asset::do_create_class(&1, 2000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, 3000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, 4000, [0].to_vec()));

    let asset_ids = [1, 2, 3, 4, 5].to_vec();
    let amounts = [100, 200, 300, 400, 500].to_vec();

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        2000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        3000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        4000,
        asset_ids.clone(),
        amounts.clone(),
    ));
}

#[test]
fn create_bundle_works() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids = [1, 2, 3, 4, 5].to_vec();

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone().try_into().unwrap();
        let bamounts: BoundedVec<u128, MaxAssets> = vec![1, 2, 3, 4, 5].try_into().unwrap();

        let schema: BundleSchema<Test> = (
            vec![2000, 3000, 4000].try_into().unwrap(),
            vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()]
                .try_into()
                .unwrap(),
            vec![bamounts.clone(), bamounts.clone(), bamounts.clone()]
                .try_into()
                .unwrap(),
        );

        let (bundle_id, bundle_account) = Bundle::do_create_bundles(&2, &schema, 10).unwrap();
        assert_eq!(BlakeTwo256::hash_of(&schema), bundle_id);

        assert_eq!(
            last_event(),
            Event::Bundle(crate::Event::Created(bundle_id, 2, bundle_account, 10)),
        );

        assert_eq!(
            vec![10, 20, 30, 40, 50,],
            Asset::balance_of_single_owner_batch(&bundle_account, 4000, asset_ids.clone()).unwrap()
        );

        assert_eq!(
            vec![90, 180, 270, 360, 450,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.clone()).unwrap()
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);
        let (a_account_id, a_balance) = Bundle::balances((2, bundle_id));
        assert_eq!(bundle_account, a_account_id);
        assert_eq!(10, a_balance);
    })
}

#[test]
fn create_bundle_fails() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids = [1, 2, 3, 4, 5].to_vec();

        assert_ok!(Asset::do_transfer_from(&2, &2, &1, 4000, 1, 91));

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone().try_into().unwrap();
        let bamounts: BoundedVec<u128, MaxAssets> = vec![1, 2, 3, 4, 5].try_into().unwrap();

        let schema: BundleSchema<Test> = (
            vec![2000, 3000, 4000].try_into().unwrap(),
            vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()]
                .try_into()
                .unwrap(),
            vec![bamounts.clone(), bamounts.clone(), bamounts.clone()]
                .try_into()
                .unwrap(),
        );

        assert_err!(
            Bundle::do_create_bundles(&2, &schema, 10),
            Error::<Test>::InsufficientBalance
        );

        assert_eq!(
            vec![9, 200, 300, 400, 500,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.clone()).unwrap()
        );
    })
}

#[test]
fn create_same_bundle_adds_to_existing_bundle() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids = [1, 2, 3, 4, 5].to_vec();

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone().try_into().unwrap();
        let bamounts: BoundedVec<u128, MaxAssets> = vec![1, 2, 3, 4, 5].try_into().unwrap();

        let schema: BundleSchema<Test> = (
            vec![2000, 3000, 4000].try_into().unwrap(),
            vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()]
                .try_into()
                .unwrap(),
            vec![bamounts.clone(), bamounts.clone(), bamounts.clone()]
                .try_into()
                .unwrap(),
        );

        let (bundle_id, first_bundle_account) = Bundle::do_create_bundles(&2, &schema, 10).unwrap();
        assert_eq!(BlakeTwo256::hash_of(&schema), bundle_id);

        let (bundle_id, second_bundle_account) =
            Bundle::do_create_bundles(&2, &schema, 10).unwrap();
        assert_eq!(BlakeTwo256::hash_of(&schema), bundle_id);

        assert_eq!(first_bundle_account, second_bundle_account);

        let bundle_account = first_bundle_account;

        assert_eq!(
            vec![20, 40, 60, 80, 100,],
            Asset::balance_of_single_owner_batch(&bundle_account, 4000, asset_ids.clone()).unwrap()
        );

        assert_eq!(
            vec![80, 160, 240, 320, 400,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.clone()).unwrap()
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);
        let (a_account_id, a_balance) = Bundle::balances((2, bundle_id));
        assert_eq!(bundle_account, a_account_id);
        assert_eq!(20, a_balance);
    })
}

#[test]
fn before_bundle_works() {
    new_test_ext().execute_with(|| {
        before_bundle();
    })
}

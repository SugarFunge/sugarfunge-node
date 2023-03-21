use crate::{mock::*, BundleSchema, Error};
use frame_support::{assert_err, assert_ok, bounded_vec, BoundedVec};
use sp_runtime::traits::{BlakeTwo256, Hash};

fn last_event() -> RuntimeEvent {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_bundle() {
    run_to_block(10);

    assert_ok!(Asset::do_create_class(&1, &1, 2000, bounded_vec![0]));
    assert_ok!(Asset::do_create_class(&1, &1, 3000, bounded_vec![0]));
    assert_ok!(Asset::do_create_class(&1, &1, 4000, bounded_vec![0]));

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
fn mint_bundle_works() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids: BoundedVec<u64, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone();
        let bamounts: BoundedVec<u128, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let schema: BundleSchema<Test> = (
            vec![2000, 3000, 4000].try_into().unwrap(),
            vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()]
                .try_into()
                .unwrap(),
            vec![bamounts.clone(), bamounts.clone(), bamounts.clone()]
                .try_into()
                .unwrap(),
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);

        assert_ok!(Bundle::do_register_bundle(
            &1,
            9000,
            0,
            bundle_id,
            &schema,
            bounded_vec![]
        ));
        assert_eq!(Bundle::asset_bundles((9000, 0)), Some(bundle_id));

        Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10).unwrap();

        let bundle = Bundle::bundles(bundle_id).unwrap();

        let bundle_asset_balance = Asset::balance_of(&2, 9000, 0);
        assert_eq!(10, bundle_asset_balance);

        assert_eq!(
            last_event(),
            RuntimeEvent::Bundle(crate::Event::Mint {
                bundle_id,
                who: 2,
                from: 2,
                to: 2,
                amount: 10
            }),
        );

        assert_eq!(
            vec![10, 20, 30, 40, 50,],
            Asset::balance_of_single_owner_batch(&bundle.vault, 4000, asset_ids.to_vec()).unwrap()
        );

        assert_eq!(
            vec![90, 180, 270, 360, 450,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.to_vec()).unwrap()
        );
    })
}

#[test]
fn mint_bundle_fails() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids: BoundedVec<u64, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        assert_ok!(Asset::do_transfer_from(&2, &2, &1, 4000, 1, 91));

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone();
        let bamounts: BoundedVec<u128, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let schema: BundleSchema<Test> = (
            bounded_vec![2000, 3000, 4000],
            bounded_vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()],
            bounded_vec![bamounts.clone(), bamounts.clone(), bamounts.clone()],
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);

        assert_ok!(Bundle::do_register_bundle(
            &1,
            9000,
            0,
            bundle_id,
            &schema,
            bounded_vec![]
        ));

        assert_err!(
            Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10),
            Error::<Test>::InsufficientBalance
        );

        assert_eq!(
            vec![9, 200, 300, 400, 500,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.to_vec()).unwrap()
        );
    })
}

#[test]
fn add_assets_to_existing_bundle() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids: BoundedVec<u64, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone();
        let bamounts: BoundedVec<u128, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let schema: BundleSchema<Test> = (
            bounded_vec![2000, 3000, 4000],
            bounded_vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()],
            bounded_vec![bamounts.clone(), bamounts.clone(), bamounts.clone()],
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);

        assert_ok!(Bundle::do_register_bundle(
            &1,
            9000,
            0,
            bundle_id,
            &schema,
            bounded_vec![]
        ));

        Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10).unwrap();
        Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10).unwrap();

        let bundle = Bundle::bundles(bundle_id).unwrap();

        let bundle_asset_balance = Asset::balance_of(&2, 9000, 0);
        assert_eq!(20, bundle_asset_balance);

        assert_eq!(
            vec![20, 40, 60, 80, 100,],
            Asset::balance_of_single_owner_batch(&bundle.vault, 4000, asset_ids.to_vec()).unwrap()
        );

        assert_eq!(
            vec![80, 160, 240, 320, 400,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.to_vec()).unwrap()
        );
    })
}

#[test]
fn burn_bundle_works() {
    new_test_ext().execute_with(|| {
        before_bundle();

        let asset_ids: BoundedVec<u64, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let basset_ids: BoundedVec<u64, MaxAssets> = asset_ids.clone();
        let bamounts: BoundedVec<u128, MaxAssets> = bounded_vec![1, 2, 3, 4, 5];

        let schema: BundleSchema<Test> = (
            bounded_vec![2000, 3000, 4000],
            bounded_vec![basset_ids.clone(), basset_ids.clone(), basset_ids.clone()],
            bounded_vec![bamounts.clone(), bamounts.clone(), bamounts.clone()],
        );

        let bundle_id = BlakeTwo256::hash_of(&schema);

        assert_ok!(Bundle::do_register_bundle(
            &1,
            9000,
            0,
            bundle_id,
            &schema,
            bounded_vec![]
        ));

        Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10).unwrap();
        Bundle::do_mint_bundles(&2, &2, &2, bundle_id, 10).unwrap();

        let bundle = Bundle::bundles(bundle_id).unwrap();

        let bundle_asset_balance = Asset::balance_of(&2, 9000, 0);
        assert_eq!(20, bundle_asset_balance);

        assert_eq!(
            vec![20, 40, 60, 80, 100,],
            Asset::balance_of_single_owner_batch(&bundle.vault, 4000, asset_ids.to_vec()).unwrap()
        );

        assert_eq!(
            vec![80, 160, 240, 320, 400,],
            Asset::balance_of_single_owner_batch(&2, 4000, asset_ids.to_vec()).unwrap()
        );

        assert_ok!(Bundle::do_burn_bundles(&2, &2, &2, bundle_id, 5));

        assert_eq!(
            last_event(),
            RuntimeEvent::Bundle(crate::Event::Burn {
                bundle_id,
                who: 2,
                from: 2,
                to: 2,
                amount: 5
            }),
        );

        let bundle_asset_balance = Asset::balance_of(&2, 9000, 0);
        assert_eq!(15, bundle_asset_balance);

        assert_eq!(
            vec![15, 30, 45, 60, 75,],
            Asset::balance_of_single_owner_batch(&bundle.vault, 4000, asset_ids.to_vec()).unwrap()
        );
    })
}

#[test]
fn before_bundle_works() {
    new_test_ext().execute_with(|| {
        before_bundle();
    })
}

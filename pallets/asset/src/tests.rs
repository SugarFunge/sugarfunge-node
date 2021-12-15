use crate::{mock::*, pallet::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_asset_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Asset::do_create_class(&1, data));
        let uri = vec![0, 1];
        assert_ok!(Asset::do_create_asset(&1, 0, 2, uri));
        println!("asset: {:?}", Assets::<Test>::get(0, 2));
    })
}

#[test]
fn create_asset_not_works() {
    new_test_ext().execute_with(|| {
        let uri = vec![0, 1];
        assert_noop!(
            Asset::do_create_asset(&1, 1, 2, uri),
            Error::<Test>::InvalidClassId
        );
        println!("asset: {:?}", Assets::<Test>::get(1, 2));
    })
}

#[test]
fn create_class_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Asset::do_create_class(&1, data));
    })
}

#[test]
fn do_set_approval_for_all() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_set_approval_for_all(&1, &2, 1, true));
        assert_ok!(Asset::do_set_approval_for_all(&1, &2, 1, false));
    })
}

#[test]
fn do_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        let asset_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Asset::do_batch_mint(&1, &2, 1, asset_ids, amounts));
    })
}

#[test]
fn do_burn_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
        assert_ok!(Asset::do_burn(&1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        let asset_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_burn(&1, &2, 1, asset_ids, amounts));
    })
}

#[test]
fn do_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_mint(&1, &1, 1, 2, 100));
        assert_ok!(Asset::do_transfer_from(&1, &1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        let asset_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Asset::do_batch_mint(
            &1,
            &1,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_transfer_from(
            &1, &1, &2, 1, asset_ids, amounts
        ));
    })
}

#[test]
fn approved_or_owner() {
    new_test_ext().execute_with(|| {
        assert_eq!(Asset::approved_or_owner(&1, &2, 1), false);
    })
}

#[test]
fn is_approved_for_all() {
    new_test_ext().execute_with(|| {
        assert_eq!(Asset::is_approved_for_all(&1, &2, 1), false);
    })
}

#[test]
fn balance_of() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
        assert_eq!(Asset::balance_of(&2, 1, 2), 100);
    })
}

#[test]
fn balance_of_batch_asset_ids_sample() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        let asset_ids = vec![1; 3];
        let amounts = vec![100; 3];

        assert_ok!(Asset::do_batch_mint(
            &1,
            &1,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));

        let account = vec![1, 2, 3];
        assert_eq!(
            Asset::balance_of_batch(&account, 1, asset_ids).unwrap(),
            vec![300; 3]
        );
    })
}

#[test]
fn balance_of_batch_asset_ids_not_sample() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, [0].to_vec()));
        let asset_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];

        assert_ok!(Asset::do_batch_mint(
            &1,
            &1,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));

        let account = vec![1, 2, 3];
        assert_eq!(
            Asset::balance_of_batch(&account, 1, asset_ids).unwrap(),
            amounts
        );
    })
}
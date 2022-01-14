use crate::{
    mock::*,
    pallet::{Assets, Classes},
    Error,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_asset_works() {
    new_test_ext().execute_with(|| {
        let data = vec![0, 1];
        assert_ok!(Asset::do_create_class(&1, &1, 1, data));
        let metadata = vec![0, 1];
        assert_ok!(Asset::do_create_asset(&1, 1, 2, metadata));
    })
}

#[test]
fn create_asset_not_works() {
    new_test_ext().execute_with(|| {
        let metadata = vec![0, 1];
        assert_noop!(
            Asset::do_create_asset(&1, 1, 2, metadata),
            Error::<Test>::InvalidClassId
        );
        println!("asset: {:?}", Assets::<Test>::get(1, 2));
    })
}

#[test]
fn create_class_works() {
    new_test_ext().execute_with(|| {
        let metadata = vec![0, 1];
        assert_ok!(Asset::do_create_class(&1, &1, 1, metadata));
    })
}

#[test]
fn update_class_metadata() {
    new_test_ext().execute_with(|| {
        let metadata = vec![0, 1];
        assert_ok!(Asset::do_create_class(&1, &1, 1, metadata.clone()));
        let class = Classes::<Test>::get(1).unwrap();
        assert_eq!(class.metadata, metadata);
        let new_metadata = vec![0, 1, 2, 3, 4];
        assert_ok!(Asset::do_update_class_metadata(&1, 1, new_metadata.clone()));
        let class = Classes::<Test>::get(1).unwrap();
        assert_eq!(class.metadata, new_metadata);
    })
}

#[test]
fn update_asset_metadata() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        let metadata = vec![0, 1];
        assert_ok!(Asset::do_create_asset(&1, 1, 1000, metadata.clone()));
        let asset = Assets::<Test>::get(1, 1000).unwrap();
        assert_eq!(asset.metadata, metadata);
        let new_metadata = vec![0, 1, 2, 3, 4];
        assert_ok!(Asset::do_update_asset_metadata(
            &1,
            1,
            1000,
            new_metadata.clone()
        ));
        let asset = Assets::<Test>::get(1, 1000).unwrap();
        assert_eq!(asset.metadata, new_metadata);
    })
}

// #[test]
// fn do_set_operator_approval_for_all() {
//     new_test_ext().execute_with(|| {
//         assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
//         assert_ok!(Asset::do_create_class(&1, &1, 2, [0].to_vec()));
//         assert_ok!(Asset::do_set_operator_approval_for_all(&1, &2, 1, true));
//         assert_ok!(Asset::do_set_operator_approval_for_all(&1, &2, 1, false));
//     })
// }

#[test]
fn do_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_mint() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));
        let asset_ids = vec![1, 2, 3];
        let amounts = vec![100; 3];
        assert_ok!(Asset::do_batch_mint(&1, &2, 1, asset_ids, amounts));
    })
}

#[test]
fn do_burn_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
        assert_ok!(Asset::do_burn(&1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_burn() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));

        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));

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
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_mint(&1, &1, 1, 2, 100));
        assert_ok!(Asset::do_transfer_from(&1, &1, &2, 1, 2, 100));
    })
}

#[test]
fn do_batch_transfer_from() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));
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

// #[test]
// fn approved_or_owner() {
//     new_test_ext().execute_with(|| {
//         assert_eq!(Asset::operator_approved_or_account_owner(&1, &2, 1), false);
//     })
// }

// #[test]
// fn is_approved_for_all() {
//     new_test_ext().execute_with(|| {
//         assert_eq!(Asset::is_operator_approved_for_all(&1, &2, 1), false);
//     })
// }

#[test]
fn balance_of() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_mint(&1, &2, 1, 2, 100));
        assert_eq!(Asset::balance_of(&2, 1, 2), 100);
    })
}

#[test]
fn balance_of_batch_asset_ids_sample() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));

        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));

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
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));

        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));

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

#[test]
fn do_iter_all_balances_of_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 2, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 3, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 4, [0].to_vec()));

        assert_ok!(Asset::do_create_asset(&1, 1, 1, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 2, vec![0]));
        assert_ok!(Asset::do_create_asset(&1, 1, 3, vec![0]));

        let asset_ids = vec![1, 2, 3];
        let amounts = vec![1, 20, 300];
        assert_ok!(Asset::do_batch_mint(&1, &2, 1, asset_ids, amounts));

        let account_id = 2;
        let mut balances = Asset::balances_of_owner(&account_id).unwrap();
        balances.sort();

        let expected_balances = vec![(1, 1, 1), (1, 2, 20), (1, 3, 300)];
        assert_eq!(balances, expected_balances);
    })
}

#[test]
fn do_iter_class_balances_of_owner() {
    new_test_ext().execute_with(|| {
        assert_ok!(Asset::do_create_class(&1, &1, 1, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 2, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 3, [0].to_vec()));
        assert_ok!(Asset::do_create_class(&1, &1, 4, [0].to_vec()));

        let asset_ids = vec![1, 2, 3];
        let amounts = vec![1, 20, 300];

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            1,
            asset_ids.clone(),
            amounts.clone()
        ));
        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            2,
            asset_ids.clone(),
            amounts.clone()
        ));

        let account_id = 2;
        let class_id = 1;
        let mut balances = Asset::class_balances_of_owner(&account_id, class_id).unwrap();
        balances.sort();

        let expected_balances = vec![(1, 1), (2, 20), (3, 300)];
        assert_eq!(balances, expected_balances);
    })
}

use crate::mock::Asset;
use crate::mock::*;
use frame_support::{assert_ok, bounded_vec};

fn test_create_class() {
    assert_ok!(Asset::do_create_class(&1, &1, 2000, bounded_vec![0]));
}

#[test]
fn test_verify_class_id_successful() {
    new_test_ext().execute_with(|| {
        test_create_class();
        assert_eq!(Asset::class_exists(2000), true);
    })
}

#[test]
fn test_verify_class_id_not_initialized() {
    new_test_ext().execute_with(|| {
        assert_eq!(Asset::class_exists(2000), false);
    })
}

#[test]
fn test_verify_class_id_failed() {
    new_test_ext().execute_with(|| {
        test_create_class();
        assert_eq!(Asset::class_exists(2001), false);
    })
}

#[test]

fn test_verify_account_owner_successful() {
    new_test_ext().execute_with(|| {
        test_create_class();
        assert_eq!(Asset::account_is_owner(&1, 2000), true);
    })
}

#[test]

fn test_verify_account_owner_no_initialized() {
    new_test_ext().execute_with(|| {
        assert_eq!(Asset::account_is_owner(&1, 2000), false);
    })
}

#[test]

fn test_verify_account_owner_failed_wrong_account() {
    new_test_ext().execute_with(|| {
        test_create_class();
        assert_eq!(Asset::account_is_owner(&2, 2000), false);
    })
}

#[test]

fn test_verify_account_owner_failed_wrong_class_id() {
    new_test_ext().execute_with(|| {
        test_create_class();
        assert_eq!(Asset::account_is_owner(&2, 2000), false);
    })
}

//! Unit tests for the non-fungible-token module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn create_collection_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
    });
}

#[test]
fn create_collection_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        NextCollectionId::<Runtime>::mutate(|id| {
            *id = <Runtime as Config>::CollectionId::max_value()
        });
        assert_noop!(
            NonFungibleTokenModule::create_collection(&ALICE, vec![1], ()),
            Error::<Runtime>::NoAvailableCollectionId
        );
    });
}

#[test]
fn mint_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        let next_collection_id = NonFungibleTokenModule::next_collection_id();
        assert_eq!(next_collection_id, CLASS_ID);
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 0);
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 1);
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 2);

        let next_collection_id = NonFungibleTokenModule::next_collection_id();
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_eq!(NonFungibleTokenModule::next_token_id(next_collection_id), 0);
        assert_ok!(NonFungibleTokenModule::mint(
            &BOB,
            next_collection_id,
            vec![1],
            ()
        ));
        assert_eq!(NonFungibleTokenModule::next_token_id(next_collection_id), 1);

        assert_eq!(NonFungibleTokenModule::next_token_id(CLASS_ID), 2);
    });
}

#[test]
fn mint_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        Collections::<Runtime>::mutate(CLASS_ID, |collection_info| {
            collection_info.as_mut().unwrap().total_issuance =
                <Runtime as Config>::TokenId::max_value();
        });
        assert_noop!(
            NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()),
            Error::<Runtime>::NumOverflow
        );

        NextTokenId::<Runtime>::mutate(CLASS_ID, |id| {
            *id = <Runtime as Config>::TokenId::max_value()
        });
        assert_noop!(
            NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()),
            Error::<Runtime>::NoAvailableTokenId
        );
    });
}

#[test]
fn transfer_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_ok!(NonFungibleTokenModule::transfer(
            &BOB,
            &BOB,
            (CLASS_ID, TOKEN_ID)
        ));
        assert_ok!(NonFungibleTokenModule::transfer(
            &BOB,
            &ALICE,
            (CLASS_ID, TOKEN_ID)
        ));
        assert_ok!(NonFungibleTokenModule::transfer(
            &ALICE,
            &BOB,
            (CLASS_ID, TOKEN_ID)
        ));
        assert!(NonFungibleTokenModule::is_owner(&BOB, (CLASS_ID, TOKEN_ID)));
    });
}

#[test]
fn transfer_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            NonFungibleTokenModule::transfer(&BOB, &ALICE, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenNotFound
        );
        assert_noop!(
            NonFungibleTokenModule::transfer(&ALICE, &BOB, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );
        assert_noop!(
            NonFungibleTokenModule::mint(&BOB, CLASS_ID_NOT_EXIST, vec![1], ()),
            Error::<Runtime>::CollectionNotFound
        );
        assert_noop!(
            NonFungibleTokenModule::transfer(&ALICE, &ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );
    });
}

#[test]
fn burn_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_ok!(NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID)));
    });
}

#[test]
fn burn_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID_NOT_EXIST)),
            Error::<Runtime>::TokenNotFound
        );

        assert_noop!(
            NonFungibleTokenModule::burn(&ALICE, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NoPermission
        );
    });

    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));

        Collections::<Runtime>::mutate(CLASS_ID, |collection_info| {
            collection_info.as_mut().unwrap().total_issuance = 0;
        });
        assert_noop!(
            NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID)),
            Error::<Runtime>::NumOverflow
        );
    });
}

#[test]
fn destroy_collection_should_work() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_ok!(NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID)));
        assert_ok!(NonFungibleTokenModule::destroy_collection(&ALICE, CLASS_ID));
        assert_eq!(Collections::<Runtime>::contains_key(CLASS_ID), false);
        assert_eq!(NextTokenId::<Runtime>::contains_key(CLASS_ID), false);
    });
}

#[test]
fn destroy_collection_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(NonFungibleTokenModule::create_collection(
            &ALICE,
            vec![1],
            ()
        ));
        assert_ok!(NonFungibleTokenModule::mint(&BOB, CLASS_ID, vec![1], ()));
        assert_noop!(
            NonFungibleTokenModule::destroy_collection(&ALICE, CLASS_ID_NOT_EXIST),
            Error::<Runtime>::CollectionNotFound
        );

        assert_noop!(
            NonFungibleTokenModule::destroy_collection(&BOB, CLASS_ID),
            Error::<Runtime>::NoPermission
        );

        assert_noop!(
            NonFungibleTokenModule::destroy_collection(&ALICE, CLASS_ID),
            Error::<Runtime>::CannotDestroyCollection
        );

        assert_ok!(NonFungibleTokenModule::burn(&BOB, (CLASS_ID, TOKEN_ID)));
        assert_ok!(NonFungibleTokenModule::destroy_collection(&ALICE, CLASS_ID));
        assert_eq!(Collections::<Runtime>::contains_key(CLASS_ID), false);
    });
}

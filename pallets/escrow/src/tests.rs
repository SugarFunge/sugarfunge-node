use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_escrow() {
    run_to_block(10);
    assert_ok!(Currency::mint(
        Origin::signed(1),
        SUGAR,
        500 * DOLLARS
    ));
    assert_eq!(
        last_event(),
        Event::Currency(sugarfunge_currency::Event::AssetMint(
            SUGAR,
            500 * DOLLARS,
            1
        )),
    );
    assert_eq!(Asset::balance_of(&1, 0, SUGAR.into()), 500 * DOLLARS);
    assert_ok!(Asset::create_class(Origin::signed(1), 1, [0].to_vec()));
    assert_ok!(Asset::create_asset(Origin::signed(1), 1, 1, [0].to_vec()));
    assert_ok!(Asset::mint(Origin::signed(1), 1, 1, 1, 50000 * DOLLARS));
    assert_eq!(Asset::balance_of(&1, 1, 1), 50000 * DOLLARS);
}

#[test]
fn before_escrow_works() {
    new_test_ext().execute_with(|| {
        before_escrow();
    })
}

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(Escrow::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(Escrow::something(), Some(42));
    });
}

#[test]
fn correct_error_for_none_value() {
    new_test_ext().execute_with(|| {
        // Ensure the expected error is thrown when no value is present.
        assert_noop!(
            Escrow::cause_error(Origin::signed(1)),
            Error::<Test>::NoneValue
        );
    });
}

use super::*;
use crate::mock::*;
use frame_support::assert_ok;

fn last_event() -> mock::Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

#[test]
fn currency_eth_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Currency::mint(Origin::signed(1), ETH, 500 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(crate::Event::TokenMint(ETH, 500 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, ETH.into()), 500 * CENTS);
    })
}

#[test]
fn currency_btc_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Currency::mint(Origin::signed(1), BTC, 500 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(crate::Event::TokenMint(BTC, 500 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, BTC.into()), 500 * CENTS);
    })
}

#[test]
fn currency_mint_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Currency::mint(Origin::signed(1), SUGAR, 500 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(crate::Event::TokenMint(SUGAR, 500 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, SUGAR.into()), 500 * CENTS);
    })
}

#[test]
fn currency_burn_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Currency::mint(Origin::signed(1), SUGAR, 500 * CENTS));
        assert_ok!(Currency::burn(Origin::signed(1), SUGAR, 400 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(crate::Event::TokenBurn(SUGAR, 400 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, SUGAR.into()), 100 * CENTS);
    })
}

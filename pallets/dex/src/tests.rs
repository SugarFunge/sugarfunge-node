use super::*;
use crate::mock::*;
use frame_support::assert_ok;

fn last_event() -> mock::Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_exchange() {
    run_to_block(10);
    assert_ok!(CurrencyToken::mint(Origin::signed(1), SUGAR, 500 * CENTS));
    assert_eq!(
        last_event(),
        mock::Event::CurrencyToken(sugarfunge_currency::Event::TokenMint(SUGAR, 500 * CENTS, 1)),
    );
    assert_eq!(Token::balance_of(&1, 0, SUGAR.into()), 500 * CENTS);
    assert_ok!(Token::create_collection(Origin::signed(1), [0].to_vec()));
    assert_ok!(Token::create_token(Origin::signed(1), 1, 1, [0].to_vec()));
    assert_ok!(Token::mint(Origin::signed(1), 1, 1, 1, 50000 * CENTS));
    assert_eq!(Token::balance_of(&1, 1, 1), 50000 * CENTS);
}

pub fn endow_user_2() {
    run_to_block(10);
    assert_eq!(Token::balance_of(&2, 0, SUGAR.into()), 0 * CENTS);
    assert_eq!(Token::balance_of(&2, 1, 1), 0 * CENTS);
    assert_ok!(Token::transfer_from(
        Origin::signed(1),
        1,
        2,
        0,
        SUGAR.into(),
        100 * CENTS
    ));
    assert_eq!(
        last_event(),
        mock::Event::Token(sugarfunge_token::Event::Transferred(
            1,
            2,
            0,
            SUGAR.into(),
            100 * CENTS
        )),
    );
    assert_eq!(Token::balance_of(&2, 0, SUGAR.into()), 100 * CENTS);
}

#[test]
fn before_exchange_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
    })
}

#[test]
fn many_currencies_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(CurrencyToken::mint(Origin::signed(1), ETH, 500 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::CurrencyToken(sugarfunge_currency::Event::TokenMint(ETH, 500 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, ETH.into()), 500 * CENTS);
        assert_ok!(CurrencyToken::mint(Origin::signed(1), BTC, 500 * CENTS));
        assert_eq!(
            last_event(),
            mock::Event::CurrencyToken(sugarfunge_currency::Event::TokenMint(BTC, 500 * CENTS, 1)),
        );
        assert_eq!(Token::balance_of(&1, 0, BTC.into()), 500 * CENTS);
    })
}

#[test]
fn endow_user_2_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        endow_user_2();
    })
}

#[test]
fn create_exchange_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), SUGAR, 1));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated(0, 1)),
        );
    })
}

#[test]
fn add_liquidity_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), SUGAR, 1));
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [1000 * CENTS].to_vec(),
            [100 * CENTS].to_vec(),
        ));
        let balances = Dex::get_token_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * CENTS].to_vec());
    });
}

#[test]
fn sell_price_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), SUGAR, 1));
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [1000 * CENTS].to_vec(),
            [100 * CENTS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * CENTS);
        let balances = Dex::get_token_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * CENTS].to_vec());
        let amount = 1 * CENTS;
        let price =
            Dex::get_sell_price(amount, balances[0].saturating_sub(amount), currency_reserve);
        assert_eq!(price, Ok(20306124521033));
    });
}

#[test]
fn buy_price_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), SUGAR, 1));
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [1000 * CENTS].to_vec(),
            [100 * CENTS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * CENTS);
        let balances = Dex::get_token_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * CENTS].to_vec());
        let amount = 1 * CENTS;
        let price = Dex::get_buy_price(amount, balances[0], currency_reserve);
        assert_eq!(price, Ok(4974366783412009543));
    });
}

#[test]
fn buy_tokens_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        endow_user_2();
        assert_ok!(Dex::create_exchange(Origin::signed(1), SUGAR, 1));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated(0, 1)),
        );
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [1000 * CENTS].to_vec(),
            [100 * CENTS].to_vec(),
        ));
        assert_eq!(Token::balance_of(&2, 1, 1), 0 * CENTS);
        assert_ok!(Dex::buy_tokens(
            Origin::signed(2),
            0,
            [1].to_vec(),
            [10 * CENTS].to_vec(),
            1000 * CENTS,
            2
        ));
        assert_eq!(Token::balance_of(&2, 1, 1), 10 * CENTS);
    });
}

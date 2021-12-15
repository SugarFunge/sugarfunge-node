use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_ok};

fn last_event() -> mock::Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_exchange() {
    run_to_block(10);
    assert_ok!(mock::Currency::mint(
        Origin::signed(1),
        SUGAR,
        500 * DOLLARS
    ));
    assert_eq!(
        last_event(),
        mock::Event::Currency(sugarfunge_currency::Event::AssetMint(
            SUGAR,
            500 * DOLLARS,
            1
        )),
    );
    assert_eq!(Asset::balance_of(&1, 0, SUGAR.into()), 500 * DOLLARS);
    assert_ok!(Asset::create_class(Origin::signed(1), [0].to_vec()));
    assert_ok!(Asset::create_asset(Origin::signed(1), 1, 1, [0].to_vec()));
    assert_ok!(Asset::mint(Origin::signed(1), 1, 1, 1, 50000 * DOLLARS));
    assert_eq!(Asset::balance_of(&1, 1, 1), 50000 * DOLLARS);
}

pub fn endow_user_2() {
    run_to_block(10);
    assert_eq!(Asset::balance_of(&2, 0, SUGAR.into()), 0 * DOLLARS);
    assert_eq!(Asset::balance_of(&2, 1, 1), 0 * DOLLARS);
    assert_ok!(Asset::transfer_from(
        Origin::signed(1),
        1,
        2,
        0,
        SUGAR.into(),
        100 * DOLLARS
    ));
    assert_eq!(
        last_event(),
        mock::Event::Asset(sugarfunge_asset::Event::Transferred(
            1,
            2,
            0,
            SUGAR.into(),
            100 * DOLLARS
        )),
    );
    assert_eq!(Asset::balance_of(&2, 0, SUGAR.into()), 100 * DOLLARS);
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
        assert_ok!(mock::Currency::mint(Origin::signed(1), ETH, 500 * DOLLARS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(sugarfunge_currency::Event::AssetMint(ETH, 500 * DOLLARS, 1)),
        );
        assert_eq!(Asset::balance_of(&1, 0, ETH.into()), 500 * DOLLARS);
        assert_ok!(mock::Currency::mint(Origin::signed(1), BTC, 500 * DOLLARS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(sugarfunge_currency::Event::AssetMint(BTC, 500 * DOLLARS, 1)),
        );
        assert_eq!(Asset::balance_of(&1, 0, BTC.into()), 500 * DOLLARS);
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
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated(0, 1)),
        );
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            2, // Give out liquidity assets to account 2
            [1].to_vec(),
            [1000 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        let balances = Dex::get_asset_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * DOLLARS].to_vec());
        let exchange = Exchanges::<Test>::get(0).unwrap();
        assert_eq!(
            Asset::balance_of(&2, exchange.lp_class_id, 1),
            100 * DOLLARS
        );
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
            [1000 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * DOLLARS);
        let balances = Dex::get_asset_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * DOLLARS].to_vec());
        let amount = 1 * DOLLARS;
        let price =
            Dex::get_sell_price(amount, balances[0].saturating_sub(amount), currency_reserve);
        assert_eq!(price, Ok(0_002030612452103311));
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
            [1000 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * DOLLARS);
        let balances = Dex::get_asset_reserves(&1, 1, [1].to_vec());
        assert_eq!(balances, [49000 * DOLLARS].to_vec());
        let amount = 1 * DOLLARS;
        let price = Dex::get_buy_price(amount, balances[0], currency_reserve);
        assert_eq!(price, Ok(497_436678341200954267));
    });
}

#[test]
fn buy_assets_works() {
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
            [1000 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        assert_eq!(Asset::balance_of(&2, 1, 1), 0 * DOLLARS);
        assert_ok!(Dex::buy_assets(
            Origin::signed(2),
            0,
            [1].to_vec(),
            [10 * DOLLARS].to_vec(),
            1000 * DOLLARS,
            2
        ));
        assert_eq!(Asset::balance_of(&2, 1, 1), 10 * DOLLARS);
    });
}

#[test]
fn buy_more_assets_than_available() {
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
            [2 * DOLLARS].to_vec(),
            [1 * DOLLARS].to_vec(),
        ));
        assert_err!(
            Dex::buy_assets(
                Origin::signed(2),
                0,
                [1].to_vec(),
                [3 * DOLLARS].to_vec(),
                1000 * DOLLARS,
                2
            ),
            Error::<Test>::InsufficientLiquidity
        );
    });
}

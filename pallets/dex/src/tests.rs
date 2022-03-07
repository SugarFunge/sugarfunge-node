use super::*;
use crate::mock::*;
use frame_support::{assert_err, assert_ok, bounded_vec};

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
        mock::Event::Currency(sugarfunge_currency::Event::Mint {
            currency_id: SUGAR,
            amount: 500 * DOLLARS,
            who: 1
        }),
    );
    assert_eq!(Asset::balance_of(&1, SUGAR.0, SUGAR.1), 500 * DOLLARS);
    assert_ok!(Asset::do_create_class(&1, &1, 1, bounded_vec![0]));
    assert_ok!(Asset::do_create_asset(&1, 1, 1, bounded_vec![0]));
    assert_ok!(Asset::do_mint(&1, &1, 1, 1, 50000 * DOLLARS));
    assert_eq!(Asset::balance_of(&1, 1, 1), 50000 * DOLLARS);
}

pub fn endow_user_2() {
    run_to_block(10);
    assert_eq!(Asset::balance_of(&2, SUGAR.0, SUGAR.1), 0 * DOLLARS);
    assert_eq!(Asset::balance_of(&2, 1, 1), 0 * DOLLARS);
    assert_ok!(Asset::do_transfer_from(
        &1,
        &1,
        &2,
        SUGAR.0,
        SUGAR.1,
        100 * DOLLARS
    ));
    assert_eq!(
        last_event(),
        mock::Event::Asset(sugarfunge_asset::Event::Transferred {
            who: 1,
            from: 1,
            to: 2,
            class_id: SUGAR.0,
            asset_id: SUGAR.1,
            amount: 100 * DOLLARS
        }),
    );
    assert_eq!(Asset::balance_of(&2, SUGAR.0, SUGAR.1), 100 * DOLLARS);
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
        assert_ok!(Asset::do_create_asset(
            &sugarfunge_currency::Pallet::<Test>::account_id(),
            ETH.0.into(),
            ETH.1.into(),
            bounded_vec![]
        ));
        assert_ok!(mock::Currency::mint(Origin::signed(1), ETH, 500 * DOLLARS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(sugarfunge_currency::Event::Mint {
                currency_id: ETH,
                amount: 500 * DOLLARS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, ETH.0, ETH.1), 500 * DOLLARS);

        assert_ok!(Asset::do_create_asset(
            &sugarfunge_currency::Pallet::<Test>::account_id(),
            BTC.0.into(),
            BTC.1.into(),
            bounded_vec![]
        ));
        assert_ok!(mock::Currency::mint(Origin::signed(1), BTC, 500 * DOLLARS));
        assert_eq!(
            last_event(),
            mock::Event::Currency(sugarfunge_currency::Event::Mint {
                currency_id: BTC,
                amount: 500 * DOLLARS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, BTC.0, BTC.1), 500 * DOLLARS);
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
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated {
                exchange_id: 0,
                who: 1
            }),
        );
    })
}

#[test]
fn add_liquidity_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated {
                exchange_id: 0,
                who: 1
            }),
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
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [100 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * DOLLARS);
        let exchange = Exchanges::<Test>::get(0).unwrap();
        let balances = Dex::get_asset_reserves(&exchange.vault, 1, [1].to_vec());
        assert_eq!(balances, [100 * DOLLARS].to_vec());
        let amount = 1 * DOLLARS;
        let price =
            Dex::get_sell_price(amount, balances[0].saturating_sub(amount), currency_reserve);
        assert_eq!(price, Ok(0_995049752487624381));
    });
}

#[test]
fn buy_price_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_ok!(Dex::add_liquidity(
            Origin::signed(1),
            0,
            1,
            [1].to_vec(),
            [100 * DOLLARS].to_vec(),
            [100 * DOLLARS].to_vec(),
        ));
        let currency_reserve = Dex::currency_reserves(0, 1);
        assert_eq!(currency_reserve, 100 * DOLLARS);
        let exchange = Exchanges::<Test>::get(0).unwrap();
        let balances = Dex::get_asset_reserves(&exchange.vault, 1, [1].to_vec());
        assert_eq!(balances, [100 * DOLLARS].to_vec());
        let amount = 1 * DOLLARS;
        let price = Dex::get_buy_price(amount, currency_reserve, balances[0]);
        assert_eq!(price, Ok(1_015176894573879499));
    });
}

#[test]
fn buy_assets_works() {
    new_test_ext().execute_with(|| {
        before_exchange();
        endow_user_2();
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated {
                exchange_id: 0,
                who: 1
            }),
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
        assert_ok!(Dex::create_exchange(Origin::signed(1), 0, SUGAR, 1, 9000));
        assert_eq!(
            last_event(),
            mock::Event::Dex(crate::Event::ExchangeCreated {
                exchange_id: 0,
                who: 1
            }),
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

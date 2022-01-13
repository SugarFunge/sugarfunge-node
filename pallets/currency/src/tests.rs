use super::{CurrencyAssets, CurrencyId};
use crate::mock::*;
use frame_support::assert_ok;

fn last_event() -> Event {
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
            Event::Currency(crate::Event::Mint {
                currency_id: ETH,
                amount: 500 * CENTS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, ETH.0, ETH.1), 500 * CENTS);
    })
}

#[test]
fn currency_btc_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_ok!(Currency::mint(Origin::signed(1), BTC, 500 * CENTS));
        assert_eq!(
            last_event(),
            Event::Currency(crate::Event::Mint {
                currency_id: BTC,
                amount: 500 * CENTS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, BTC.0, BTC.1), 500 * CENTS);
    })
}

#[test]
fn issue_and_mint_currency() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        let new_currency_id = CurrencyId(0, 1000);
        assert_eq!(
            Asset::balance_of(&1, new_currency_id.0, new_currency_id.1),
            0
        );
        let call = Box::new(Call::OrmlCurrencies(
            orml_currencies::module::Call::update_balance {
                who: 1,
                currency_id: new_currency_id,
                amount: 1000000 * DOLLARS as i128,
            },
        ));
        assert_eq!(Sudo::key(), 1u64);
        assert_ok!(Sudo::sudo(Origin::signed(1), call));
        assert_ok!(Currency::mint(
            Origin::signed(1),
            new_currency_id,
            500 * CENTS
        ));
        assert_eq!(
            last_event(),
            Event::Currency(crate::Event::Mint {
                currency_id: new_currency_id,
                amount: 500 * CENTS,
                who: 1
            }),
        );
        assert_eq!(
            Asset::balance_of(&1, new_currency_id.0, new_currency_id.1),
            500 * CENTS
        );
        let asset_info = CurrencyAssets::<Test>::get(new_currency_id).unwrap();
        assert_eq!(asset_info.total_supply, 500 * CENTS);
    })
}

#[test]
fn currency_mint_works() {
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(Asset::balance_of(&1, SUGAR.0, SUGAR.1), 0 * CENTS);
        assert_ok!(Currency::mint(Origin::signed(1), SUGAR, 500 * CENTS));
        assert_eq!(
            last_event(),
            Event::Currency(crate::Event::Mint {
                currency_id: SUGAR,
                amount: 500 * CENTS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, SUGAR.0, SUGAR.1), 500 * CENTS);
        let asset_info = CurrencyAssets::<Test>::get(SUGAR).unwrap();
        assert_eq!(asset_info.total_supply, 500 * CENTS);
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
            Event::Currency(crate::Event::Burn {
                currency_id: SUGAR,
                amount: 400 * CENTS,
                who: 1
            }),
        );
        assert_eq!(Asset::balance_of(&1, SUGAR.0, SUGAR.1), 100 * CENTS);
    })
}

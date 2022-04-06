use crate::{mock::*, AmountOp, AssetRate, Error, RateAccount, RateAction, RateBalance, Rates};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use sp_std::prelude::*;

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

fn simple_market_rates() -> Rates<Test> {
    vec![
        //
        // Buyer wants these goods
        //
        // Market will transfer 1 asset of class_id: 2000 asset_id: 1 to buyer
        AssetRate {
            class_id: 2000,
            asset_id: 1,
            action: RateAction::Transfer,
            amount: 1,
            from: RateAccount::Market,
            to: RateAccount::Buyer,
        },
        // Market will mint 100 assets of class_id: 2000 asset_id: 2 for buyer
        AssetRate {
            class_id: 2000,
            asset_id: 2,
            action: RateAction::Mint,
            amount: 1,
            from: RateAccount::Market,
            to: RateAccount::Buyer,
        },
        //
        // Market asking price
        //
        // Market requires buyer owns 5000 or more assets of type class_id: 3000 asset_id: 0
        AssetRate {
            class_id: 3000,
            asset_id: 1,
            action: RateAction::Has(AmountOp::GreaterEqualThan),
            amount: 5000,
            from: RateAccount::Buyer,
            to: RateAccount::Market,
        },
        // Buyer will transfer 5 assets of type class_id: 3000 asset_id: 2 to market
        AssetRate {
            class_id: 3000,
            asset_id: 2,
            action: RateAction::Transfer,
            amount: 5,
            from: RateAccount::Buyer,
            to: RateAccount::Market,
        },
        // Market will burn 50 assets of class_id: 3000 asset_id: 3 from buyer
        AssetRate {
            class_id: 3000,
            asset_id: 3,
            action: RateAction::Burn,
            amount: 50,
            from: RateAccount::Buyer,
            to: RateAccount::Market,
        },
        //
        // Royalties
        //
        // Market pays royalties to account 0
        AssetRate {
            class_id: 4000,
            asset_id: 1,
            action: RateAction::Transfer,
            amount: 2,
            from: RateAccount::Market,
            to: RateAccount::Account(0),
        },
        // Buyer pays royalties to account 0
        AssetRate {
            class_id: 4000,
            asset_id: 1,
            action: RateAction::Transfer,
            amount: 1,
            from: RateAccount::Buyer,
            to: RateAccount::Account(0),
        },
    ]
    .try_into()
    .unwrap()
}

pub fn before_market() {
    run_to_block(10);

    assert_ok!(Asset::do_create_class(&1, &1, 2000, bounded_vec![]));
    assert_ok!(Asset::do_create_class(&1, &1, 3000, bounded_vec![]));
    assert_ok!(Asset::do_create_class(&1, &1, 4000, bounded_vec![]));

    let asset_ids = [1, 2, 3, 4, 5].to_vec();
    let amounts = [100, 200, 300, 400, 500].to_vec();

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        2000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        3000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Asset::do_batch_mint(
        &1,
        &2,
        4000,
        asset_ids.clone(),
        amounts.clone(),
    ));

    assert_ok!(Market::do_create_market(&1, 1000));

    assert_eq!(
        last_event(),
        Event::Market(crate::Event::Created {
            market_id: 1000,
            who: 1,
        }),
    );
}

#[test]
fn before_market_works() {
    new_test_ext().execute_with(|| {
        before_market();
    })
}

#[test]
fn create_market_works() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&1, 1001));

        assert_eq!(
            last_event(),
            Event::Market(crate::Event::Created {
                market_id: 1001,
                who: 1,
            }),
        );
    })
}

#[test]
fn create_market_rate_works() {
    new_test_ext().execute_with(|| {
        before_market();

        let market_rate = simple_market_rates();

        assert_ok!(Market::do_create_market_rate(&1, 1000, 2, &market_rate));

        assert_eq!(
            last_event(),
            Event::Market(crate::Event::RateCreated {
                market_id: 1000,
                market_rate_id: 2,
                who: 1
            }),
        );
    })
}

#[test]
fn invalid_market_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_noop!(
            Market::do_quote_deposit(&1, 1001, 1, 2),
            Error::<Test>::InvalidMarket
        );
    })
}

#[test]
fn invalid_market_rate_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_noop!(
            Market::do_quote_deposit(&1, 1000, 1000, 2),
            Error::<Test>::InvalidMarketRate
        );
    })
}

#[test]
fn quote_deposit_fails() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&2, 2000));

        let rates = simple_market_rates();

        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));

        let result = Market::do_quote_deposit(&2, 2000, 100, 100);
        if let Ok((can_deposit, deposits)) = result {
            assert_eq!(can_deposit, false);
            assert_eq!(deposits.get(&rates[0]), Some(&100));
            assert_eq!(deposits.get(&rates[1]), None);
            assert_eq!(deposits.get(&rates[2]), None);
            assert_eq!(deposits.get(&rates[3]), None);
            assert_eq!(deposits.get(&rates[4]), None);
            assert_eq!(deposits.get(&rates[5]), Some(&-100));
            assert_eq!(deposits.get(&rates[6]), None);
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn quote_deposit_works() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&2, 2000));

        let rates = simple_market_rates();

        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));

        let result = Market::do_quote_deposit(&2, 2000, 100, 2);
        if let Ok((can_deposit, deposits)) = result {
            assert_eq!(can_deposit, true);
            assert_eq!(deposits.get(&rates[0]), Some(&2));
            assert_eq!(deposits.get(&rates[1]), None);
            assert_eq!(deposits.get(&rates[2]), None);
            assert_eq!(deposits.get(&rates[3]), None);
            assert_eq!(deposits.get(&rates[4]), None);
            assert_eq!(deposits.get(&rates[5]), Some(&4));
            assert_eq!(deposits.get(&rates[6]), None);
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn add_liquidity_works() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 4));

        if let Event::Market(crate::Event::LiquidityAdded {
            who,
            market_id,
            market_rate_id,
            amount,
            balances,
            success,
        }) = last_event()
        {
            assert_eq!(who, 2);
            assert_eq!(market_id, 2000);
            assert_eq!(market_rate_id, 100);
            assert_eq!(amount, 4);

            let get_balance = |rate_idx: usize| {
                balances.iter().find_map(|RateBalance { rate, balance }| {
                    if *rate == rates[rate_idx] {
                        Some(balance)
                    } else {
                        None
                    }
                })
            };

            assert_eq!(success, true);
            assert_eq!(get_balance(0), Some(&4));
            assert_eq!(get_balance(1), None);
            assert_eq!(get_balance(2), None);
            assert_eq!(get_balance(3), None);
            assert_eq!(get_balance(5), Some(&8));
            assert_eq!(get_balance(6), None);
        } else {
            unreachable!()
        }
    })
}

#[test]
fn add_liquidity_fails() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 100));

        if let Event::Market(crate::Event::LiquidityAdded {
            who,
            market_id,
            market_rate_id,
            amount,
            balances,
            success,
        }) = last_event()
        {
            assert_eq!(success, false);
            assert_eq!(who, 2);
            assert_eq!(market_id, 2000);
            assert_eq!(market_rate_id, 100);
            assert_eq!(amount, 100);

            let get_balance = |rate_idx: usize| {
                balances.iter().find_map(|RateBalance { rate, balance }| {
                    if *rate == rates[rate_idx] {
                        Some(balance)
                    } else {
                        None
                    }
                })
            };

            assert_eq!(get_balance(0), Some(&100));
            assert_eq!(get_balance(1), None);
            assert_eq!(get_balance(2), None);
            assert_eq!(get_balance(3), None);
            assert_eq!(get_balance(5), Some(&-100));
            assert_eq!(get_balance(6), None);
        } else {
            unreachable!()
        }
    })
}

#[test]
fn quote_exchange_insufficient() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 4));

        let result = Market::do_quote_exchange(&3, 2000, 100, 3);
        if let Ok((can_do_exchange, balances)) = result {
            assert_eq!(can_do_exchange, false);
            assert_eq!(balances.get(&rates[0]), Some(&3));
            assert_eq!(balances.get(&rates[1]), Some(&3));
            assert_eq!(balances.get(&rates[2]), Some(&-5000));
            assert_eq!(balances.get(&rates[3]), Some(&-15));
            assert_eq!(balances.get(&rates[4]), Some(&-150));
            assert_eq!(balances.get(&rates[5]), Some(&6));
            assert_eq!(balances.get(&rates[6]), Some(&-3));
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn quote_exchange_sufficient() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 4));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            2000,
            vec![1, 2,],
            vec![100, 100],
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            3000,
            vec![1, 2, 3],
            vec![10000, 50, 300],
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            4000,
            vec![1, 2],
            vec![100, 100],
        ));

        let result = Market::do_quote_exchange(&3, 2000, 100, 3);
        if let Ok((can_do_exchange, balances)) = result {
            assert_eq!(can_do_exchange, true);
            assert_eq!(balances.get(&rates[0]), Some(&3));
            assert_eq!(balances.get(&rates[1]), Some(&3));
            assert_eq!(balances.get(&rates[2]), Some(&5000));
            assert_eq!(balances.get(&rates[3]), Some(&15));
            assert_eq!(balances.get(&rates[4]), Some(&150));
            assert_eq!(balances.get(&rates[5]), Some(&6));
            assert_eq!(balances.get(&rates[6]), Some(&3));
        } else {
            result.unwrap();
        };
    })
}

#[test]
fn exchange_assets_works() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 4));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            2000,
            vec![1, 2,],
            vec![100, 100],
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            3000,
            vec![1, 2, 3],
            vec![10000, 50, 300],
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            4000,
            vec![1, 2],
            vec![100, 100],
        ));

        assert_ok!(Market::do_exchange_assets(&3, 2000, 100, 3));

        if let Event::Market(crate::Event::Exchanged {
            buyer,
            market_id,
            market_rate_id,
            amount,
            balances,
            success,
        }) = last_event()
        {
            let get_balance = |rate_idx: usize| {
                balances.iter().find_map(|RateBalance { rate, balance }| {
                    if *rate == rates[rate_idx] {
                        Some(balance)
                    } else {
                        None
                    }
                })
            };

            assert_eq!(success, true);
            assert_eq!(buyer, 3);
            assert_eq!(market_id, 2000);
            assert_eq!(market_rate_id, 100);
            assert_eq!(amount, 3);
            assert_eq!(get_balance(0), Some(&3));
            assert_eq!(get_balance(1), Some(&3));
            assert_eq!(get_balance(2), Some(&5000));
            assert_eq!(get_balance(3), Some(&15));
            assert_eq!(get_balance(4), Some(&150));
            assert_eq!(get_balance(5), Some(&6));
            assert_eq!(get_balance(6), Some(&3));
        } else {
            unreachable!()
        }
    })
}

#[test]
fn exchange_assets_fails() {
    new_test_ext().execute_with(|| {
        before_market();
        assert_ok!(Market::do_create_market(&2, 2000));
        let rates = simple_market_rates();
        assert_ok!(Market::do_create_market_rate(&2, 2000, 100, &rates));
        assert_ok!(Market::do_add_liquidity(&2, 2000, 100, 4));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            2000,
            vec![1, 2,],
            vec![100, 100],
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &3,
            3000,
            vec![1, 2, 3],
            vec![10000, 50, 300],
        ));

        assert_ok!(Market::do_exchange_assets(&3, 2000, 100, 3));

        if let Event::Market(crate::Event::Exchanged {
            buyer,
            market_id,
            market_rate_id,
            amount,
            balances,
            success,
        }) = last_event()
        {
            let get_balance = |rate_idx: usize| {
                balances.iter().find_map(|RateBalance { rate, balance }| {
                    if *rate == rates[rate_idx] {
                        Some(balance)
                    } else {
                        None
                    }
                })
            };

            assert_eq!(success, false);
            assert_eq!(buyer, 3);
            assert_eq!(market_id, 2000);
            assert_eq!(market_rate_id, 100);
            assert_eq!(amount, 3);
            assert_eq!(get_balance(0), Some(&3));
            assert_eq!(get_balance(1), Some(&3));
            assert_eq!(get_balance(2), Some(&5000));
            assert_eq!(get_balance(3), Some(&15));
            assert_eq!(get_balance(4), Some(&150));
            assert_eq!(get_balance(5), Some(&6));
            assert_eq!(get_balance(6), Some(&-3));
        } else {
            unreachable!()
        }
    })
}

use crate::mock::*;
use frame_support::{assert_ok, bounded_vec};

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_escrow() {
    run_to_block(10);
    assert_ok!(Currency::mint(Origin::signed(1), SUGAR, 500 * DOLLARS));
    assert_eq!(
        last_event(),
        Event::Currency(sugarfunge_currency::Event::Mint {
            currency_id: SUGAR,
            amount: 500 * DOLLARS,
            who: 1
        }),
    );
    assert_eq!(Asset::balance_of(&1, SUGAR.0, SUGAR.1), 500 * DOLLARS);
    assert_ok!(Asset::create_class(Origin::signed(1), 1, 1, bounded_vec![]));
    assert_ok!(Asset::create_asset(Origin::signed(1), 1, 1, bounded_vec![]));
    assert_ok!(Asset::do_mint(&1, &1, 1, 1, 50000 * DOLLARS));
    assert_eq!(Asset::balance_of(&1, 1, 1), 50000 * DOLLARS);

    assert_ok!(Escrow::do_register_escrow(&1, 1000, bounded_vec![]));
}

#[test]
fn deposit_assets() {
    new_test_ext().execute_with(|| {
        before_escrow();

        assert_ok!(Asset::create_class(Origin::signed(1), 1, 2, bounded_vec![]));
        assert_ok!(Asset::create_class(Origin::signed(1), 1, 3, bounded_vec![]));
        assert_ok!(Asset::create_class(Origin::signed(1), 1, 4, bounded_vec![]));

        let asset_ids = [0, 1, 2, 3, 4].to_vec();
        let amounts = [
            100 * DOLLARS,
            200 * DOLLARS,
            300 * DOLLARS,
            400 * DOLLARS,
            500 * DOLLARS,
        ]
        .to_vec();

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            2,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            3,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            4,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Escrow::create_account(
            Origin::signed(1),
            1000,
            vec![2],
            vec![1]
        ));
        if let Event::Escrow(crate::Event::AccountCreated {
            escrow,
            who,
            class_id,
            asset_id,
            owners,
        }) = last_event()
        {
            assert_eq!(who, 1);
            assert_eq!(class_id, 1000);
            assert_eq!(asset_id, 0);
            assert_eq!(owners, vec![2]);

            assert_ok!(Escrow::deposit_assets(
                Origin::signed(2),
                escrow,
                vec![2, 3, 4],
                vec![asset_ids.clone(), asset_ids.clone(), asset_ids.clone()],
                vec![amounts.clone(), amounts.clone(), amounts.clone()],
            ));

            let mut balances = Asset::balances_of_owner(&escrow).unwrap();
            balances.sort();

            let expected_balances = vec![
                (2, 0, 100 * DOLLARS),
                (2, 1, 200 * DOLLARS),
                (2, 2, 300 * DOLLARS),
                (2, 3, 400 * DOLLARS),
                (2, 4, 500 * DOLLARS),
                (3, 0, 100 * DOLLARS),
                (3, 1, 200 * DOLLARS),
                (3, 2, 300 * DOLLARS),
                (3, 3, 400 * DOLLARS),
                (3, 4, 500 * DOLLARS),
                (4, 0, 100 * DOLLARS),
                (4, 1, 200 * DOLLARS),
                (4, 2, 300 * DOLLARS),
                (4, 3, 400 * DOLLARS),
                (4, 4, 500 * DOLLARS),
            ];
            assert_eq!(balances, expected_balances);

            // let mut balances = Asset::balances_of_owner(&2).unwrap();
            // balances.sort();
            // let expected_balances = vec![
            //     (2, 0, 0),
            //     (2, 1, 0),
            //     (2, 2, 0),
            //     (2, 3, 0),
            //     (2, 4, 0),
            //     (3, 0, 0),
            //     (3, 1, 0),
            //     (3, 2, 0),
            //     (3, 3, 0),
            //     (3, 4, 0),
            //     (4, 0, 0),
            //     (4, 1, 0),
            //     (4, 2, 0),
            //     (4, 3, 0),
            //     (4, 4, 0),
            //     (1000, 0, 1), // Escrow shares
            // ];
            // assert_eq!(balances, expected_balances);

        } else {
            unreachable!()
        }
    })
}

#[test]
fn sweep_assets() {
    new_test_ext().execute_with(|| {
        before_escrow();

        assert_ok!(Asset::create_class(Origin::signed(1), 1, 2, bounded_vec![]));
        assert_ok!(Asset::create_class(Origin::signed(1), 1, 3, bounded_vec![]));
        assert_ok!(Asset::create_class(Origin::signed(1), 1, 4, bounded_vec![]));

        let asset_ids = [0, 1, 2, 3, 4].to_vec();
        let amounts = [
            100 * DOLLARS,
            200 * DOLLARS,
            300 * DOLLARS,
            400 * DOLLARS,
            500 * DOLLARS,
        ]
        .to_vec();

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            2,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            3,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Asset::do_batch_mint(
            &1,
            &2,
            4,
            asset_ids.clone(),
            amounts.clone(),
        ));

        assert_ok!(Escrow::create_account(
            Origin::signed(1),
            1000,
            vec![2],
            vec![1]
        ));
        if let Event::Escrow(crate::Event::AccountCreated {
            escrow,
            who,
            class_id,
            asset_id,
            owners,
        }) = last_event()
        {
            assert_eq!(who, 1);
            assert_eq!(class_id, 1000);
            assert_eq!(asset_id, 0);
            assert_eq!(owners, vec![2]);

            assert_ok!(Escrow::deposit_assets(
                Origin::signed(2),
                escrow,
                vec![2, 3, 4],
                vec![asset_ids.clone(), asset_ids.clone(), asset_ids.clone()],
                vec![amounts.clone(), amounts.clone(), amounts.clone()],
            ));

            let mut balances = Asset::balances_of_owner(&escrow).unwrap();
            balances.sort();
            let expected_balances = vec![
                (2, 0, 100 * DOLLARS),
                (2, 1, 200 * DOLLARS),
                (2, 2, 300 * DOLLARS),
                (2, 3, 400 * DOLLARS),
                (2, 4, 500 * DOLLARS),
                (3, 0, 100 * DOLLARS),
                (3, 1, 200 * DOLLARS),
                (3, 2, 300 * DOLLARS),
                (3, 3, 400 * DOLLARS),
                (3, 4, 500 * DOLLARS),
                (4, 0, 100 * DOLLARS),
                (4, 1, 200 * DOLLARS),
                (4, 2, 300 * DOLLARS),
                (4, 3, 400 * DOLLARS),
                (4, 4, 500 * DOLLARS),
            ];
            assert_eq!(balances, expected_balances);

            let mut balances = Asset::balances_of_owner(&2).unwrap();
            balances.sort();
            let expected_balances = vec![
                (2, 0, 0),
                (2, 1, 0),
                (2, 2, 0),
                (2, 3, 0),
                (2, 4, 0),
                (3, 0, 0),
                (3, 1, 0),
                (3, 2, 0),
                (3, 3, 0),
                (3, 4, 0),
                (4, 0, 0),
                (4, 1, 0),
                (4, 2, 0),
                (4, 3, 0),
                (4, 4, 0),
                (1000, 0, 1), // Escrow shares
            ];
            assert_eq!(balances, expected_balances);

            assert_ok!(Escrow::sweep_assets(Origin::signed(2), 2, escrow));
            let mut balances = Asset::balances_of_owner(&2).unwrap();
            balances.sort();
            let expected_balances = vec![
                (2, 0, 100 * DOLLARS),
                (2, 1, 200 * DOLLARS),
                (2, 2, 300 * DOLLARS),
                (2, 3, 400 * DOLLARS),
                (2, 4, 500 * DOLLARS),
                (3, 0, 100 * DOLLARS),
                (3, 1, 200 * DOLLARS),
                (3, 2, 300 * DOLLARS),
                (3, 3, 400 * DOLLARS),
                (3, 4, 500 * DOLLARS),
                (4, 0, 100 * DOLLARS),
                (4, 1, 200 * DOLLARS),
                (4, 2, 300 * DOLLARS),
                (4, 3, 400 * DOLLARS),
                (4, 4, 500 * DOLLARS),
                (1000, 0, 0), // Escrow shares
            ];
            assert_eq!(balances, expected_balances);
        } else {
            unreachable!()
        }
    })
}

#[test]
fn before_escrow_works() {
    new_test_ext().execute_with(|| {
        before_escrow();
    })
}

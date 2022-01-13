use crate::{mock::*, AssetRate, MarketRate, RateAmount, RateTarget};
use frame_support::assert_ok;

fn last_event() -> Event {
    frame_system::Pallet::<Test>::events()
        .pop()
        .expect("Event expected")
        .event
}

pub fn before_market() {
    run_to_block(10);

    assert_ok!(Asset::do_create_class(&1, &1, 2000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, &1, 3000, [0].to_vec()));
    assert_ok!(Asset::do_create_class(&1, &1, 4000, [0].to_vec()));

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

        assert_ok!(Market::do_create_market(&1, 1000));

        assert_eq!(
            last_event(),
            Event::Market(crate::Event::Created {
                market_id: 1000,
                who: 1,
            }),
        );
    })
}

#[test]
fn create_market_rate_works() {
    new_test_ext().execute_with(|| {
        before_market();

        assert_ok!(Market::do_create_market(&1, 1000));

        assert_eq!(
            last_event(),
            Event::Market(crate::Event::Created {
                market_id: 1000,
                who: 1,
            }),
        );

        let market_rate = MarketRate {
            goods: vec![
                AssetRate {
                    class_id: 2000,
                    asset_id: 1,
                    amount: RateAmount::Credit(5),
                    target: RateTarget::Buyer,
                },
                AssetRate {
                    class_id: 2000,
                    asset_id: 2,
                    amount: RateAmount::Credit(5),
                    target: RateTarget::Buyer,
                },
            ],
            price: vec![
                AssetRate {
                    class_id: 3000,
                    asset_id: 1,
                    amount: RateAmount::Debit(1),
                    target: RateTarget::Buyer,
                },
                AssetRate {
                    class_id: 3000,
                    asset_id: 2,
                    amount: RateAmount::Debit(1),
                    target: RateTarget::Buyer,
                },
            ],
            metadata: vec![],
        };

        assert_ok!(Market::do_create_market_rate(&1, 1000, 1, &market_rate));
    })
}

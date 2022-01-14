use crate::{mock::*, AmountOp, AssetRate, MarketRate, RateAmount, RateTarget};
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
            // Buyer wants these goods
            goods: vec![
                // Seller will transfer 1 asset of class_id: 2000 asset_id: 1 to buyer
                AssetRate {
                    class_id: 2000,
                    asset_id: 1,
                    amount: RateAmount::Credit(1),
                    target: RateTarget::Buyer,
                },
                // Market will mint 100 assets of class_id: 2000 asset_id: 2 for buyer
                AssetRate {
                    class_id: 2000,
                    asset_id: 2,
                    amount: RateAmount::Mint(100),
                    target: RateTarget::Buyer,
                },
            ],
            // Seller asking price
            price: vec![
                // Buyer has to own 5000 or more assets of type class_id: 0 asset_id: 0
                AssetRate {
                    class_id: 0,
                    asset_id: 0,
                    amount: RateAmount::Has(AmountOp::GreaterEqualThan(5000)),
                    target: RateTarget::Buyer,
                },
                // Buyer will transfer 5 assets of type class_id: 3000 asset_id: 2 to seller
                AssetRate {
                    class_id: 3000,
                    asset_id: 2,
                    amount: RateAmount::Debit(5),
                    target: RateTarget::Buyer,
                },
                // Market will burn 50 assets of class_id: 3000 asset_id: 3 from seller
                AssetRate {
                    class_id: 3000,
                    asset_id: 3,
                    amount: RateAmount::Burn(50),
                    target: RateTarget::Buyer,
                },
            ],
            metadata: vec![],
        };

        assert_ok!(Market::do_create_market_rate(&1, 1000, 1, &market_rate));
    })
}

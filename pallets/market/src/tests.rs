use crate::mock::*;
use frame_support::assert_ok;

// fn last_event() -> Event {
//     frame_system::Pallet::<Test>::events()
//         .pop()
//         .expect("Event expected")
//         .event
// }

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

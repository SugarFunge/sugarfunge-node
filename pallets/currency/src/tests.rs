use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn create_currency_works() {
    new_test_ext().execute_with(|| {
        assert_ok!(Currency::mint(Origin::signed(1), SUGAR, 500 * CENTS));
        assert_ok!(Token::create_instance(Origin::signed(1), [0].to_vec()));
        assert_ok!(Token::create_token(
            Origin::signed(1),
            1,
            1,
            false,
            [0].to_vec()
        ));
        assert_ok!(Token::mint(Origin::signed(1), 1, 1, 1, 50000 * CENTS));
    })
}

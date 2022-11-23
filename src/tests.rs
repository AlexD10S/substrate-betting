use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".to_owned();
		let team2 = "team2".to_owned();
		let start = 10;
		let length = 10;
		// Dispatch a signed extrinsic.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1, team2, start, length));
		// Read pallet storage and assert an expected result.
		// assert_eq!(Betting::get_value(), Some(42));
	});
}

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			Betting::cause_error(RuntimeOrigin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }

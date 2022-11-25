use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn creates_a_match() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".as_bytes().to_vec();
		let team2 = "team2".as_bytes().to_vec();
		let start = 10;
		let length = 10;
		// Dispatch a signed extrinsic.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1.clone(), team2.clone(), start, length));
		// Read pallet storage and assert an expected result.
		// println!("{:#?}",Betting::get_matches());
		assert_eq!(Betting::get_matches(1).unwrap().start, start);
		assert_eq!(Betting::get_matches(1).unwrap().length, length);
		assert_eq!(Betting::get_matches(1).unwrap().team1.to_owned(), team1);
		assert_eq!(Betting::get_matches(1).unwrap().team2.to_owned(), team2);
	});
}

#[test]
fn error_creating_a_match_with_an_open_match() {
	new_test_ext().execute_with(|| {
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), "team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10));
		// Ensure the expected error is thrown when the user tries to create a second match.
		assert_noop!(
			Betting::create_match_to_bet(RuntimeOrigin::signed(1), "team3".as_bytes().to_vec(), "team3".as_bytes().to_vec(), 20, 20),
			Error::<Test>::OriginHasAlreadyOpenMatch
		);
	});
}

#[test]
fn error_creating_a_match_that_has_finished() {
	new_test_ext().execute_with(|| {
		System::set_block_number(40);
		// Ensure the expected error is thrown when the user tries to create a match with a wrong time.
		assert_noop!(
			Betting::create_match_to_bet(RuntimeOrigin::signed(1), "team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10),
			Error::<Test>::TimeMatchOver
		);
	});
}

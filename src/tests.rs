use crate::{mock::*, Error};
use super::*;
use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::*;
use pallet_balances::Error as BalancesError;

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

#[test]
fn creates_a_bet() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".as_bytes().to_vec();
		let team2 = "team2".as_bytes().to_vec();
		let start = 10;
		let length = 10;
		// First we create a bet.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1.clone(), team2.clone(), start, length));
		// Create a bet from that match.
		let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
		assert_eq!(Balances::free_balance(Test::account_id()), 0);
		// println!("{:#?}",Balances::free_balance(Test::account_id());
		
		assert_ok!(Betting::bet(RuntimeOrigin::signed(2), match_id, 100, MatchResult::Team1Victory));
		assert_eq!(Balances::free_balance(Test::account_id()), 100);
		assert_eq!(Betting::get_matches(match_id).unwrap().bets.len(), 1);
	});
}

#[test]
fn error_betting_a_match_does_not_exist() {
	new_test_ext().execute_with(|| {
		let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
		// Ensure the expected error is thrown when the user tries to create a bet in a match doesn't exist.
		assert_noop!(
			Betting::bet(RuntimeOrigin::signed(2), match_id, 100, MatchResult::Team1Victory),
			Error::<Test>::MatchDoesNotExists
		);
	});
}

#[test]
fn error_betting_a_match_has_start() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".as_bytes().to_vec();
		let team2 = "team2".as_bytes().to_vec();
		let start = 10;
		let length = 10;
		// First we create a bet.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1.clone(), team2.clone(), start, length));
		System::set_block_number(12);
		let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
		// Ensure the expected error is thrown when the user tries to create a bet in a match that has started.
		assert_noop!(
			Betting::bet(RuntimeOrigin::signed(2), match_id, 100, MatchResult::Team1Victory),
			Error::<Test>::MatchHasStarted
		);
	});
}

#[test]
fn error_max_number_bets() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".as_bytes().to_vec();
		let team2 = "team2".as_bytes().to_vec();
		let start = 10;
		let length = 10;
		// First we create a bet.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1.clone(), team2.clone(), start, length));
		let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
		// Ensure the expected error is thrown when the user tries to bet in a match that has reach its limit.
		Betting::bet(RuntimeOrigin::signed(2), match_id, 100, MatchResult::Team1Victory);
		Betting::bet(RuntimeOrigin::signed(3), match_id, 100, MatchResult::Team2Victory);
		assert_noop!(
			Betting::bet(RuntimeOrigin::signed(4), match_id, 100, MatchResult::Draw),
			Error::<Test>::MaxBets
		);
	});
}

#[test]
fn error_no_funds_to_bet() {
	new_test_ext().execute_with(|| {
		let team1 = "team1".as_bytes().to_vec();
		let team2 = "team2".as_bytes().to_vec();
		let start = 10;
		let length = 10;
		// First we create a bet.
		assert_ok!(Betting::create_match_to_bet(RuntimeOrigin::signed(1), team1.clone(), team2.clone(), start, length));
		let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
		// Ensure the expected error is thrown when the user tries to bet without funds.
		assert_noop!(
			Betting::bet(RuntimeOrigin::signed(4), match_id, 100, MatchResult::Draw),
			BalancesError::<Test, _>::InsufficientBalance,
		);
		assert_eq!(Betting::get_matches(match_id).unwrap().bets.len(), 0);
	});
}

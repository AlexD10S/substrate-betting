use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_balances::Error as BalancesError;
use sp_runtime::traits::BadOrigin;

fn create_match(who: u64, t1: &str, t2: &str, start: u64, length: u64) -> AccountIdOf<Test> {
    // Dispatch a signed extrinsic.
    assert_ok!(Betting::create_match_to_bet(
        RuntimeOrigin::signed(who.clone()),
        t1.as_bytes().to_vec(),
        t2.as_bytes().to_vec(),
        start,
        length
    ));
    who.into()
}

#[test]
fn creates_a_match() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        // Read pallet storage and assert an expected result.
        let stored_bet = Betting::get_matches(match_id).unwrap();
        assert_eq!(stored_bet.start, 10);
        assert_eq!(stored_bet.length, 10);
        assert_eq!(stored_bet.team1.to_owned(), "team1".as_bytes().to_vec());
        assert_eq!(stored_bet.team2.to_owned(), "team2".as_bytes().to_vec());
        assert_eq!(stored_bet.result, None);
    });
}

#[test]
fn error_creating_same_match() {
    new_test_ext().execute_with(|| {
        let _ = create_match(1, "team1", "team2", 10, 10);
        // Do not allow other user to create a match with same specs of a previous one.
        assert_noop!(
            Betting::create_match_to_bet(
                RuntimeOrigin::signed(2),
                "team1".as_bytes().to_vec(),
                "team2".as_bytes().to_vec(),
                10,
                10
            ),
            Error::<Test>::MatchAlreadyExists
        );
    });
}

#[test]
fn error_creating_a_match_with_an_open_match() {
    new_test_ext().execute_with(|| {
        let _ = create_match(1, "team1", "team2", 10, 10);
        // Ensure the expected error is thrown when the user tries to create a second match.
        assert_noop!(
            Betting::create_match_to_bet(
                RuntimeOrigin::signed(1),
                "team3".as_bytes().to_vec(),
                "team3".as_bytes().to_vec(),
                20,
                20
            ),
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
            Betting::create_match_to_bet(
                RuntimeOrigin::signed(1),
                "team1".as_bytes().to_vec(),
                "team2".as_bytes().to_vec(),
                10,
                10
            ),
            Error::<Test>::TimeMatchOver
        );
    });
}

#[test]
fn creates_a_bet() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        assert_eq!(Balances::free_balance(Test::account_id()), 0);
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(2),
            match_id,
            100,
            MatchResult::Team1Victory
        ));
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
            Betting::bet(
                RuntimeOrigin::signed(2),
                match_id,
                100,
                MatchResult::Team1Victory
            ),
            Error::<Test>::MatchDoesNotExist
        );
    });
}

#[test]
fn error_betting_a_match_has_start() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        System::set_block_number(12);
        // Ensure the expected error is thrown when the user tries to create a bet in a match that has started.
        assert_noop!(
            Betting::bet(
                RuntimeOrigin::signed(2),
                match_id,
                100,
                MatchResult::Team1Victory
            ),
            Error::<Test>::MatchHasStarted
        );
    });
}

#[test]
fn error_max_number_bets() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        // Ensure the expected error is thrown when the user tries to bet in a match that has reach its limit.
        Betting::bet(
            RuntimeOrigin::signed(2),
            match_id,
            100,
            MatchResult::Team1Victory,
        )
        .ok();
        Betting::bet(
            RuntimeOrigin::signed(3),
            match_id,
            100,
            MatchResult::Team2Victory,
        )
        .ok();
        Betting::bet(RuntimeOrigin::signed(3), match_id, 50, MatchResult::Draw).ok();
        assert_noop!(
            Betting::bet(RuntimeOrigin::signed(4), match_id, 100, MatchResult::Draw),
            Error::<Test>::MaxBets
        );
    });
}

#[test]
fn error_no_funds_to_bet() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        // Ensure the expected error is thrown when the user tries to bet without funds.
        assert_noop!(
            Betting::bet(RuntimeOrigin::signed(5), match_id, 100, MatchResult::Draw),
            BalancesError::<Test, _>::InsufficientBalance,
        );
        assert_eq!(Betting::get_matches(match_id).unwrap().bets.len(), 0);
    });
}

#[test]
fn set_result_of_match() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        // Set the result of that match when it ends.
        System::set_block_number(22);
        assert_ok!(Betting::set_result(
            RawOrigin::Root.into(),
            match_id,
            MatchResult::Team1Victory
        ));
        assert_eq!(
            Betting::get_matches(match_id).unwrap().result,
            Some(MatchResult::Team1Victory)
        );
    });
}

#[test]
fn error_set_result_no_root() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        assert_noop!(
            Betting::set_result(
                RuntimeOrigin::signed(2),
                match_id,
                MatchResult::Team1Victory
            ),
            BadOrigin
        );
    });
}

#[test]
fn error_set_result_of_match_not_end() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);
        assert_noop!(
            Betting::set_result(RawOrigin::Root.into(), match_id, MatchResult::Team1Victory),
            Error::<Test>::TimeMatchNotOver
        );
    });
}

#[test]
fn distribute_winnings() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);

        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(2),
            match_id,
            10,
            MatchResult::Team1Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(3),
            match_id,
            10,
            MatchResult::Team2Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(4),
            match_id,
            30,
            MatchResult::Team1Victory
        ));

        // Set the result of that match when it ends.
        System::set_block_number(22);
        assert_ok!(Betting::set_result(
            RawOrigin::Root.into(),
            match_id,
            MatchResult::Team1Victory
        ));

        // The owner distributes the prizes
        assert_ok!(Betting::distribute_winnings(RuntimeOrigin::signed(1)));
        // Check that the prizes has been distributed properly.
        // With the maths there were 50 UNITS bet, 10 to the Team2 that lost and has to be shared by the rest

        //The first player deposit 10 and win back 12, has to have 2 UNITS more
        assert_eq!(Balances::free_balance(2), 1000000000000002);

        // The second player deposit 10 and lost, has to have 10 UNITS les
        assert_eq!(Balances::free_balance(3), 999999999999990);

        // The third player deposit 30 and win back 37, has to have 7 UNITS more
        assert_eq!(Balances::free_balance(4), 1000000000000007);

        // Check that the matches has been deleted after the distribution
        assert_eq!(Betting::get_matches(match_id), None);
    });
}

#[test]
fn error_distribute_winnings_no_match() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);

        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(2),
            match_id,
            10,
            MatchResult::Team1Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(3),
            match_id,
            10,
            MatchResult::Team2Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(4),
            match_id,
            30,
            MatchResult::Team1Victory
        ));
        // Set the result of that match when it ends.
        System::set_block_number(22);
        assert_ok!(Betting::set_result(
            RawOrigin::Root.into(),
            match_id,
            MatchResult::Team1Victory
        ));

        assert_noop!(
            Betting::distribute_winnings(RuntimeOrigin::signed(3)),
            Error::<Test>::MatchDoesNotExist
        );
    });
}

#[test]
fn error_distribute_winnings_no_result() {
    new_test_ext().execute_with(|| {
        let match_id = create_match(1, "team1", "team2", 10, 10);

        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(2),
            match_id,
            10,
            MatchResult::Team1Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(3),
            match_id,
            10,
            MatchResult::Team2Victory
        ));
        assert_ok!(Betting::bet(
            RuntimeOrigin::signed(4),
            match_id,
            30,
            MatchResult::Team1Victory
        ));

        // The owner tries to distributes the prizes
        assert_noop!(
            Betting::distribute_winnings(RuntimeOrigin::signed(1)),
            Error::<Test>::MatchNotResult
        );
    });
}

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::pallet_prelude::*;
use frame_system::RawOrigin;
use pallet_balances::Error as BalancesError;
use sp_runtime::traits::BadOrigin;

#[test]
fn creates_a_match() {
    new_test_ext().execute_with(|| {
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // Dispatch a signed extrinsic.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Read pallet storage and assert an expected result.
        // println!("{:#?}",Betting::get_matches());
        assert_eq!(Betting::get_matches(1).unwrap().start, start);
        assert_eq!(Betting::get_matches(1).unwrap().length, length);
        assert_eq!(Betting::get_matches(1).unwrap().team1.to_owned(), team1);
        assert_eq!(Betting::get_matches(1).unwrap().team2.to_owned(), team2);
    });
}

#[test]
fn error_creating_same_match() {
    new_test_ext().execute_with(|| {
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            "team1".as_bytes().to_vec(),
            "team2".as_bytes().to_vec(),
            10,
            10
        ));
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
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            "team1".as_bytes().to_vec(),
            "team2".as_bytes().to_vec(),
            10,
            10
        ));
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
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Create a bet from that match.
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
        assert_eq!(Balances::free_balance(Test::account_id()), 0);
        // println!("{:#?}",Balances::free_balance(Test::account_id());

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
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        System::set_block_number(12);
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Set the result of that match when it ends.
        System::set_block_number(22);
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();

        assert_ok!(Betting::set_result(
            RawOrigin::Root.into(),
            match_id,
            MatchResult::Team1Victory
        ));
        assert_eq!(
            Betting::get_results(match_id).unwrap(),
            MatchResult::Team1Victory
        );
    });
}

#[test]
fn error_set_result_no_root() {
    new_test_ext().execute_with(|| {
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Set the result of that match.
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();

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
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;
        // First we create a bet.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Set the result of that match
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();

        assert_noop!(
            Betting::set_result(RawOrigin::Root.into(), match_id, MatchResult::Team1Victory),
            Error::<Test>::TimeMatchNotOver
        );
    });
}

#[test]
fn distribute_winnings() {
    new_test_ext().execute_with(|| {
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;

        // First we create a match.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Create some bets from that match.
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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

        //println!("{:#?}",Balances::free_balance(ensure_signed(RuntimeOrigin::signed(2)).unwrap()));
        // Set the result of that match when it ends.
        System::set_block_number(22);
        assert_ok!(Betting::set_result(
            RawOrigin::Root.into(),
            match_id,
            MatchResult::Team1Victory
        ));

        //The owner distributes the prizes
        assert_ok!(Betting::distribute_winnings(RuntimeOrigin::signed(1)));
        //Check that the prizess has been distributed propertly.
        // With the maths there were 50 UNITS bet, 10 to the Team2 that lost and has to be shared by the rest

        //The first player deposit 10 and win back 12, has to have 2 UNITS more
        assert_eq!(
            Balances::free_balance(ensure_signed(RuntimeOrigin::signed(2)).unwrap()),
            1000000000000002
        );

        //The second player deposit 10 and wlost, has to have 10 UNITS les
        assert_eq!(
            Balances::free_balance(ensure_signed(RuntimeOrigin::signed(3)).unwrap()),
            999999999999990
        );

        //The third player deposit 30 and win back 37, has to have 7 UNITS more
        assert_eq!(
            Balances::free_balance(ensure_signed(RuntimeOrigin::signed(4)).unwrap()),
            1000000000000007
        );

        //Check that the matches has been deleted after the distribution
        assert_eq!(Betting::get_results(match_id), None);
        // let match_deleted = Betting::get_matches(match_id);
        // println!("{:#?}",match_deleted);
        // assert_eq!(
        // 	match_deleted,
        // 	None
        // );
    });
}

#[test]
fn error_distribute_winnings_no_match() {
    new_test_ext().execute_with(|| {
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;

        // First we create a match.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Create some bets from that match.
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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
            Error::<Test>::MatchDoesNotExists
        );
    });
}

#[test]
fn error_distribute_winnings_no_result() {
    new_test_ext().execute_with(|| {
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = 10;
        let length = 10;

        // First we create a match.
        assert_ok!(Betting::create_match_to_bet(
            RuntimeOrigin::signed(1),
            team1.clone(),
            team2.clone(),
            start,
            length
        ));
        // Create some bets from that match.
        let match_id = ensure_signed(RuntimeOrigin::signed(1)).unwrap();
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

        //The owner tries to distributes the prizes
        assert_noop!(
            Betting::distribute_winnings(RuntimeOrigin::signed(1)),
            Error::<Test>::MatchNotResult
        );
    });
}

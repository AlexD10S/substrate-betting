//! Benchmarking setup for pallet-betting

use super::*;

#[allow(unused)]
use crate::Pallet as Betting;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;


fn create_match<T: Config>(result: Option<MatchResult>) -> T::AccountId {
    let caller: T::AccountId = account("creator", 0, 0);
    T::Currency::make_free_balance_be(&caller, T::MatchDeposit::get() * T::Currency::minimum_balance() * 1000u32.into());

    let start = T::BlockNumber::from(5u32);
    let length = T::BlockNumber::from(5u32);


    let betting_match = Match {
        start,
        length,
        team1: <BoundedVec<_, T::MaxTeamNameLength>>::try_from("team1".as_bytes().to_vec())
            .unwrap(),
        team2: <BoundedVec<_, T::MaxTeamNameLength>>::try_from("team2".as_bytes().to_vec())
            .unwrap(),
        result,
        bets: Default::default(),
        deposit: T::MatchDeposit::get()
    };

    let match_hash = Betting::<T>::get_match_hash(betting_match.clone());

    <MatchHashes<T>>::insert(&match_hash, caller.clone());
    <Matches<T>>::insert(&caller, betting_match);

    caller
}

fn add_bet<T: Config>(user: &'static str, match_id: AccountIdOf<T>, a: u32, r: MatchResult) {
    let caller = account(user, 0, 0);
    T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() * 10u32.into());
    let origin = <T::RuntimeOrigin>::from(RawOrigin::Signed(caller));
    let _ = Betting::<T>::bet(
        origin,
        match_id,
        T::Currency::minimum_balance() * a.into(),
        r,
    );
}

benchmarks! {
    create_match_to_bet {
        // setup initial state
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, T::MatchDeposit::get() * T::Currency::minimum_balance() * 10u32.into());
        let team1 = "team1".as_bytes().to_vec();
        let team2 = "team2".as_bytes().to_vec();
        let start = T::BlockNumber::from(10u32);
        let length = T::BlockNumber::from(10u32);
    }: _(RawOrigin::Signed(caller.clone()), team1, team2, start, length) //execute extrinsic or function
    verify {
        assert!(Matches::<T>::contains_key(&caller)); //verify final state
    }

    bet {
        let match_id = create_match::<T>(None);
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, T::Currency::minimum_balance() * 10u32.into());
        let amount = BalanceOf::<T>::from(T::Currency::minimum_balance());
        let result = MatchResult::Draw;
    }: _(RawOrigin::Signed(caller.clone()), match_id.clone(), amount, result)
    verify {
        let m = Matches::<T>::get(&match_id).unwrap();
        assert_eq!(m.bets.len(), 1);
    }

    set_result {
        let match_id = create_match::<T>(None);
        frame_system::Pallet::<T>::set_block_number(15u32.into());
        let result = MatchResult::Team1Victory;
    }: _(RawOrigin::Root, match_id.clone(), result)
    verify {
        let m = Matches::<T>::get(&match_id).unwrap();
        assert_eq!(m.result, Some(MatchResult::Team1Victory));
    }

    distribute_winnings {
        let match_id = create_match::<T>(Some(MatchResult::Team1Victory));
        frame_system::Pallet::<T>::set_block_number(15u32.into());
        let result = MatchResult::Team1Victory;
        add_bet::<T>("user1", match_id.clone(), 1, MatchResult::Team1Victory);
        add_bet::<T>("user2", match_id.clone(), 2, MatchResult::Team2Victory);
        add_bet::<T>("user3", match_id.clone(), 3, MatchResult::Draw);
        add_bet::<T>("user4", match_id.clone(), 4, MatchResult::Draw);
        add_bet::<T>("user5", match_id.clone(), 5, MatchResult::Team1Victory);
    }: _(RawOrigin::Signed(match_id.clone()))
    verify {
        assert_eq!(Matches::<T>::contains_key(&match_id), false);
    }

    impl_benchmark_test_suite!(Betting, crate::mock::new_test_ext(), crate::mock::Test);
}

#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


pub use pallet::*;
use codec::{Decode, Encode, MaxEncodedLen, HasCompact};
use scale_info::TypeInfo;
use frame_support::{
	traits::{Get, Currency, ExistenceRequirement::KeepAlive},
	RuntimeDebug, BoundedVec,
};
use sp_std::{
	cmp::Ordering,
	prelude::*,
};

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum MatchResult {
	Team1Victory,
	Team2Victory,
	Draw,
}
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, scale_info::TypeInfo)]
/// A bet.
pub struct Bet<AccountId, MatchResult, Balance> {
	/// Account of the better.
	bettor: AccountId,
	/// Amount bet.
	amount: Balance,
	/// Result he picked
	result: MatchResult,
}
impl<AccountId, Balance> Ord for Bet<AccountId, MatchResult, Balance>
where
	AccountId: Ord,
	Balance: Ord + HasCompact,
{
	fn cmp(&self, other: &Self) -> Ordering {
		self.bettor
			.cmp(&other.bettor)
			.then_with(|| self.amount.cmp(&other.amount))
	}
}
impl<AccountId, Balance> PartialOrd for Bet<AccountId, MatchResult, Balance>
where
	AccountId: Ord,
	Balance: Ord + HasCompact,
{
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(MaxBetsPerMatch))]
pub struct Match<BlockNumber, Vec, Bet, MaxBetsPerMatch>
where
	MaxBetsPerMatch: Get<u32>,
{
	/// Starting block of the match.
	start: BlockNumber,
	/// Length of the match (start + length = end).,s
	length: BlockNumber,
	/// Team1 name
	team1: Vec,
	/// Team2 name
	team2: Vec,
	/// Array with the bets
	bets: BoundedVec<Bet, MaxBetsPerMatch>,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{
            traits::{
                AccountIdConversion
            },
        },
		traits::{ReservableCurrency},
		PalletId,
	};
	use frame_system::pallet_prelude::*;

	/// TODO: #[pallet::without_storage_info] line added after error:
    ///the trait `MaxEncodedLen` is not implemented for `Vec<u8>`
	#[pallet::pallet]
	#[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The Betting's pallet id
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The currency trait.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Number of the max amount of bets a match can have
		#[pallet::constant]
		type MaxBetsPerMatch: Get<u32>;

	}
	pub trait ConfigHelper: Config {
		fn account_id() -> AccountIdOf<Self>;
	}
	
	impl<T: Config> ConfigHelper for T {
		/// The account ID of the betting pot.
		///
		/// This actually does computation. If you need to keep using it, then make sure you cache the
		/// value and only call this once.
		#[inline(always)]
		fn account_id() -> AccountIdOf<Self> {
			Self::PalletId::get().into_account_truncating()
		}
	}

	// The set of open matches.
	#[pallet::storage]
	#[pallet::getter(fn get_matches)]
	pub type Matches<T: Config> = StorageMap<
		_, Twox64Concat, T::AccountId, Match<T::BlockNumber, Vec<u8>, Bet<T::AccountId, MatchResult, BalanceOf<T>,>, T::MaxBetsPerMatch>, 
	>;

	// The set of open matches.
	#[pallet::storage]
	#[pallet::getter(fn get_results)]
	pub type MatchResults<T: Config> = StorageMap<
		_, Twox64Concat, T::AccountId, MatchResult>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new match has been created. [who, team1, team2, start, length]
		MatchCreated(T::AccountId, Vec<u8>, Vec<u8>, T::BlockNumber, T::BlockNumber),
		/// A new bet has been created. [matchId, who, amount, result]
		BetPlaced(T::AccountId, T::AccountId, BalanceOf<T>, MatchResult),
		// A match result has been set
		MatchResult(T::AccountId, MatchResult),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The match to be created already exist.
		MatchAlreadyExists,
		/// Each account can only have one match open.
		OriginHasAlreadyOpenMatch,
		/// The time of the match is over.
		TimeMatchOver,
		/// The match where the bet is placed does not exist
		MatchDoesNotExists,
		/// No allowing betting if the match has started
		MatchHasStarted,
		/// The match has reach its betting limit
		MaxBets,
		/// You already place the same bet in that match
		AlreadyBet,
		/// No allowing set the result if the match not over
		TimeMatchNotOver,
		// The match still has not a result set
		MatchNotResult,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new match to bet on.
        /// Emit an event on success: `MatchCreated`.
        ///
        /// **Parameters:**
        ///   * `origin` – Origin for the call. Must be signed.
        ///   * `team1` – name of the first team.
        ///   * `team2` – name of the second team.
        ///   * `start` – time when the match starts and a bet can not be placed (in blocks).
        ///   * `lenght` – Duration of the match (in blocks).
        ///
        /// **Errors:**
        ///   * `MatchAlreadyExists` – A match for the specified values already exists.
		///   * `OriginHasAlreadyOpenMatch` – An origin can only have one match open.
		///   * `TimeMatchOver` – The match is created when the match time is over.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_match_to_bet(
			origin: OriginFor<T>, 
			team1: Vec<u8>,
			team2: Vec<u8>,
			start: T::BlockNumber,
			length: T::BlockNumber,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			// Check account has not an open match
			ensure!(!<Matches<T>>::contains_key(&who), Error::<T>::OriginHasAlreadyOpenMatch);
			// Check time start and time length are valid
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block_number < (start + length), Error::<T>::TimeMatchOver);

			//TODO: Check if match: team1 vs team2 exists?
			//TODO: Deposit to create match?

			// Initialize the bets bounded_vec
			let bets: BoundedVec<Bet<T::AccountId, MatchResult, BalanceOf<T>,>, T::MaxBetsPerMatch> = Default::default();
			// Create the betting match
			let betting_match = Match {
				start,
				length,
				team1: team1.clone(),
				team2: team2.clone(),
				bets
			};
			// Store the betting match in the list of open matches
			<Matches<T>>::insert(&who, betting_match);

			// Emit an event.
			Self::deposit_event(Event::MatchCreated(who, team1, team2, start, length));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Create bet for a match.
        /// Emit an event on success: `BetPlaced`.
        ///
        /// **Parameters:**
        ///   * `origin` – Origin for the call. Must be signed.
        ///   * `match_id` – Id of the match, in our case the creator of the bet accountId .
        ///   * `amount_to_bet` – Amount placed for the bet.
        ///   * `result` – the result for the bet.
        ///
        /// **Errors:**
        ///   * `MatchDoesNotExists` – A match selected for the bet doesn't exist.
		///   * `MatchHasStarted` – If the match has started, betting is not allowed.
		///   * `TimeMatchOver` – The match is created when the match time is over.
		///   * `MaxBets`   - The match has reach its betting limit
		///   * `AlreadyBet`   - You already place the same bet in that match
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn bet(
			origin: OriginFor<T>, 
			match_id: T::AccountId,
			amount_to_bet: BalanceOf<T>,
			result: MatchResult
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			//Find the match where user wants to place the bet
			let mut match_to_bet = <Matches<T>>::get(&match_id).ok_or(Error::<T>::MatchDoesNotExists)?;
			
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block_number < match_to_bet.start, Error::<T>::MatchHasStarted);

			//Create the bet to be placed
			let bet = Bet {
				bettor: who.clone(),
				amount: amount_to_bet.clone(),
				result
			};
			// let new_match_to_bet = &match_to_bet;
			match match_to_bet.bets.binary_search(&bet) {
				Ok(_pos) => return Err(Error::<T>::AlreadyBet.into()),
				Err(pos) => match_to_bet
					.bets
					.try_insert(pos, bet.clone())
					.map_err(|_| Error::<T>::MaxBets)?,
			}
			// Store the betting match in the list of open matches
			<Matches<T>>::insert(&match_id, match_to_bet);
			// Check user has enough funds and send it to the betting pallet account.
			T::Currency::transfer(&who, &T::account_id(), amount_to_bet, KeepAlive)?;

			// Emit an event.
			Self::deposit_event(Event::BetPlaced(
				match_id,
				who, 
				amount_to_bet, 
				result
			));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// When a match ends someone the owner of the match can distribute the money from the winers and delete the match .
        /// Emit an event on success: `WinningsDistributed`.
        ///
        /// **Parameters:**
        ///   * `origin` – Origin for the call. Must be signed.
        ///
        /// **Errors:**
        ///   * `MatchDoesNotExists` – A match selected for the bet doesn't exist.
		///   * `MatchNotResult` – The match still has not a result.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn distribute_winnings(
			origin: OriginFor<T>, 
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;
			//Find the match where user wants to place the bet an delete it
			let mut match_to_bet = <Matches<T>>::take(&who).ok_or(Error::<T>::MatchDoesNotExists)?;
			//Find the result of the match the bet an delete it
			let result_match_to_bet = <MatchResults<T>>::take(&who).ok_or(Error::<T>::MatchNotResult)?;
			//Check the bets from the array
			let mut total_winners: BalanceOf<T> = 0u32.into();
			let mut total_bet: BalanceOf<T> = 0u32.into();
			let mut winners = Vec::new();
			for bet in match_to_bet.bets.iter_mut() {
				total_bet +=  bet.amount;
				if bet.result == result_match_to_bet {
					total_winners +=  bet.amount;
					winners.push(bet)
				}
			}
			//Share the money
			for bet in winners.iter_mut() {
				let weighted = bet.amount / total_winners;
				let amount_won = weighted * total_bet;
				T::Currency::transfer(&T::account_id(), &bet.bettor, amount_won, KeepAlive)?;
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	

		/// Notify the result of an existing match.
		/// The dispatch origin for this call must be _Root_.
		/// 
        /// Emit an event on success: `MatchResult`.
        ///
        /// **Parameters:**
        ///   * `origin` – Origin for the call. Must be signed.
        ///   * `match_id` – Id of the match, in our case the creator of the bet accountId .
        ///   * `result` – the result for the match.
        ///
        /// **Errors:**
        ///   * `MatchDoesNotExists` – A match selected for the bet doesn't exist.
		///   * `TimeMatchNotOver` – If the match is not over, set the result is not allowed.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn set_result(
			origin: OriginFor<T>, 
			match_id: T::AccountId,
			match_result: MatchResult
		) -> DispatchResult {
			// Only root can call this extrinic.
			ensure_root(origin)?;
			//Find the match where user wants to place the bet
			let match_to_set_result = <Matches<T>>::get(&match_id).ok_or(Error::<T>::MatchDoesNotExists)?;
			
			// Check time start and time length are valid
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block_number > (match_to_set_result.start + match_to_set_result.length), Error::<T>::TimeMatchNotOver);

			// Store the result of the match
			<MatchResults<T>>::insert(&match_id, match_result);

			// Emit an event.
			Self::deposit_event(Event::MatchResult(
				match_id,
				match_result, 
			));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

}

	





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
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use frame_support::{
	traits::{Get, Currency},
	RuntimeDebug, BoundedVec,
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

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
/// A bet.
pub struct Bet<AccountId, MatchResult, Balance> {
	/// Account of the better.
	bettor: AccountId,
	/// Amount bet.
	amount: Balance,
	/// Result he picked
	result: MatchResult,
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
		/// The Lottery's pallet id
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
	#[pallet::getter(fn get_value)]
	pub type Matchs<T: Config> = StorageMap<
		_, Twox64Concat, T::AccountId, Match<T::BlockNumber, Vec<u8>, Bet<T::AccountId, MatchResult, BalanceOf<T>,>, T::MaxBetsPerMatch>, 
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new match has been created. [who, team1, team2, start, length]
		MatchCreated(T::AccountId, String, String, T::BlockNumber, T::BlockNumber,),
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
			team1: String,
			team2: String,
			start: T::BlockNumber,
			length: T::BlockNumber,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
			// Check account has not an open match
			ensure!(!<Matchs<T>>::contains_key(&who), Error::<T>::OriginHasAlreadyOpenMatch);
			// Check time start and time length are valid
			let current_block_number = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block_number < (start + length), Error::<T>::TimeMatchOver);

			//TODO: Check if match: team1 vs team2 exists?

			// Initialize the bets bounded_vec
			let bets: BoundedVec<Bet<T::AccountId, MatchResult, BalanceOf<T>,>, T::MaxBetsPerMatch> = Default::default();
			//Store the strings as Vec<8>
			let team1_bytes: Vec<u8> = team1.as_bytes().to_vec();
			let team2_bytes: Vec<u8> = team2.as_bytes().to_vec();
			// Create the betting match
			let betting_match = Match {
				start,
				length,
				team1: team1_bytes,
				team2: team2_bytes,
				bets
			};
			// Store the betting match in the list of open matches
			<Matchs<T>>::insert(&who, betting_match);

			// Emit an event.
			Self::deposit_event(Event::MatchCreated(who, team1, team2, start, length));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		// #[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;

		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => return Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
	}

}



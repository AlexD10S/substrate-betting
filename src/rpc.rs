use crate::{TeamName, Bets, Config, Error, Match, Pallet};
use codec::{Decode, Encode};
use scale_info::prelude::format;
use sp_std::fmt::Debug;
use sp_std::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub enum RpcError {
    MatchDoesNotExist,
    Unexpected(Vec<u8>),
}

pub type RpcResult<T> = Result<T, RpcError>;

impl<T: Config> From<Error<T>> for RpcError {
    fn from(err: Error<T>) -> Self {
        match err {
            Error::MatchDoesNotExist => Self::MatchDoesNotExist,
            err => Self::Unexpected(format!("{:?}", err).into_bytes()),
        }
    }
}

impl<T: Config> Pallet<T> {
    pub fn get_match(match_id: T::AccountId) -> RpcResult<Match<T::BlockNumber, TeamName<T>, Bets<T>>> {
        match Self::get_matches(match_id) {
            Some(m) => Ok(m),
            None => Err(RpcError::MatchDoesNotExist)
        }
    }
}
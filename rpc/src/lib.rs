pub use pallet_betting_rpc_runtime_api::BettingApi as BettingRuntimeApi;
use codec::Codec;
use jsonrpsee::{
	core::{Error as RpcError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::MaybeDisplay;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Custom {
	code: u32,
	sum: u32,
}
#[cfg(test)]
mod tests;

#[rpc(client, server)]
pub trait BettingApi<BlockHash, AccountId, Match> {
	#[method(name = "betting_getMatch")]
	fn get_match(&self, match_id: AccountId, at: Option<BlockHash>) -> RpcResult<Match>;
}

/// A struct that implements the `BettingApi`.
pub struct BettingPallet<C, Block> {
	// If you have more generics, no need to BettingPallet<C, M, N, P, ...>
	// just use a tuple like BettingPallet<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> BettingPallet<C, Block> {
	/// Create new `BettingPallet` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block, AccountId, Match> BettingApiServer<<Block as BlockT>::Hash, AccountId, Match> for BettingPallet<C, Block>
where
	Block: sp_runtime::traits::Block,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: BettingRuntimeApi<Block, AccountId, Match>,
	AccountId: Codec + MaybeDisplay + Copy + Send + Sync + 'static,
    Match: Codec + MaybeDisplay + Copy + Send + Sync + 'static,
{
	fn get_match(&self, match_id: AccountId, at: Option<Block::Hash>) -> RpcResult<Match> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||self.client.info().best_hash));

		api.get_match(&at, match_id).map_err(runtime_error_into_rpc_err)
	}
}

const RUNTIME_ERROR: i32 = 1;


/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
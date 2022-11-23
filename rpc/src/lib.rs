pub use pallet_betting_rpc_runtime_api::BettingApi as BettingRuntimeApi;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
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
pub trait BettingApi<BlockHash> {
	#[method(name = "betting_getMatches")]
	fn get_matches(&self, at: Option<BlockHash>) -> RpcResult<Custom>;
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

impl<C, Block> BettingApiServer<<Block as BlockT>::Hash> for BettingPallet<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: BettingRuntimeApi<Block>,
{
	fn get_matches(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Custom> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||self.client.info().best_hash));

		let value = api.get_matches(&at).map_err(runtime_error_into_rpc_err);
		Ok(Custom{ code: 200, sum: value.unwrap()})
	}
}

const RUNTIME_ERROR: i32 = 1;


/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		RUNTIME_ERROR,
		"Runtime error",
		Some(format!("{:?}", err)),
	))
	.into()
}
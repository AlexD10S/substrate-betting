#![cfg_attr(not(feature = "std"), no_std)]
use codec::Codec;
use sp_runtime::traits::MaybeDisplay;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait BettingApi <Matches> where
		Matches: Codec + MaybeDisplay,
	{
		fn get_matches() -> Matches;
	}
}

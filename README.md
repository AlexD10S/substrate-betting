# Betting pallet

## Overview

This pallet implements a basic protocol for decentralized betting. 

:warning: It is **not a production-ready paller**, but a sample built for learning purposes. It is discouraged to use this code 'as-is' in a production runtime.

## Main concepts

* **Currency** – The chain's main currency/token (e.g. DOT for the relay chain, ACA for Acala).


## Configuration

### Types
* `RuntimeEvent` – The overarching event type.
* `Currency` – The currency type.

### Constants
* `MinDeposit` – Minimum amount of currency which must be deposited when creating a new bet.

## Extrinsics

<details>
<summary><h3>do_something</h3></summary>

Takes a singles value as a parameter, writes the value to storage and emits an event. This function must be dispatched by a signed extrinsic.
Emit an event on success: `SomethingStored`.

#### Parameters:
  * `origin` – Origin for the call. Must be signed.
  * `something` – value to store.

#### Errors:
  * `NoneValue` – Error names should be descriptive.
</details>

## RPC

<details>
<summary><h3>template_getValue</h3></summary>

Get a value stored

#### Parameters:
</details>

## How to add `pallet-betting` to a node

:information_source: The pallet is compatible with Substrate version
[polkadot-v0.9.31](https://github.com/paritytech/substrate/tree/polkadot-v0.9.10).

:information_source: This section is based on
[Substrate node template](https://github.com/substrate-developer-hub/substrate-node-template/tree/polkadot-v0.9.31).
Integrating `pallet-betting` with another node might look slightly different.

### Runtime's `Cargo.toml`

Add `pallet-betting`, and the RPC runtime API, to dependencies.
```toml

[dependencies.pallet-betting]
version = "0.0.1"
default-features = false
git = "https://github.com/AlexD10S/substrate-betting.git"
branch = "main"

[dependencies.pallet-betting-rpc-runtime-api]
version = "0.0.1"
default-features = false
git = "https://github.com/AlexD10S/substrate-betting.git"
branch = "main"
```

Update the runtime's `std` feature:
```toml
std = [
    # --snip--
    "pallet-betting/std",
    "pallet-betting-rpc-runtime-api/std",
    # --snip--
]
```

### Node's `Cargo.toml`

Add `pallet-betting-rpc` to dependencies.
```toml

[dependencies.pallet-betting-rpc]
version = "0.0.1"
default-features = false
git = "https://github.com/AlexD10S/substrate-betting.git"
branch = "main"
```

### Runtime's `lib.rs`


Configure the betting pallet.
```rust

parameter_types! {
    pub const Deposit: ConstU128 = ConstU128<1>;
}

impl pallet_betting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinDeposit = Deposit;
}
```

Add configured pallets to the `construct_runtime` macro call.
```rust
construct_runtime!(
    pub enum Runtime where
        // --snip--
    {
        // --snip---
        Betting: pallet_betting,
        // --snip---
    }
);
```

Add the RPC implementation.
```rust
impl_runtime_apis! {
    // --snip--
    impl pallet_betting_rpc_runtime_api::BettingApi<Block> for Runtime {
      fn get_value() -> u32 {
        Betting::get_value().unwrap_or(0)
      }
    }
}
```


### Node's `rpc.rs`

Instantiate the RPC extension and merge it into the RPC module.
```rust
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    // --snip--
    C::Api: pallet_betting_rpc::BettingRuntimeApi<Block, Balance>,
{
    use pallet_betting_rpc::{Betting, BettingApiServer};
    // --snip--
    module.merge(Betting::new(client).into_rpc())?;
    Ok(module)
}
```

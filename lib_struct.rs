// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);

  #[pallet::config]  // <-- Step 2. code block will replace this.
  #[pallet::event]   // <-- Step 3. code block will replace this.
  #[pallet::error]   // <-- Step 4. code block will replace this.
  #[pallet::storage] // <-- Step 5. code block will replace this.
  #[pallet::call]    // <-- Step 6. code block will replace this.
}
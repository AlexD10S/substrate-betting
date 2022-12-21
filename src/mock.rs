use crate as pallet_betting;
use frame_support::traits::{ConstU128, ConstU16, ConstU32, ConstU64};
use frame_support::{parameter_types, PalletId};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Betting: pallet_betting,
        Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
    }

);

impl pallet_balances::Config for Test {
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU128<2>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
}

impl system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

parameter_types! {
    pub const MatchDeposit: u64 = 10;
    pub const BettingPalletId: PalletId = PalletId(*b"bet_mock");
}

impl pallet_betting::Config for Test {
    type PalletId = BettingPalletId;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MaxTeamNameLength = ConstU32<64>;
    type MaxBetsPerMatch = ConstU32<3>;
    type MatchDeposit = MatchDeposit;
    type WeightInfo = ();
}

pub(crate) const ACCOUNT_A: u64 = 0;
pub(crate) const ACCOUNT_B: u64 = 1;
pub(crate) const ACCOUNT_C: u64 = 2;
pub(crate) const ACCOUNT_D: u64 = 3;
pub(crate) const ACCOUNT_E: u64 = 4;
pub(crate) const INIT_BALANCE: u128 = 1_000_000_000_000_000;
// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (ACCOUNT_A, INIT_BALANCE),
            (ACCOUNT_B, INIT_BALANCE),
            (ACCOUNT_C, INIT_BALANCE),
            (ACCOUNT_D, INIT_BALANCE),
            (ACCOUNT_E, INIT_BALANCE),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut test_ext: sp_io::TestExternalities = storage.into();
    test_ext.execute_with(|| System::set_block_number(1));
    test_ext
}

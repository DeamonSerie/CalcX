use crate::*;
use crate as pallet;
use frame_support::construct_runtime;
use frame_support::{derive_impl, parameter_types};
use sp_runtime::{BuildStorage, FixedU128};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        Calcx: pallet,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<Balance>;
}

pub type Balance = u128;

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MinWindow: FixedU128 = FixedU128::from_rational(1001u128, 1000u128);
    pub const TreasuryAccount: u64 = 999;
    pub const PriceToCxScale: u128 = 1000;
}

impl pallet_balances::Config for Test {
    type MaxLocks = MaxLocks;
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u32;
    type MinWindow = MinWindow;
    type Currency = Balances;
    type TreasuryAccount = TreasuryAccount;
    type PriceToCxScale = PriceToCxScale;
    type PriceProvider = StoragePriceProvider<Self>;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

use crate::*;
use crate as pallet;
use frame_support::construct_runtime;
use frame_support::{
    derive_impl, parameter_types,
};
use sp_runtime::{BuildStorage, FixedU128};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Calcx: pallet,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = ();
}

parameter_types! {
    pub const MinWindow: FixedU128 = FixedU128::from_rational(1001u128, 1000u128);
}

impl Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u32;
    type MinWindow = MinWindow;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

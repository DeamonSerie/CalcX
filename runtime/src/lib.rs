#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
use sp_std::prelude::*;
use frame_support::{
    construct_runtime, derive_impl, parameter_types,
    traits::{ConstU128, ConstU32, ConstU64, ConstU8},
};
use frame_system as system;

pub type BlockNumber = u32;
pub type Signature = sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type Nonce = u32;
pub type Hash = sp_core::H256;

pub struct TreasuryPot;
impl frame_support::traits::Get<AccountId> for TreasuryPot {
    fn get() -> AccountId {
        pallet_treasury::Pallet::<Runtime>::account_id()
    }
}
impl sp_core::TypedGet for TreasuryPot {
    type Type = AccountId;
    fn get() -> Self::Type {
        pallet_treasury::Pallet::<Runtime>::account_id()
    }
}
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: sp_consensus_aura::sr25519::AuthorityId,
        pub grandpa: sp_consensus_grandpa::AuthorityId,
    }
}



construct_runtime!(
    pub enum Runtime
    {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Aura: pallet_aura,
        Grandpa: pallet_grandpa,
        Balances: pallet_balances,
        Treasury: pallet_treasury,
        Assets: pallet_assets,
        Sudo: pallet_sudo,
        TransactionPayment: pallet_transaction_payment,
        Calcx: pallet_calcx,
        TimeN: pallet_time_n,
        NodeRewards: pallet_node_rewards,
    }
);

pub type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<AccountId, RuntimeCall, Signature, SignedExtra>;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type Executive = frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

parameter_types! {
    pub const SS58Prefix: u8 = 42;
}

parameter_types! {
    pub const Version: sp_version::RuntimeVersion = sp_version::RuntimeVersion {
        spec_name: create_runtime_str!("calcx"),
        impl_name: create_runtime_str!("calcx"),
        authoring_version: 1,
        spec_version: 100,
        impl_version: 1,
        apis: sp_version::create_apis_vec!([]),
        transaction_version: 1,
        system_version: 1,
    };
}

#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type RuntimeTask = RuntimeTask;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU32<2400>;
    type Version = Version;
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
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


parameter_types! {
    pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/trsry");
    pub const TreasurySpendPeriod: u32 = 100;
    pub const TreasuryBurn: sp_runtime::Permill = sp_runtime::Permill::from_percent(0);
    pub const TreasuryMaxApprovals: u32 = 100;
    pub const TreasuryPayoutPeriod: u32 = 100;
}

impl pallet_treasury::Config for Runtime {
    type Currency = Balances;
    type RejectOrigin = frame_system::EnsureRoot<AccountId>;
    type RuntimeEvent = RuntimeEvent;
    type SpendPeriod = TreasurySpendPeriod;
    type Burn = TreasuryBurn;
    type PalletId = TreasuryPalletId;
    type BurnDestination = ();
    type WeightInfo = ();
    type SpendFunds = ();
    type MaxApprovals = TreasuryMaxApprovals;
    type SpendOrigin = frame_system::EnsureRootWithSuccess<AccountId, frame_support::traits::ConstU128<{u128::MAX}>>;
    type AssetKind = ();
    type Beneficiary = AccountId;
    type BeneficiaryLookup = sp_runtime::traits::IdentityLookup<AccountId>;
    type Paymaster = frame_support::traits::tokens::PayFromAccount<pallet_balances::Pallet<Runtime>, TreasuryPot>;
    type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
    type PayoutPeriod = TreasuryPayoutPeriod;
    type BlockNumberProvider = System;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = Aura;
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const SlotDuration: u64 = 6000;
}

impl pallet_aura::Config for Runtime {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
    type MaxAuthorities = ConstU32<32>;
    type AllowMultipleBlocksPerSlot = ();
    type SlotDuration = ConstU64<6000>;
}

impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_session::MembershipProof;
    type EquivocationReportSystem = ();
}

parameter_types! {
    pub const AssetDeposit: Balance = 100;
    pub const ApprovalDeposit: Balance = 1;
    pub const AssetsStringLimit: u32 = 50;
    pub const MetadataDepositBase: Balance = 10;
    pub const MetadataDepositPerByte: Balance = 1;
}

impl pallet_assets::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = u32;
    type AssetIdParameter = codec::Compact<u32>;
    type Currency = Balances;
    type CreateOrigin = frame_system::EnsureSigned<AccountId>;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type AssetDeposit = AssetDeposit;
    type MetadataDepositBase = MetadataDepositBase;
    type MetadataDepositPerByte = MetadataDepositPerByte;
    type ApprovalDeposit = ApprovalDeposit;
    type StringLimit = AssetsStringLimit;
    type Freezer = ();
    type Extra = ();
    type CallbackHandle = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<1000>;
    type ReserveData = ();
    type AssetAccountDeposit = ConstU128<10>;
    type Holder = ();
    type AssetIdAllocator = ();
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = ();
}

impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
}

parameter_types! {
    pub const CalcxMinWindow: sp_runtime::FixedU128 = sp_runtime::FixedU128::from_rational(1001u128, 1000u128);
}

impl pallet_calcx::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type AssetId = u32;
    type MinWindow = CalcxMinWindow;
    type Currency = Balances;
    type TreasuryAccount = TreasuryPot;
    type PriceToCxScale = ConstU128<1000>;
    type PriceProvider = pallet_calcx::StoragePriceProvider<Runtime>;
}

impl pallet_time_n::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
    pub const CxPerTx: u128 = 1;
}

impl pallet_node_rewards::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CxPerTx = CxPerTx;
}

impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = sp_weights::IdentityFee<Balance>;
    type LengthToFee = sp_weights::IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    type WeightInfo = ();
}


pub const WASM_BINARY: Option<&[u8]> = None;

sp_api::impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> sp_version::RuntimeVersion {
            Version::get()
        }

        fn execute_block(block: <Block as BlockT>::LazyBlock) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(json: Vec<u8>) -> sp_genesis_builder::Result {
            frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(json)
        }

        fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }

        fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
            vec![]
        }
    }
}

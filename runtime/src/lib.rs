#![cfg_attr(not(feature = "std"), no_std)]
//
//! BitNice Runtime
//! 兼容 Polkadot SDK 的简化模板，满足如下需求：
//! - 共识算法可扩展（后续可集成 PoW）
//! - 区块时间 6 秒
//! - 代币名称：BitNice，符号：BN，总量：2,100,000,000
//! - Pallet: System、Timestamp、Balances、TransactionPayment、Sudo

extern crate alloc;
use alloc::vec::Vec;
use sp_core::OpaqueMetadata;
use sp_runtime::{
    generic,
    traits::{BlakeTwo256, IdentifyAccount, Verify},
    MultiAddress, MultiSignature,
};
use sp_version::RuntimeVersion;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_sudo::Call as SudoCall;
pub use pallet_timestamp::Call as TimestampCall;

// WASM 二进制
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// 运行时版本信息
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: alloc::borrow::Cow::Borrowed("bitnice"),
    impl_name: alloc::borrow::Cow::Borrowed("bitnice"),
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    apis: alloc::borrow::Cow::Borrowed(&[]),
    transaction_version: 1,
    system_version: 1,
};

/// 区块时间相关常量
pub const MILLI_SECS_PER_BLOCK: u64 = 6000;
pub const SLOT_DURATION: u64 = MILLI_SECS_PER_BLOCK;

/// 代币单位与总量
pub const UNIT: u128 = 1_000_000_000_000;
pub const TOTAL_SUPPLY: u128 = 2_100_000_000 * UNIT;
pub const EXISTENTIAL_DEPOSIT: u128 = UNIT / 1000;

/// 类型定义
pub type Signature = MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Balance = u128;
pub type Nonce = u32;
pub type Hash = sp_core::H256;
pub type BlockNumber = u32;
pub type Address = MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, ()>;

/// Opaque 类型定义
pub mod opaque {
    pub use super::{Block, Header, UncheckedExtrinsic};
    pub type BlockId = sp_runtime::generic::BlockId<Block>;
}

// BitNice 运行时结构体，集成所有必要的 Pallets
frame_support::construct_runtime!(
    pub enum Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
    }
);

/// System Pallet 配置
impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = sp_runtime::traits::AccountIdLookup<AccountId, ()>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = frame_support::traits::ConstU32<2400>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = frame_support::traits::ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type RuntimeTask = ();
    type ExtensionsWeightInfo = ();
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

/// Timestamp Pallet 配置
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = frame_support::traits::ConstU64<{ MILLI_SECS_PER_BLOCK / 2 }>;
    type WeightInfo = ();
}

/// Balances Pallet 配置
impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = frame_support::traits::ConstU128<EXISTENTIAL_DEPOSIT>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = frame_support::traits::ConstU32<50>;
    type MaxReserves = frame_support::traits::ConstU32<50>;
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
    type DoneSlashHandler = ();
}

/// TransactionPayment Pallet 配置
impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = frame_support::traits::ConstU8<5>;
    type WeightToFee = frame_support::weights::IdentityFee<Balance>;
    type LengthToFee = frame_support::weights::IdentityFee<Balance>;
    type FeeMultiplierUpdate = ();
    type WeightInfo = pallet_transaction_payment::weights::SubstrateWeight<Runtime>;
}

/// Sudo Pallet 配置
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = ();
}

// BitNice 运行时 API 实现，提供区块链核心接口
sp_api::impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }
        fn execute_block(block: Block) {
            frame_executive::Executive::<
                Runtime,
                Block,
                frame_system::ChainContext<Runtime>,
                Runtime,
                AllPalletsWithSystem,
                ()
            >::execute_block(block);
        }
        fn initialize_block(header: &<Block as sp_runtime::traits::Block>::Header) -> sp_runtime::ExtrinsicInclusionMode {
            frame_executive::Executive::<
                Runtime,
                Block,
                frame_system::ChainContext<Runtime>,
                Runtime,
                AllPalletsWithSystem,
                ()
            >::initialize_block(header)
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }
        fn metadata_versions() -> Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as sp_runtime::traits::Block>::Extrinsic) -> sp_runtime::ApplyExtrinsicResult {
            frame_executive::Executive::<
                Runtime,
                Block,
                frame_system::ChainContext<Runtime>,
                Runtime,
                AllPalletsWithSystem,
                ()
            >::apply_extrinsic(extrinsic)
        }
        fn finalize_block() -> <Block as sp_runtime::traits::Block>::Header {
            frame_executive::Executive::<
                Runtime,
                Block,
                frame_system::ChainContext<Runtime>,
                Runtime,
                AllPalletsWithSystem,
                ()
            >::finalize_block()
        }
        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as sp_runtime::traits::Block>::Extrinsic> {
            data.create_extrinsics()
        }
        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: sp_runtime::transaction_validity::TransactionSource,
            tx: <Block as sp_runtime::traits::Block>::Extrinsic,
            block_hash: <Block as sp_runtime::traits::Block>::Hash,
        ) -> sp_runtime::transaction_validity::TransactionValidity {
            frame_executive::Executive::<
                Runtime,
                Block,
                frame_system::ChainContext<Runtime>,
                Runtime,
                AllPalletsWithSystem,
                ()
            >::validate_transaction(source, tx, block_hash)
        }
    }

    // PoW 难度接口
    impl sp_consensus_pow::DifficultyApi<Block, u128> for Runtime {
        fn difficulty() -> u128 {
            1000000
        }
    }

    // TransactionPayment RPC API
    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as sp_runtime::traits::Block>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        
        fn query_fee_details(
            uxt: <Block as sp_runtime::traits::Block>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        
        fn query_weight_to_fee(weight: sp_runtime::Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    // System RPC API
    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }
}

// 实现 PoW 难度接口

/// 运行时原生版本（仅 std 环境）
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

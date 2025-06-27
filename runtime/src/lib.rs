//! # BitNice Runtime
//! 
//! 基于比特币 PoW 共识算法的区块链运行时
//! 
//! - 代币名称：BitNice
//! - 代币符号：BN  
//! - 代币总量：2.1 亿
//! - 区块时间：6 秒
//! - 共识算法：PoW (类似比特币)

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

// 导入 polkadot-sdk 必要组件
extern crate alloc;
use alloc::{vec, vec::Vec};

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

// Polkadot SDK imports
use polkadot_sdk::{
    frame_support::{
        construct_runtime, derive_impl,
        dispatch::DispatchClass,
        genesis_builder_helper::{build_state, get_preset},
        parameter_types,
        traits::{
            ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, EitherOfDiverse,
            KeyOwnerProofSystem, Randomness, StorageInfo,
        },
        weights::{
            constants::WEIGHT_REF_TIME_PER_SECOND, IdentityFee, Weight, WeightToFeeCoefficient,
            WeightToFeeCoefficients, WeightToFeePolynomial,
        },
        PalletId,
    },
    frame_system::{
        limits::{BlockHashCount, BlockLength, BlockWeights},
        EnsureRoot,
    },
    pallet_grandpa::{
        fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
    },
    pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier},
    sp_api::impl_runtime_apis,
    sp_consensus_aura::sr25519::AuthorityId as AuraId,
    sp_core::{crypto::KeyTypeId, OpaqueMetadata, H256, U256},
    sp_runtime::{
        create_runtime_str, generic, impl_opaque_keys,
        traits::{
            AccountIdLookup, BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One,
            Saturating, StaticLookup, Verify,
        },
        transaction_validity::{TransactionSource, TransactionValidity},
        ApplyExtrinsicResult, FixedPointNumber, Perbill, Permill,
    },
    sp_std::prelude::*,
    sp_version::RuntimeVersion,
};

// PoW 相关导入
use polkadot_sdk::sp_consensus_pow::{Seal, TotalDifficulty};
use primitive_types::U256 as PrimitiveU256;
use sha2::{Digest, Sha256};

// 本地导入
use pallet_template;

/// 区块链的不透明类型。这些类型供客户端使用，不透明是指
/// 客户端无需知道这些类型的内部结构
pub mod opaque {
    use super::*;
    use polkadot_sdk::sp_runtime::{generic, traits::BlakeTwo256};

    pub use polkadot_sdk::sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// 不透明区块头类型
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// 不透明区块类型
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// 不透明区块标识符类型
    pub type BlockId = generic::BlockId<Block>;

    impl_opaque_keys! {
        pub struct SessionKeys {
            pub grandpa: Grandpa,
        }
    }
}

// 运行时版本信息
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("bitnice"),
    impl_name: create_runtime_str!("bitnice"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version: 1,
};

/// 运行时中使用的类型定义
pub type Signature = polkadot_sdk::sp_runtime::MultiSignature;
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Address = polkadot_sdk::sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic =
    generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
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

/// 区块类型
pub type BlockNumber = u32;
/// 余额类型
pub type Balance = u128;
/// 随机数类型 
pub type Nonce = u32;
/// 哈希类型
pub type Hash = H256;

/// 运行时常量参数
parameter_types! {
    pub const BlockHashCount: BlockNumber = 2400;
    pub const Version: RuntimeVersion = VERSION;
    
    /// 区块权重限制 (6秒区块时间)
    pub BlockWeights: frame_system::limits::BlockWeights =
        frame_system::limits::BlockWeights::with_sensible_defaults(
            Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
            Perbill::from_percent(75),
        );
    pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
        ::max_with_normal_ratio(5 * 1024 * 1024, Perbill::from_percent(75));
    pub const SS58Prefix: u8 = 42;
}

/// 系统 pallet 配置
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig as frame_system::DefaultConfig)]
impl frame_system::Config for Runtime {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = BlockWeights;
    type BlockLength = BlockLength;
    type AccountId = AccountId;
    type RuntimeCall = RuntimeCall;
    type Nonce = Nonce;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type RuntimeOrigin = RuntimeOrigin;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = Version;
    type PalletInfo = PalletInfo;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type SystemWeightInfo = ();
    type SS58Prefix = SS58Prefix;
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub const MinimumPeriod: u64 = 3000; // 6秒区块时间的一半 (3秒)
}

/// 时间戳 pallet 配置
impl pallet_timestamp::Config for Runtime {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    /// BitNice 代币总量：2.1 亿 BN
    pub const ExistentialDeposit: u128 = 500;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

/// 余额 pallet 配置
impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
}

parameter_types! {
    pub FeeMultiplier: Multiplier = Multiplier::one();
}

/// 交易支付 pallet 配置
impl pallet_transaction_payment::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = FungibleAdapter<Balances, ()>;
    type OperationalFeeMultiplier = ConstU8<5>;
    type WeightToFee = IdentityFee<Balance>;
    type LengthToFee = IdentityFee<Balance>;
    type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

/// Sudo pallet 配置
impl pallet_sudo::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

/// 实用工具 pallet 配置
impl pallet_utility::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type RuntimeCall = RuntimeCall;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

/// 作者身份 pallet 配置 (PoW 需要)
impl pallet_authorship::Config for Runtime {
    type FindAuthor = ();
    type EventHandler = ();
}

/// 模板 pallet 配置
impl pallet_template::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_template::WeightInfo;
}

/// GRANDPA pallet 配置 (用于最终性)
impl pallet_grandpa::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type MaxAuthorities = ConstU32<32>;
    type MaxNominators = ConstU32<0>;
    type MaxSetIdSessionEntries = ConstU64<0>;
    type KeyOwnerProof = sp_core::Void;
    type EquivocationReportSystem = ();
}

// 构造运行时
construct_runtime!(
    pub struct Runtime {
        System: frame_system,
        Timestamp: pallet_timestamp,
        Balances: pallet_balances,
        TransactionPayment: pallet_transaction_payment,
        Sudo: pallet_sudo,
        Utility: pallet_utility,
        Authorship: pallet_authorship,
        Grandpa: pallet_grandpa,
        TemplateModule: pallet_template,
    }
);

/// 运行时 API 版本
const RUNTIME_API_VERSIONS: polkadot_sdk::sp_version::ApisVec = polkadot_sdk::sp_version::create_apis_vec![
    [polkadot_sdk::sp_api::Core::<Block>::ID, polkadot_sdk::sp_api::Core::<Block>::VERSION],
    [polkadot_sdk::sp_api::Metadata::<Block>::ID, polkadot_sdk::sp_api::Metadata::<Block>::VERSION],
    [polkadot_sdk::sp_block_builder::BlockBuilder::<Block>::ID, polkadot_sdk::sp_block_builder::BlockBuilder::<Block>::VERSION],
    [polkadot_sdk::sp_transaction_pool::runtime_api::TaggedTransactionQueue::<Block>::ID, polkadot_sdk::sp_transaction_pool::runtime_api::TaggedTransactionQueue::<Block>::VERSION],
    [polkadot_sdk::sp_offchain::OffchainWorkerApi::<Block>::ID, polkadot_sdk::sp_offchain::OffchainWorkerApi::<Block>::VERSION],
    [polkadot_sdk::sp_session::SessionKeys::<Block>::ID, polkadot_sdk::sp_session::SessionKeys::<Block>::VERSION],
    [polkadot_sdk::sp_consensus_grandpa::GrandpaApi::<Block>::ID, polkadot_sdk::sp_consensus_grandpa::GrandpaApi::<Block>::VERSION],
    [polkadot_sdk::frame_system_rpc_runtime_api::AccountNonceApi::<Block, AccountId, Nonce>::ID, polkadot_sdk::frame_system_rpc_runtime_api::AccountNonceApi::<Block, AccountId, Nonce>::VERSION],
    [polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi::<Block, Balance>::ID, polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi::<Block, Balance>::VERSION],
    [polkadot_sdk::sp_genesis_builder::GenesisBuilder::<Block>::ID, polkadot_sdk::sp_genesis_builder::GenesisBuilder::<Block>::VERSION],
];

/// PoW 算法实现 (类似比特币的 SHA-256)
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct BitcoinPowAlgorithm;

impl polkadot_sdk::sp_consensus_pow::PowAlgorithm<H256> for BitcoinPowAlgorithm {
    type Difficulty = U256;

    fn difficulty(&self, _parent: H256) -> Result<Self::Difficulty, polkadot_sdk::sp_consensus_pow::Error<H256>> {
        // 简化的难度调整 - 实际项目中应该根据区块时间动态调整
        Ok(U256::from(1000u64))
    }

    fn verify(
        &self,
        _parent: &H256,
        pre_hash: &H256,
        _pre_runtime: Option<&[u8]>,
        seal: &Seal,
        difficulty: Self::Difficulty,
    ) -> Result<bool, polkadot_sdk::sp_consensus_pow::Error<H256>> {
        // 验证 PoW 工作量证明
        let hash = Sha256::digest(&[pre_hash.as_bytes(), seal].concat());
        let hash_value = U256::from_big_endian(hash.as_slice());
        
        // 检查哈希值是否小于目标难度
        Ok(hash_value < difficulty)
    }
}

// 实现运行时 API
impl_runtime_apis! {
    impl polkadot_sdk::sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block);
        }

        fn initialize_block(header: &<Block as BlockT>::Header) -> polkadot_sdk::sp_runtime::ExtrinsicInclusionMode {
            Executive::initialize_block(header)
        }
    }

    impl polkadot_sdk::sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }

        fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
            Runtime::metadata_at_version(version)
        }

        fn metadata_versions() -> alloc::vec::Vec<u32> {
            Runtime::metadata_versions()
        }
    }

    impl polkadot_sdk::sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: polkadot_sdk::sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: polkadot_sdk::sp_inherents::InherentData,
        ) -> polkadot_sdk::sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

    impl polkadot_sdk::sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl polkadot_sdk::sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl polkadot_sdk::sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
        fn grandpa_authorities() -> GrandpaAuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn current_set_id() -> polkadot_sdk::sp_consensus_grandpa::SetId {
            Grandpa::current_set_id()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: polkadot_sdk::sp_consensus_grandpa::EquivocationProof<
                <Block as BlockT>::Hash,
                NumberFor<Block>,
            >,
            _key_owner_proof: polkadot_sdk::sp_consensus_grandpa::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }

        fn generate_key_ownership_proof(
            _set_id: polkadot_sdk::sp_consensus_grandpa::SetId,
            _authority_id: GrandpaId,
        ) -> Option<polkadot_sdk::sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
            None
        }
    }

    impl polkadot_sdk::frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
        fn account_nonce(account: AccountId) -> Nonce {
            System::account_nonce(account)
        }
    }

    impl polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> polkadot_sdk::pallet_transaction_payment_rpc_runtime_api::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }
        fn query_weight_to_fee(weight: Weight) -> Balance {
            TransactionPayment::weight_to_fee(weight)
        }
        fn query_length_to_fee(length: u32) -> Balance {
            TransactionPayment::length_to_fee(length)
        }
    }

    impl polkadot_sdk::sp_genesis_builder::GenesisBuilder<Block> for Runtime {
        fn build_state(config: Vec<u8>) -> polkadot_sdk::sp_genesis_builder::Result {
            build_state::<RuntimeGenesisConfig>(config)
        }

        fn get_preset(id: &Option<polkadot_sdk::sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
            get_preset::<RuntimeGenesisConfig>(id, |_| None)
        }
    }
}

/// Executive：负责协调运行时中各 pallet 的执行
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
>;

type EventRecord = frame_system::EventRecord<
    <Runtime as frame_system::Config>::RuntimeEvent,
    <Runtime as frame_system::Config>::Hash,
>;

/// Wasm binary
pub const WASM_BINARY: Option<&[u8]> = include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

/// Runtime API Dispatch
pub struct RuntimeApiDispatch;

impl RuntimeApiDispatch {
    pub fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        match method {
            _ => None,
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
mod benches {
    frame_benchmarking::define_benchmarks!(
        [frame_system, SystemBench::<Runtime>]
        [pallet_balances, Balances]
        [pallet_timestamp, Timestamp]
        [pallet_template, TemplateModule]
    );
} 
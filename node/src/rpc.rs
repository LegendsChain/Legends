//! BitNice 节点 RPC 服务配置
//!
//! 极简版本 - 只提供基本的空RPC服务

use std::sync::Arc;

use jsonrpsee::RpcModule;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

use bitnice_runtime::{opaque::Block, AccountId, Balance, Nonce};

/// 完整节点 RPC 依赖
pub struct FullDeps<C, P> {
    /// 客户端引用
    pub client: Arc<C>,
    /// 交易池引用
    pub pool: Arc<P>,
    /// 是否拒绝不安全的 RPC 调用
    pub deny_unsafe: sc_rpc_api::DenyUnsafe,
}

/// 创建完整节点的 RPC 服务
pub fn create_full<C, P>(
    _deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: BlockBuilder<Block>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>,
    P: TransactionPool<Block = Block> + Send + Sync + 'static,
{
    // 创建空的RPC模块 - 暂时不添加任何功能
    let io = RpcModule::new(());
    
    // TODO: 等API稳定后重新添加RPC功能
    // - 系统RPC调用
    // - 交易支付RPC调用  
    // - 自定义BitNice RPC调用
    
    Ok(io)
}

/// 挖矿统计信息结构
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MiningStats {
    /// 当前难度
    pub current_difficulty: u64,
    /// 网络哈希率
    pub network_hashrate: u64,
    /// 区块时间（秒）
    pub block_time: u64,
    /// 今日挖出的区块数
    pub blocks_mined_today: u64,
}

/// RPC 错误类型
#[derive(Debug, thiserror::Error)]
pub enum RpcError {
    #[error("客户端错误: {0}")]
    Client(String),

    #[error("交易池错误: {0}")]
    TransactionPool(String),

    #[error("运行时错误: {0}")]
    Runtime(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

impl From<RpcError> for jsonrpsee::core::Error {
    fn from(error: RpcError) -> Self {
        match error {
            RpcError::Client(msg) => jsonrpsee::core::Error::Custom(msg),
            RpcError::TransactionPool(msg) => jsonrpsee::core::Error::Custom(msg),
            RpcError::Runtime(msg) => jsonrpsee::core::Error::Custom(msg),
            RpcError::Internal(msg) => jsonrpsee::core::Error::Custom(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mining_stats_serialization() {
        let stats = MiningStats {
            current_difficulty: 1000,
            network_hashrate: 1000000,
            block_time: 6,
            blocks_mined_today: 14400,
        };

        // 测试序列化
        let serialized = serde_json::to_string(&stats).unwrap();
        assert!(serialized.contains("current_difficulty"));
        assert!(serialized.contains("1000"));

        // 测试反序列化
        let deserialized: MiningStats = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.current_difficulty, 1000);
        assert_eq!(deserialized.network_hashrate, 1000000);
        assert_eq!(deserialized.block_time, 6);
        assert_eq!(deserialized.blocks_mined_today, 14400);
    }

    #[test]
    fn test_rpc_error_conversion() {
        let error = RpcError::Client("测试错误".to_string());
        let jsonrpc_error: jsonrpsee::core::Error = error.into();

        // 确保错误能够正确转换
        assert!(matches!(jsonrpc_error, jsonrpsee::core::Error::Custom(_)));
    }
}

// 这个文件实现了 BitNice 的 PoW 算法
// 基于比特币的 SHA256 双重哈希算法

use sp_consensus_pow::{Seal, TotalDifficulty, POW_ENGINE_ID};
use sp_core::{H256, U256};
use sp_runtime::traits::{Block as BlockT, Header as HeaderT};
use sp_runtime::generic::BlockId;
use codec::{Encode, Decode};
use sha2::{Sha256, Digest};

#[cfg(feature = "std")]
use std::sync::Arc;

/// BitNice PoW 算法参数
pub struct BitNicePowAlgorithm {
    /// 目标区块时间（秒）
    pub target_block_time: u64,
    /// 难度调整窗口（区块数）
    pub difficulty_adjustment_window: u32,
    /// 最大难度变化倍数
    pub max_difficulty_change: f64,
    /// 最小难度
    pub min_difficulty: U256,
}

impl Default for BitNicePowAlgorithm {
    fn default() -> Self {
        Self {
            target_block_time: 6, // 6秒区块时间
            difficulty_adjustment_window: 100, // 每100个区块调整一次
            max_difficulty_change: 4.0, // 最大4倍变化
            min_difficulty: U256::from(1_000u64), // 最小难度
        }
    }
}

/// BitNice PoW Seal 结构
#[derive(Clone, PartialEq, Eq, Encode, Decode, Debug)]
pub struct BitNicePoWSeal {
    /// Nonce 值
    pub nonce: u64,
    /// 工作量证明哈希
    pub work: H256,
}

impl BitNicePowAlgorithm {
    /// 创建新的 BitNice PoW 算法实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算 SHA256 双重哈希（类比特币）
    pub fn sha256d(data: &[u8]) -> H256 {
        let first_hash = Sha256::digest(data);
        let second_hash = Sha256::digest(&first_hash);
        H256::from_slice(&second_hash)
    }

    /// 验证工作量证明
    pub fn verify_pow(
        &self,
        pre_hash: &H256,
        nonce: u64,
        difficulty: U256,
    ) -> bool {
        let mut input = Vec::new();
        input.extend_from_slice(pre_hash.as_bytes());
        input.extend_from_slice(&nonce.to_le_bytes());

        let work_hash = Self::sha256d(&input);
        let work_value = U256::from(work_hash.as_bytes());
        
        // 检查工作量是否满足难度要求
        // 难度越高，要求的哈希值越小
        let target = U256::max_value() / difficulty;
        work_value <= target
    }

    /// 计算区块的工作量
    pub fn calculate_work(&self, pre_hash: &H256, nonce: u64) -> H256 {
        let mut input = Vec::new();
        input.extend_from_slice(pre_hash.as_bytes());
        input.extend_from_slice(&nonce.to_le_bytes());
        Self::sha256d(&input)
    }

    /// 难度调整算法
    /// 根据实际区块时间vs目标区块时间来调整难度
    pub fn adjust_difficulty(
        &self,
        current_difficulty: U256,
        actual_time: u64,
        target_time: u64,
    ) -> U256 {
        if actual_time == 0 {
            return current_difficulty;
        }

        // 计算时间比率
        let time_ratio = actual_time as f64 / target_time as f64;
        
        // 限制最大变化倍数
        let clamped_ratio = if time_ratio > self.max_difficulty_change {
            self.max_difficulty_change
        } else if time_ratio < (1.0 / self.max_difficulty_change) {
            1.0 / self.max_difficulty_change
        } else {
            time_ratio
        };

        // 调整难度：时间长了难度降低，时间短了难度提高
        let new_difficulty = if clamped_ratio > 1.0 {
            // 实际时间比目标时间长，降低难度
            current_difficulty / U256::from((clamped_ratio * 1000.0) as u64) * U256::from(1000u64)
        } else {
            // 实际时间比目标时间短，提高难度
            current_difficulty * U256::from((1000.0 / clamped_ratio) as u64) / U256::from(1000u64)
        };

        // 确保不低于最小难度
        if new_difficulty < self.min_difficulty {
            self.min_difficulty
        } else {
            new_difficulty
        }
    }

    /// 获取初始难度
    pub fn initial_difficulty() -> U256 {
        U256::from(1_000_000u64) // 初始难度
    }
}

/// PoW 相关的辅助函数
pub mod pow_utils {
    use super::*;

    /// 从区块头提取 pre-hash
    pub fn extract_pre_hash<Block: BlockT>(header: &Block::Header) -> H256 {
        let mut header_copy = header.clone();
        // 清除 seal 部分，只保留区块头的其他内容
        *header_copy.seal_mut() = Default::default();
        header_copy.hash()
    }

    /// 创建 PoW seal
    pub fn create_seal(nonce: u64, work: H256) -> Seal {
        let seal = BitNicePoWSeal { nonce, work };
        seal.encode()
    }

    /// 解析 PoW seal
    pub fn decode_seal(seal: &Seal) -> Result<BitNicePoWSeal, codec::Error> {
        BitNicePoWSeal::decode(&mut &seal[..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256d() {
        let data = b"hello world";
        let hash = BitNicePowAlgorithm::sha256d(data);
        
        // 验证双重哈希的结果
        let expected_first = Sha256::digest(data);
        let expected_second = Sha256::digest(&expected_first);
        let expected_final = H256::from_slice(&expected_second);
        
        assert_eq!(hash, expected_final);
    }

    #[test]
    fn test_pow_verification() {
        let algorithm = BitNicePowAlgorithm::new();
        let pre_hash = H256::random();
        let nonce = 12345u64;
        
        // 计算工作量
        let work = algorithm.calculate_work(&pre_hash, nonce);
        
        // 使用较低的难度进行测试
        let low_difficulty = U256::from(1u64);
        let result = algorithm.verify_pow(&pre_hash, nonce, low_difficulty);
        
        // 应该通过验证（因为难度很低）
        assert!(result);
    }

    #[test]
    fn test_difficulty_adjustment() {
        let algorithm = BitNicePowAlgorithm::new();
        let current_difficulty = U256::from(1000000u64);
        
        // 测试实际时间比目标时间长的情况（应该降低难度）
        let new_difficulty = algorithm.adjust_difficulty(
            current_difficulty,
            12, // 实际12秒
            6,  // 目标6秒
        );
        assert!(new_difficulty < current_difficulty);
        
        // 测试实际时间比目标时间短的情况（应该提高难度）
        let new_difficulty = algorithm.adjust_difficulty(
            current_difficulty,
            3, // 实际3秒
            6, // 目标6秒
        );
        assert!(new_difficulty > current_difficulty);
    }
} 
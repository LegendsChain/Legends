// BitNice PoW 共识算法的 Substrate 集成实现

use crate::pow_algorithm::{BitNicePowAlgorithm, pow_utils};
use sp_consensus_pow::{Error, PowAlgorithm, Seal, TotalDifficulty};
use sp_core::{H256, U256};
use sp_runtime::{
    traits::{Block as BlockT, Header as HeaderT, Zero},
    generic::BlockId,
};
use codec::Decode;

#[cfg(feature = "std")]
use std::{sync::Arc, collections::HashMap};

/// BitNice PoW 共识算法的 Substrate 适配器
pub struct BitNicePowEngine {
    /// 内部算法实现
    algorithm: BitNicePowAlgorithm,
    /// 难度缓存，用于存储历史难度信息（仅在std模式下可用）
    #[cfg(feature = "std")]
    difficulty_cache: Arc<std::sync::Mutex<HashMap<H256, U256>>>,
}

impl BitNicePowEngine {
    /// 创建新的 BitNice PoW 引擎
    pub fn new() -> Self {
        Self {
            algorithm: BitNicePowAlgorithm::new(),
            #[cfg(feature = "std")]
            difficulty_cache: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// 计算指定区块的难度
    fn calculate_difficulty_for_block<Block: BlockT>(
        &self,
        client: &Arc<dyn sp_blockchain::HeaderBackend<Block>>,
        parent_hash: &Block::Hash,
    ) -> Result<U256, Error<Block>> {
        // 获取父区块信息
        let parent_header = client
            .header(BlockId::Hash(*parent_hash))
            .map_err(|e| Error::Environment(format!("Failed to get parent header: {:?}", e)))?
            .ok_or_else(|| Error::Environment("Parent header not found".to_string()))?;

        let parent_number = *parent_header.number();

        // 如果是创世区块的下一块，使用初始难度
        if parent_number.is_zero() {
            return Ok(BitNicePowAlgorithm::initial_difficulty());
        }

        // 检查是否需要调整难度
        let current_block_number = parent_number + 1u32.into();
        if current_block_number % self.algorithm.difficulty_adjustment_window.into() != Zero::zero() {
            // 不需要调整难度，使用父区块的难度
            return self.get_difficulty_from_header(&parent_header);
        }

        // 需要调整难度，计算过去窗口内的平均区块时间
        let window_start_number = if current_block_number > self.algorithm.difficulty_adjustment_window.into() {
            current_block_number - self.algorithm.difficulty_adjustment_window.into()
        } else {
            1u32.into()
        };

        // 获取窗口开始和结束的时间戳
        let window_start_header = self.get_header_by_number(client, window_start_number)?;
        let window_end_header = parent_header;

        let window_start_time = self.extract_timestamp(&window_start_header)?;
        let window_end_time = self.extract_timestamp(&window_end_header)?;

        // 计算实际经过的时间
        let actual_time = window_end_time - window_start_time;
        let blocks_in_window = *window_end_header.number() - *window_start_header.number();
        let avg_block_time = if blocks_in_window > Zero::zero() {
            actual_time / blocks_in_window.saturated_into::<u64>()
        } else {
            self.algorithm.target_block_time
        };

        // 获取当前难度并调整
        let current_difficulty = self.get_difficulty_from_header(&parent_header)?;
        let new_difficulty = self.algorithm.adjust_difficulty(
            current_difficulty,
            avg_block_time,
            self.algorithm.target_block_time,
        );

        // 在no_std环境中，我们暂时不使用日志
        #[cfg(feature = "std")]
        log::info!(
            "💎 难度调整: 块 {} -> {}, 平均时间 {}s -> 目标 {}s, 难度 {} -> {}",
            window_start_number,
            current_block_number,
            avg_block_time,
            self.algorithm.target_block_time,
            current_difficulty,
            new_difficulty
        );

        Ok(new_difficulty)
    }

    /// 从区块头中提取时间戳
    fn extract_timestamp<Header: HeaderT>(&self, header: &Header) -> Result<u64, Error<Header::Number>> {
        // 在真实实现中，我们需要从区块头的digest中提取时间戳
        // 这里先用区块号作为时间戳的近似（仅用于演示）
        Ok((*header.number()).saturated_into::<u64>() * self.algorithm.target_block_time)
    }

    /// 根据区块号获取区块头
    fn get_header_by_number<Block: BlockT>(
        &self,
        client: &Arc<dyn sp_blockchain::HeaderBackend<Block>>,
        number: Block::Header::Number,
    ) -> Result<Block::Header, Error<Block>> {
        let hash = client
            .hash(number)
            .map_err(|e| Error::Environment(format!("Failed to get hash for block {}: {:?}", number, e)))?
            .ok_or_else(|| Error::Environment(format!("Block {} not found", number)))?;

        client
            .header(BlockId::Hash(hash))
            .map_err(|e| Error::Environment(format!("Failed to get header: {:?}", e)))?
            .ok_or_else(|| Error::Environment("Header not found".to_string()))
    }

    /// 从区块头中提取难度信息
    fn get_difficulty_from_header<Header: HeaderT>(&self, header: &Header) -> Result<U256, Error<Header::Number>> {
        // 在真实实现中，难度信息应该存储在区块头的digest中
        // 这里我们先返回一个默认值
        Ok(BitNicePowAlgorithm::initial_difficulty())
    }
}

impl<Block: BlockT<Hash = H256>> PowAlgorithm<Block> for BitNicePowEngine {
    type Difficulty = U256;

    fn difficulty(&self, parent: &BlockId<Block>) -> Result<Self::Difficulty, Error<Block>> {
        match parent {
            BlockId::Hash(hash) => {
                // 检查缓存（仅在std模式下）
                #[cfg(feature = "std")]
                {
                    if let Ok(cache) = self.difficulty_cache.lock() {
                        if let Some(&difficulty) = cache.get(hash) {
                            return Ok(difficulty);
                        }
                    }
                }

                // 如果是创世区块，返回初始难度
                if *hash == Default::default() {
                    return Ok(BitNicePowAlgorithm::initial_difficulty());
                }

                // 这里需要访问区块链客户端来计算难度
                // 在实际实现中，这个方法会接收客户端引用
                Ok(BitNicePowAlgorithm::initial_difficulty())
            },
            BlockId::Number(_) => {
                // 根据区块号计算难度
                Ok(BitNicePowAlgorithm::initial_difficulty())
            }
        }
    }

    fn verify(
        &self,
        parent: &BlockId<Block>,
        pre_hash: &H256,
        seal: &Seal,
        difficulty: Self::Difficulty,
    ) -> Result<bool, Error<Block>> {
        // 解析 seal
        let pow_seal = pow_utils::decode_seal(seal)
            .map_err(|e| Error::Environment(format!("Failed to decode seal: {:?}", e)))?;

        // 验证工作量证明
        let is_valid = self.algorithm.verify_pow(pre_hash, pow_seal.nonce, difficulty);

        #[cfg(feature = "std")]
        {
            if is_valid {
                log::debug!(
                    "✅ PoW 验证成功: nonce={}, difficulty={}, hash={}",
                    pow_seal.nonce,
                    difficulty,
                    pow_seal.work
                );
            } else {
                log::warn!(
                    "❌ PoW 验证失败: nonce={}, difficulty={}, hash={}",
                    pow_seal.nonce,
                    difficulty,
                    pow_seal.work
                );
            }
        }

        Ok(is_valid)
    }

    fn mine(
        &self,
        parent: &BlockId<Block>,
        pre_hash: &H256,
        difficulty: Self::Difficulty,
        round: u32,
    ) -> Result<Option<Seal>, Error<Block>> {
        #[cfg(feature = "std")]
        log::info!(
            "⛏️  开始挖矿: pre_hash={}, difficulty={}, round={}",
            pre_hash,
            difficulty,
            round
        );

        // 基于轮次计算起始nonce，避免重复工作
        let start_nonce = (round as u64) * 100_000;
        let max_iterations = 100_000u64; // 每轮最多尝试10万次

        for nonce in start_nonce..(start_nonce + max_iterations) {
            // 计算工作量
            let work = self.algorithm.calculate_work(pre_hash, nonce);
            
            // 检查是否满足难度要求
            if self.algorithm.verify_pow(pre_hash, nonce, difficulty) {
                #[cfg(feature = "std")]
                log::info!(
                    "🎉 找到PoW解! nonce={}, work={}, 尝试次数={}",
                    nonce,
                    work,
                    nonce - start_nonce + 1
                );

                // 创建并返回seal
                let seal = pow_utils::create_seal(nonce, work);
                return Ok(Some(seal));
            }

            // 每1000次迭代检查一次是否应该停止
            if nonce % 1000 == 0 {
                // 这里可以添加停止条件检查
            }
        }

        // 这一轮没有找到解
        #[cfg(feature = "std")]
        log::debug!("⏰ 挖矿轮次 {} 完成，未找到解", round);
        Ok(None)
    }
}

impl Default for BitNicePowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::testing::{Block as TestBlock, Header as TestHeader};

    #[test]
    fn test_pow_engine_creation() {
        let engine = BitNicePowEngine::new();
        
        // 测试初始难度
        let parent = BlockId::Hash(Default::default());
        let difficulty = engine.difficulty(&parent).unwrap();
        assert_eq!(difficulty, BitNicePowAlgorithm::initial_difficulty());
    }

    #[test]
    fn test_pow_verification() {
        let engine = BitNicePowEngine::new();
        let pre_hash = H256::random();
        let difficulty = U256::from(1u64); // 很低的难度，容易通过

        // 挖矿找到一个解
        let parent = BlockId::Hash(Default::default());
        if let Ok(Some(seal)) = engine.mine(&parent, &pre_hash, difficulty, 0) {
            // 验证这个解
            let is_valid = engine.verify(&parent, &pre_hash, &seal, difficulty).unwrap();
            assert!(is_valid);
        }
    }
} 
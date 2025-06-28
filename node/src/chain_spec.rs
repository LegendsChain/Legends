//! BitNice 链规范配置
//!
//! 极简版本 - 只提供基本的开发链配置

use sc_service::{ChainType, GenericChainSpec};

/// BitNice 链规范类型 - 使用Option<()>因为()没有实现ChainSpecExtension
pub type ChainSpec = GenericChainSpec<Option<()>>;

/// 开发配置
pub fn development_config() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        bitnice_runtime::WASM_BINARY.ok_or("WASM二进制文件未找到")?,
        None::<()>  // 使用None而不是空的扩展
    )
    .with_name("BitNice Development")
    .with_id("bitnice_dev")
    .with_chain_type(ChainType::Development)
    .with_genesis_config_patch(serde_json::json!({}))
    .build())
}

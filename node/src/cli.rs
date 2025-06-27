//! # BitNice 节点 CLI 定义
//! 
//! 定义命令行参数和子命令

use clap::Parser;
use sc_cli::RuntimeVersion;

/// BitNice 节点 CLI 参数
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[command(flatten)]
    pub run: sc_cli::RunCmd,
}

/// 可用的子命令
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// 密钥管理工具
    Key(sc_cli::KeySubcommand),

    /// 验证区块
    CheckBlock(sc_cli::CheckBlockCmd),

    /// 导出区块
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// 导出状态
    ExportState(sc_cli::ExportStateCmd),

    /// 导入区块
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// 清除链数据
    PurgeChain(sc_cli::PurgeChainCmd),

    /// 链规范重新生成
    Revert(sc_cli::RevertCmd),

    /// 基准测试 (暂时禁用)
    // #[command(name = "benchmark", about = "Benchmark runtime pallets.")]
    // Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// 尝试运行时工具
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// 构建链规范
    BuildSpec(sc_cli::BuildSpecCmd),
}

/// 运行时版本信息
pub fn load_spec(_: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(Box::new(crate::chain_spec::development_config()))
}

/// 获取运行时版本
pub fn native_runtime_version() -> &'static RuntimeVersion {
    &legends_runtime::VERSION
} 
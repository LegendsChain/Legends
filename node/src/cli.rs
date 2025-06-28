//! BitNice 节点命令行界面定义
//!
//! 定义了所有可用的命令行参数和子命令

use clap::Parser;
use sc_cli::RunCmd;

/// BitNice 节点 CLI 参数解析器
#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// 子命令
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// 运行命令参数
    #[command(flatten)]
    pub run: RunCmd,
}

/// 可用的子命令列表
#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// 密钥管理相关命令
    Key(sc_cli::KeySubcommand),

    /// 构建链规范文件
    BuildSpec(sc_cli::BuildSpecCmd),

    /// 验证区块有效性
    CheckBlock(sc_cli::CheckBlockCmd),

    /// 导出区块数据
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// 导出链状态
    ExportState(sc_cli::ExportStateCmd),

    /// 从文件导入区块
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// 清除链数据库
    PurgeChain(sc_cli::PurgeChainCmd),

    /// 回滚到指定区块
    Revert(sc_cli::RevertCmd),

    /// 运行基准测试
    #[cfg(feature = "runtime-benchmarks")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// 尝试运行时命令
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// 工作量证明挖矿命令
    Mine(MineCmd),
}

/// 挖矿命令参数
#[derive(Debug, clap::Parser)]
pub struct MineCmd {
    /// 挖矿线程数量
    #[arg(long, default_value = "1")]
    pub threads: usize,

    /// 挖矿奖励接收地址
    #[arg(long)]
    pub coinbase: Option<String>,

    /// 挖矿算法难度目标
    #[arg(long)]
    pub target: Option<u64>,

    /// 启用详细日志
    #[arg(long)]
    pub verbose: bool,

    /// 基础运行参数
    #[command(flatten)]
    pub base: RunCmd,
}

impl MineCmd {
    /// 获取挖矿线程数
    pub fn threads(&self) -> usize {
        if self.threads == 0 {
            num_cpus::get()
        } else {
            self.threads
        }
    }

    /// 获取挖矿奖励地址
    pub fn coinbase_address(&self) -> Option<&str> {
        self.coinbase.as_deref()
    }

    /// 获取难度目标
    pub fn difficulty_target(&self) -> Option<u64> {
        self.target
    }

    /// 是否启用详细日志
    pub fn is_verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_cli_parsing() {
        // 测试基本命令解析
        let cli = Cli::try_parse_from(&["bitnice-node", "--dev"]).unwrap();
        assert!(cli.run.is_dev());
    }

    #[test]
    fn test_mine_command_parsing() {
        // 测试挖矿命令解析
        let args = vec![
            "bitnice-node",
            "mine",
            "--threads",
            "4",
            "--coinbase",
            "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY",
            "--verbose",
        ];

        let cli = Cli::try_parse_from(&args).unwrap();

        if let Some(Subcommand::Mine(mine_cmd)) = cli.subcommand {
            assert_eq!(mine_cmd.threads(), 4);
            assert!(mine_cmd.coinbase_address().is_some());
            assert!(mine_cmd.is_verbose());
        } else {
            panic!("Expected Mine subcommand");
        }
    }

    #[test]
    fn test_default_threads() {
        let mine_cmd = MineCmd {
            threads: 0,
            coinbase: None,
            target: None,
            verbose: false,
            base: RunCmd::parse_from(&["test"]),
        };

        // 当 threads 为 0 时，应该返回 CPU 核心数
        assert!(mine_cmd.threads() > 0);
    }
}

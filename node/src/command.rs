//! BitNice 节点命令处理模块
//!
//! 处理所有命令行子命令的执行逻辑

use std::sync::Arc;

use crate::{
    chain_spec,
    cli::{Cli, Subcommand},
    service::{self, PartialComponents},
};
use bitnice_runtime::{Block, RuntimeApi};
use sc_cli::{
    ChainSpec, CheckBlockCmd, ExportBlocksCmd, ExportStateCmd, ImportBlocksCmd, PurgeChainCmd,
    RevertCmd, Role, RuntimeVersion, SubstrateCli,
};
use sc_service::{config::PrometheusConfig, Configuration, TaskManager};
use sp_core::crypto::Ss58AddressFormat;

/// 运行命令处理器
pub async fn run() -> sc_cli::Result<()> {
    let cli = Cli::parse();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => {
            // 处理密钥管理命令
            cmd.run(&cli)
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            // 处理构建链规范命令
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            // 处理检查区块命令
            construct_async_run!(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            // 处理导出区块命令
            construct_async_run!(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            // 处理导出状态命令
            construct_async_run!(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            // 处理导入区块命令
            construct_async_run!(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            // 处理清除链数据命令
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            // 处理回滚区块命令
            construct_async_run!(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = service::new_partial(&config)?;
                let aux_revert = Box::new(|client, _, blocks| {
                    sc_consensus_pow::revert(client, backend, blocks)
                });
                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            // 处理基准测试命令
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| {
                // 检查是否启用了基准测试功能
                if cfg!(feature = "runtime-benchmarks") {
                    return cmd.run::<bitnice_runtime::Hash, ()>(config);
                }

                Err("基准测试功能未启用。使用 --features=runtime-benchmarks 重新编译。".into())
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            // 处理尝试运行时命令
            use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let registry = config
                    .prometheus_config
                    .as_ref()
                    .map(|cfg| &cfg.registry);
                let task_manager =
                    sc_service::TaskManager::new(config.tokio_handle.clone(), registry)?;
                let info_provider = timestamp_with_aura_info(6000);
                Ok((
                    cmd.run::<Block, ExtendedHostFunctions<
                        sp_io::SubstrateHostFunctions,
                        <service::ExecutorDispatch as NativeExecutionDispatch>::ExtendHostFunctions,
                    >, _>(Some(info_provider)),
                    task_manager,
                ))
            })
        }
        Some(Subcommand::Mine(cmd)) => {
            // 处理挖矿命令
            run_mining_command(cmd, &cli).await
        }
        None => {
            // 运行完整节点
            let runner = cli.create_runner(&cli.run)?;
            runner
                .run_node_until_exit(|config| async move {
                    service::new_full(config).map_err(sc_cli::Error::Service)
                })
                .await
        }
    }
}

/// 运行挖矿命令
async fn run_mining_command(
    cmd: &crate::cli::MineCmd,
    cli: &Cli,
) -> sc_cli::Result<()> {
    use tracing::{info, warn};

    info!("🚀 启动 BitNice 挖矿节点");
    info!("⚡ 挖矿线程数: {}", cmd.threads());

    if let Some(coinbase) = cmd.coinbase_address() {
        info!("💰 奖励地址: {}", coinbase);
    } else {
        warn!("⚠️  未指定奖励地址，将使用默认地址");
    }

    if let Some(target) = cmd.difficulty_target() {
        info!("🎯 难度目标: {}", target);
    }

    if cmd.is_verbose() {
        info!("📝 启用详细日志模式");
    }

    // 创建运行器并启动挖矿节点
    let runner = cli.create_runner(&cmd.base)?;
    runner
        .run_node_until_exit(|mut config| async move {
            // 确保节点以验证者身份运行（挖矿需要）
            config.role = Role::Authority;

            // 启动完整节点服务，包含挖矿功能
            service::new_full(config).map_err(sc_cli::Error::Service)
        })
        .await
}

/// 时间戳信息提供者（用于 try-runtime）
#[cfg(feature = "try-runtime")]
fn timestamp_with_aura_info(
    millis: u64,
) -> impl Fn() -> Box<dyn sp_inherents::InherentDataProvider> {
    move || {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        Box::new(timestamp)
    }
}

/// 构造异步运行宏
macro_rules! construct_async_run {
    (|$config:ident| $task:expr) => {{
        let runner = cli.create_runner(cmd)?;
        runner.async_run(|$config| $task).await
    }};
}

use construct_async_run;

/// 获取原生运行时版本
pub fn get_native_runtime_version() -> &'static RuntimeVersion {
    &bitnice_runtime::VERSION
}

/// 链规范加载器
pub struct ChainSpecLoader;

impl ChainSpecLoader {
    /// 加载链规范
    pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_config()?),
            "" | "local" => Box::new(chain_spec::local_testnet_config()?),
            "staging" => Box::new(chain_spec::staging_testnet_config()?),
            path => Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    /// 获取所有可用的链规范 ID
    pub fn available_chains() -> Vec<&'static str> {
        vec!["dev", "local", "staging"]
    }
}

/// 验证配置
pub fn validate_config(config: &Configuration) -> Result<(), String> {
    // 验证数据库配置
    match &config.database {
        sc_client_db::DatabaseSource::RocksDb { path, .. } => {
            if !path.exists() {
                std::fs::create_dir_all(path)
                    .map_err(|e| format!("无法创建数据库目录: {}", e))?;
            }
        }
        _ => {}
    }

    // 验证网络配置
    if config.network.listen_addresses.is_empty() {
        return Err("至少需要一个监听地址".to_string());
    }

    // 验证 RPC 配置
    if config.rpc_methods == sc_service::config::RpcMethods::Unsafe {
        tracing::warn!("⚠️  启用了不安全的 RPC 方法，仅用于开发环境");
    }

    Ok(())
}

/// 命令执行结果
#[derive(Debug)]
pub enum CommandResult {
    /// 成功完成
    Success,
    /// 带有消息的成功
    SuccessWithMessage(String),
    /// 错误
    Error(String),
}

impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandResult::Success => write!(f, "命令执行成功"),
            CommandResult::SuccessWithMessage(msg) => write!(f, "命令执行成功: {}", msg),
            CommandResult::Error(err) => write!(f, "命令执行失败: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_spec_loader() {
        // 测试开发环境链规范加载
        let spec = ChainSpecLoader::load_spec("dev");
        assert!(spec.is_ok());

        // 测试本地测试网链规范加载
        let spec = ChainSpecLoader::load_spec("local");
        assert!(spec.is_ok());

        // 测试预发布测试网链规范加载
        let spec = ChainSpecLoader::load_spec("staging");
        assert!(spec.is_ok());
    }

    #[test]
    fn test_available_chains() {
        let chains = ChainSpecLoader::available_chains();
        assert!(chains.contains(&"dev"));
        assert!(chains.contains(&"local"));
        assert!(chains.contains(&"staging"));
    }

    #[test]
    fn test_command_result_display() {
        let success = CommandResult::Success;
        assert_eq!(success.to_string(), "命令执行成功");

        let success_with_msg = CommandResult::SuccessWithMessage("测试消息".to_string());
        assert_eq!(success_with_msg.to_string(), "命令执行成功: 测试消息");

        let error = CommandResult::Error("测试错误".to_string());
        assert_eq!(error.to_string(), "命令执行失败: 测试错误");
    }

    #[test]
    fn test_native_runtime_version() {
        let version = get_native_runtime_version();
        assert_eq!(version.spec_name.as_ref(), "bitnice");
        assert_eq!(version.impl_name.as_ref(), "bitnice");
    }
}

//! BitNice 区块链节点
//!
//! 极简版本 - 专注于基本的节点启动功能

use sc_service::Role;
use sc_cli::{CliConfiguration, DefaultConfigurationValues, ImportParams, SharedParams, SubstrateCli};
use clap::Parser;

mod chain_spec;
// TODO: 等API稳定后重新启用RPC
// mod rpc;
mod service;

/// BitNice 命令行参数
#[derive(Debug, clap::Parser)]
pub struct Cli {
    /// 共享参数
    #[clap(flatten)]
    pub shared_params: SharedParams,

    /// 导入参数
    #[clap(flatten)]
    pub import_params: ImportParams,

    /// 开发模式
    #[clap(long)]
    pub dev: bool,
}

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "BitNice Node".into()
    }

    fn impl_version() -> String {
        "0.1.0".into()
    }

    fn description() -> String {
        "BitNice 区块链节点 - 采用SHA-256 PoW共识算法".into()
    }

    fn author() -> String {
        "health <health@example.com>".into()
    }

    fn support_url() -> String {
        "https://github.com/your-org/bitnice".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        match id {
            "dev" | "" => Ok(Box::new(chain_spec::development_config()?)),
            path => Ok(Box::new(chain_spec::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?)),
        }
    }
}

/// 默认运行时执行器
pub struct Executor;

impl sc_executor::NativeExecutionDispatch for Executor {
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        bitnice_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        bitnice_runtime::native_version()
    }
}

impl DefaultConfigurationValues for Cli {
    fn p2p_listen_port() -> u16 {
        30333
    }

    fn rpc_listen_port() -> u16 {
        9933
    }

    fn prometheus_listen_port() -> u16 {
        9615
    }
}

impl CliConfiguration<()> for Cli {
    fn chain_id(&self, _is_dev: bool) -> Result<String, sc_cli::Error> {
        Ok("bitnice".to_string())
    }

    fn role(&self, _is_dev: bool) -> Result<Role, sc_cli::Error> {
        Ok(Role::Full)
    }

    fn shared_params(&self) -> &SharedParams {
        &self.shared_params
    }

    fn import_params(&self) -> Option<&ImportParams> {
        Some(&self.import_params)
    }
}

// 使用 tokio::main 来运行async代码
#[tokio::main]
async fn main() -> sc_cli::Result<()> {
    // 解析命令行参数
    let cli = Cli::parse();

    // 创建配置
    let runner = SubstrateCli::create_configuration(&cli, &cli, tokio::runtime::Handle::current())?;
    
    // 创建服务并运行
    let mut partial = service::new_partial(&runner)?;
    
    // 等待任务管理器结束
    partial.task_manager.future().await?;

    Ok(())
}

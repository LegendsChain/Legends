//! # BitNice 节点命令处理
//! 
//! 处理各种 CLI 命令的执行逻辑

use crate::{
    chain_spec,
    cli::{Cli, Subcommand},
    service,
};
use legends_runtime::{Block, RuntimeApi};
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "BitNice Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/health/Legends/issues".into()
    }

    fn copyright_start_year() -> i32 {
        2024
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::development_config()),
            "local" => Box::new(chain_spec::local_testnet_config()),
            "" | "bitnice" => Box::new(chain_spec::bitnice_config()),
            path => Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?),
        })
    }
}

/// 解析并运行命令
pub fn run() -> sc_cli::Result<()> {
    let cli = Cli::parse();

    match &cli.subcommand {
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, backend, .. } =
                    service::new_partial(&config)?;
                Ok((cmd.run(client, backend, None), task_manager))
            })
        }
        // 暂时禁用基准测试功能以避免依赖冲突
        // Some(Subcommand::Benchmark(cmd)) => {
        //     let runner = cli.create_runner(cmd)?;
        //     // ... 基准测试代码已注释掉
        // }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            use sc_executor::{sp_wasm_interface::ExtendedHostFunctions, NativeExecutionDispatch};
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                let task_manager =
                    sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                        .map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;
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
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config).map_err(sc_cli::Error::Service)
            })
        }
    }
}

fn inherent_benchmark_data() -> Result<sp_inherents::InherentData, sc_cli::Error> {
    use sp_inherents::InherentDataProvider;
    let mut inherent_data = sp_inherents::InherentData::new();

    let d = std::time::Duration::from_millis(0);
    let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

    futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
        .map_err(|e| format!("providing inherent data: {:?}", e))?;

    Ok(inherent_data)
}

// 暂时注释掉基准测试相关的导入和结构体
// use frame_benchmarking_cli::{
//     extrinsic_factory::ExtrinsicFactory, BenchmarkExtrinsicBuilder, ExtrinsicBuilder,
// };

// /// Generates `Balances::TransferKeepAlive` extrinsics for the benchmarks.
// struct TransferKeepAliveBuilder;

// impl BenchmarkExtrinsicBuilder for TransferKeepAliveBuilder {
//     fn pallet(&self) -> &str {
//         "balances"
//     }

//     fn extrinsic(&self) -> &str {
//         "transfer_keep_alive"
//     }

//     fn build(&self, nonce: u32) -> std::result::Result<legends_runtime::UncheckedExtrinsic, &'static str> {
//         let acc = sp_keyring::AccountKeyring::Bob.pair();
//         let extrinsic: legends_runtime::UncheckedExtrinsic = pallet_balances::Call::transfer_keep_alive {
//             dest: sp_keyring::AccountKeyring::Charlie.to_account_id().into(),
//             value: 1,
//         }
//         .into();

//         Ok(extrinsic)
//     }
// }

// /// Generates `System::Remark` extrinsics for the benchmarks.
// struct RemarkBuilder {
//     client: std::sync::Arc<service::FullClient>,
// }

// impl RemarkBuilder {
//     /// Creates a new [`Self`] from the given client.
//     fn new(client: std::sync::Arc<service::FullClient>) -> Self {
//         Self { client }
//     }
// }

// impl ExtrinsicBuilder for RemarkBuilder {
//     fn pallet(&self) -> &str {
//         "system"
//     }

//     fn extrinsic(&self) -> &str {
//         "remark"
//     }

//     fn build(&self, nonce: u32) -> std::result::Result<legends_runtime::UncheckedExtrinsic, &'static str> {
//         let acc = sp_keyring::AccountKeyring::Bob.pair();
//         let extrinsic: legends_runtime::UncheckedExtrinsic = frame_system::Call::remark { remark: vec![] }.into();

//         Ok(extrinsic)
//     }
// }

#[cfg(feature = "try-runtime")]
fn timestamp_with_aura_info(slot_duration: u64) -> impl try_runtime_cli::TryRuntimeInfoProvider {
    try_runtime_cli::substrate_info(slot_duration)
} 
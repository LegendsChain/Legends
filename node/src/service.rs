//! # BitNice 服务模块
//! 
//! 核心节点服务，集成 PoW 共识算法

use std::{sync::Arc, time::Duration};

use futures::prelude::*;
use legends_runtime::{opaque::Block, RuntimeApi, BitcoinPowAlgorithm};
use sc_client_api::Backend;
use sc_consensus_pow::{PowBlockImport, PowVerifier};
use sc_executor::{HeapAllocStrategy, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY};
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus::SlotData;
use sp_inherents::CreateInherentDataProviders;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};

// 运行时 executor 类型别名
pub type ExecutorDispatch = sc_executor::NativeElseWasmExecutor<ExecutorDispatch>;

/// 原生执行器实例
pub struct ExecutorDispatch;

impl sc_executor::NativeExecutionDispatch for ExecutorDispatch {
    /// 只有在运行时版本匹配时才启用原生
    type ExtendHostFunctions = polkadot_sdk::frame_benchmarking::benchmarking::HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        legends_runtime::RuntimeApiDispatch::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        legends_runtime::VERSION.into()
    }
}

/// 完整客户端类型别名
pub type FullClient =
    sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<ExecutorDispatch>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// 创建 PoW 内在数据提供者
pub fn pow_inherent_data_providers() -> impl CreateInherentDataProviders<Block, ()> {
    move |_, ()| async move {
        let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
        Ok(timestamp)
    }
}

/// 部分组件结构
pub type PartialComponents = sc_service::PartialComponents<
    FullClient,
    FullBackend,
    FullSelectChain,
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, FullClient>,
    (
        sc_consensus_pow::PowBlockImport<
            Block,
            Arc<FullClient>,
            FullClient,
            FullSelectChain,
            BitcoinPowAlgorithm,
            impl CreateInherentDataProviders<Block, ()>,
        >,
        Option<Telemetry>,
    ),
>;

/// 创建部分组件
pub fn new_partial(config: &Configuration) -> Result<PartialComponents, ServiceError> {
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = WasmExecutor::<ExecutorDispatch>::builder()
        .with_execution_method(config.wasm_method)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size)
        .build();

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let algorithm = BitcoinPowAlgorithm;
    let inherent_data_providers = pow_inherent_data_providers();

    let pow_block_import = sc_consensus_pow::PowBlockImport::new(
        client.clone(),
        client.clone(),
        algorithm,
        0, // 检查继承的数量
        select_chain.clone(),
        inherent_data_providers,
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
    );

    let import_queue = sc_consensus_pow::import_queue(
        Box::new(pow_block_import.clone()),
        None,
        algorithm,
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
    )?;

    Ok(PartialComponents {
        client,
        backend,
        task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (pow_block_import, telemetry),
    })
}

/// 构建完整服务
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (pow_block_import, mut telemetry),
    } = new_partial(&config)?;

    let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

    let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
        &client.block_hash(0)?.expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(
        grandpa_protocol_name.clone(),
    ));

    let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        import_queue.clone(),
        None,
    ));

    let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: Some(sc_service::WarpSyncParams::WithProvider(warp_sync)),
            block_relay: None,
            metrics: None,
        })?;

    if config.offchain_worker.enabled {
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-worker",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                is_validator: config.role.is_authority(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: network.clone(),
                enable_http_requests: true,
                custom_extensions: |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks: Option<()> = None;
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
            };
            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        network: network.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        task_manager: &mut task_manager,
        transaction_pool: transaction_pool.clone(),
        rpc_builder: rpc_extensions_builder,
        backend,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        config,
        telemetry: telemetry.as_mut(),
    })?;

    if role.is_authority() {
        let proposer_factory = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let algorithm = BitcoinPowAlgorithm;
        let inherent_data_providers = pow_inherent_data_providers();

        let authorship_future = sc_consensus_pow::start_mining_worker(
            Box::new(pow_block_import),
            client,
            select_chain,
            algorithm,
            proposer_factory,
            sync_service.clone(),
            Some(sync_service),
            inherent_data_providers,
            Duration::from_secs(6), // 6秒区块时间
            // When the time between referencing is > the block time,
            // we should start mining immediately.
            Duration::from_secs(6),
        );

        task_manager.spawn_essential_handle().spawn_blocking(
            "pow-proposer",
            Some("block-authoring"),
            authorship_future,
        );
    }

    // 启动 GRANDPA 投票者
    if enable_grandpa {
        let grandpa_config = sc_consensus_grandpa::Config {
            gossip_duration: Duration::from_millis(333),
            justification_generation_period: 512,
            name: Some(name),
            observer_enabled: false,
            keystore: Some(keystore_container.keystore()),
            local_role: role,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            protocol_name: grandpa_protocol_name,
        };

        let grandpa_voters_future = sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?;
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            grandpa_voters_future,
        );
    }

    network_starter.start_network();
    Ok(task_manager)
}

// 离线交易池工厂
pub struct OffchainTransactionPoolFactory<TP> {
    pool: Arc<TP>,
}

impl<TP> OffchainTransactionPoolFactory<TP> {
    pub fn new(pool: Arc<TP>) -> Self {
        Self { pool }
    }
}

impl<TP> sc_offchain::OffchainTransactionPool for OffchainTransactionPoolFactory<TP>
where
    TP: sc_transaction_pool_api::SubmitTransactionError + Send + Sync + 'static,
{
    fn submit_transaction(&self, extrinsic: <Block as BlockT>::Extrinsic) -> Result<(), String> {
        use sc_transaction_pool_api::SubmitTransaction;
        self.pool
            .submit_one(&futures::executor::block_on(
                self.pool
                    .validated_pool()
                    .api()
                    .validate_transaction(sp_runtime::generic::BlockId::Number(0), 
                        sp_runtime::transaction_validity::TransactionSource::External, 
                        extrinsic)
            ).map_err(|e| format!("{:?}", e))?.ok_or("Validation failed")?
            .0, 
            sp_runtime::transaction_validity::TransactionSource::External)
            .map_err(|e| format!("{:?}", e))
    }
} 
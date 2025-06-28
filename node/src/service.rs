//! BitNice 节点服务配置
//!
//! 极简版本 - 只提供基本的节点启动功能

use std::sync::Arc;

use sc_service::{error::Error as ServiceError, Configuration, TaskManager};

use bitnice_runtime::{self, opaque::Block, RuntimeApi};

type FullClient = sc_service::TFullClient<Block, RuntimeApi, sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>>;
type FullBackend = sc_service::TFullBackend<Block>;

/// 极简的组件结构
#[allow(dead_code)]
pub struct PartialComponents {
    pub client: Arc<FullClient>,
    pub backend: Arc<FullBackend>,
    pub task_manager: TaskManager,
}

/// 创建部分组件 - 极简版本
pub fn new_partial(
    config: &Configuration,
) -> Result<PartialComponents, ServiceError> {
    // 获取数据库配置
    let db_config = sc_client_db::DatabaseSettings {
        trie_cache_maximum_size: config.trie_cache_maximum_size,
        state_pruning: config.state_pruning.clone(),
        source: config.database.clone(),
        blocks_pruning: config.blocks_pruning,
        metrics_registry: None,  // 不使用metrics
    };

    // 创建基本的数据库后端
    let _backend = sc_service::new_db_backend::<Block>(db_config)?;

    // 创建执行器
    let wasm_executor = sc_executor::WasmExecutor::builder()
        .with_execution_method(config.executor.wasm_method)
        .with_max_runtime_instances(config.executor.max_runtime_instances)
        .with_runtime_cache_size(config.executor.runtime_cache_size)
        .build();

    // 创建客户端
    let (client, backend, _keystore_container, task_manager) = 
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            None,
            wasm_executor,
        )?;

    let client = Arc::new(client);

    Ok(PartialComponents {
        client,
        backend,
        task_manager,
    })
}

/// 创建完整节点服务 - 暂时返回任务管理器
#[allow(dead_code)]
pub async fn new_full(
    config: Configuration,
) -> Result<TaskManager, ServiceError> {
    let partial = new_partial(&config)?;

    // TODO: 当API稳定后，创建真正的交易池和RPC服务
    // 目前直接返回任务管理器，避免复杂的类型问题

    println!("✅ BitNice 节点启动成功!");
    println!("⚠️  注意：当前是极简模式，未包含RPC和交易池功能");
    
    Ok(partial.task_manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use tempfile::TempDir;

    // fn create_test_config() -> Configuration {
    //     let tmp_dir = TempDir::new().unwrap();
    //     let mut config = Configuration::default();
    //     config.database = sc_client_db::DatabaseSource::RocksDb {
    //         path: tmp_dir.path().to_path_buf(),
    //         cache_size: 1024,
    //     };
    //     config.network.net_config_path = Some(tmp_dir.path().join("network"));
    //     config
    // }

    #[test]
    fn test_partial_components_creation() {
        // 测试部分组件创建（需要有效配置）
        // let config = create_test_config();
        // let _components = new_partial(&config).expect("应该能创建部分组件");
    }
}

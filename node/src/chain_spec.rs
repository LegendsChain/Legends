//! # BitNice 链规范定义
//! 
//! 定义不同环境下的链规范配置

use legends_runtime::{
    AccountId, Balance, RuntimeGenesisConfig, Signature, WASM_BINARY,
};
use polkadot_sdk::{
    pallet_balances::BalancesConfig,
    pallet_sudo::SudoConfig,
    frame_system::GenesisConfig as SystemConfig,
};
use sc_service::ChainType;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// 节点命令行的链规范扩展
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// 从种子生成加密对
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// 从种子生成账户 ID
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// 从种子生成授权账户 (用于验证者)
pub fn authority_keys_from_seed(s: &str) -> GrandpaId {
    get_from_seed::<GrandpaId>(s)
}

/// 开发配置 (单节点)
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // 链名称
        "BitNice Development",
        // 链 ID
        "bitnice_dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // 初始验证者 (GRANDPA)
                vec![authority_keys_from_seed("Alice")],
                // Sudo 账户
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // 预充值账户
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        Some(bitnice_properties()),
        // Extensions
        None,
    ))
}

/// 本地测试网配置
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // 链名称
        "BitNice Local Testnet",
        // 链 ID
        "bitnice_local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // 初始验证者
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo 账户
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // 预充值账户
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        None,
        // Properties
        Some(bitnice_properties()),
        // Extensions
        None,
    ))
}

/// BitNice 主网配置
pub fn bitnice_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/bitnice.json")[..])
}

/// BitNice 链属性
fn bitnice_properties() -> sc_chain_spec::Properties {
    use sc_chain_spec::Properties;
    use serde_json::map::Map;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "BN".into());
    properties.insert("tokenDecimals".into(), 18.into());
    properties.insert("ss58Format".into(), 42.into());
    Properties::from(properties)
}

/// 配置初始创世状态
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<GrandpaId>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> RuntimeGenesisConfig {
    const ENDOWMENT: Balance = 10_000_000 * 1_000_000_000_000_000_000u128; // 1000万 BN
    const TOTAL_SUPPLY: Balance = 210_000_000 * 1_000_000_000_000_000_000u128; // 2.1亿 BN 总量

    RuntimeGenesisConfig {
        system: SystemConfig {
            // 设置 Wasm runtime 代码
            code: wasm_binary.to_vec(),
            ..Default::default()
        },
        balances: BalancesConfig {
            // 配置账户余额
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, ENDOWMENT))
                .collect(),
        },
        sudo: SudoConfig {
            // 分配 sudo 密钥给 root
            key: Some(root_key),
        },
        transaction_payment: Default::default(),
        grandpa: pallet_grandpa::GenesisConfig {
            authorities: initial_authorities.iter().map(|x| (x.clone(), 1)).collect(),
            ..Default::default()
        },
    }
} 
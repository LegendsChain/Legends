# BitNice 区块链 🚀

基于最新 Polkadot SDK 构建的 PoW 共识区块链

## 📋 项目信息

- **代币名称**: BitNice
- **代币符号**: BN  
- **代币总量**: 2.1 亿 BN
- **区块时间**: 6 秒
- **共识算法**: PoW (类似比特币的 SHA-256)
- **最终性**: GRANDPA
- **作者**: BitNice
- **公司**: BitNice科技有限公司

## 🔧 技术栈

- **框架**: Polkadot SDK (最新版本)
- **语言**: Rust
- **共识**: PoW + GRANDPA 混合共识
- **虚拟机**: Wasm

## 📁 项目结构

```
Legends/
├── Cargo.toml              # 工作空间配置
├── node/                   # 节点实现
│   ├── src/
│   │   ├── main.rs         # 节点入口
│   │   ├── cli.rs          # 命令行接口
│   │   ├── command.rs      # 命令处理
│   │   ├── service.rs      # 核心服务 + PoW 共识
│   │   ├── chain_spec.rs   # 链规范配置
│   │   └── rpc.rs          # RPC 接口
│   ├── Cargo.toml
│   └── build.rs
├── runtime/                # 运行时
│   ├── src/
│   │   └── lib.rs          # 运行时实现
│   ├── Cargo.toml
│   └── build.rs
└── pallets/                # 自定义 Pallets
    └── template/           # 模板 Pallet
        ├── src/
        │   └── lib.rs
        └── Cargo.toml
```

## ✨ 主要功能

### 🔗 核心功能
- ✅ PoW 挖矿算法 (SHA-256)
- ✅ GRANDPA 最终性共识
- ✅ 账户和余额管理
- ✅ 交易费用系统
- ✅ Sudo 管理权限
- ✅ 实用工具 Pallet

### 🎯 PoW 特性
- **挖矿算法**: 基于比特币的 SHA-256 PoW 算法
- **难度调整**: 动态难度调整保持 6 秒区块时间
- **算力激励**: 支持矿工激励机制
- **去中心化**: 完全去中心化的挖矿网络

### 💰 代币经济
- **总量**: 210,000,000 BN (2.1 亿)
- **精度**: 18 位小数
- **分配**: 通过 PoW 挖矿产出

## 🚀 快速开始

### 依赖要求

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加 Wasm 目标
rustup target add wasm32-unknown-unknown
```

### 编译节点

```bash
# 克隆项目
git clone https://github.com/health/Legends.git
cd Legends

# 编译
cargo build --release
```

### 启动开发网络

```bash
# 启动单节点开发链
./target/release/legends-node --dev

# 或启动本地测试网络
./target/release/legends-node --chain=local
```

### 挖矿

```bash
# 启动挖矿节点
./target/release/legends-node --validator --mining
```

## 🔧 配置选项

### 网络配置

```bash
# 开发网络
--chain=dev

# 本地测试网络  
--chain=local

# 主网
--chain=bitnice
```

### 挖矿配置

```bash
# 启用挖矿
--mining

# 设置挖矿线程数
--mining-threads=4

# 设置验证者
--validator
```

## 📖 技术文档

### PoW 算法实现

BitNice 使用类似比特币的 SHA-256 PoW 算法：

```rust
impl PowAlgorithm<H256> for BitcoinPowAlgorithm {
    type Difficulty = U256;
    
    fn verify(&self, pre_hash: &H256, seal: &Seal, difficulty: Self::Difficulty) -> Result<bool, Error> {
        let hash = Sha256::digest(&[pre_hash.as_bytes(), seal].concat());
        let hash_value = U256::from_big_endian(hash.as_slice());
        Ok(hash_value < difficulty)
    }
}
```

### 混合共识

- **PoW**: 负责区块生产和工作量证明
- **GRANDPA**: 负责区块最终性确认

## 🛠 开发指南

### 添加新 Pallet

1. 在 `pallets/` 目录创建新 pallet
2. 在 `runtime/src/lib.rs` 中添加配置
3. 更新 `Cargo.toml` 依赖

### 自定义 PoW 算法

修改 `runtime/src/lib.rs` 中的 `BitcoinPowAlgorithm` 实现。

### RPC 接口扩展

在 `node/src/rpc.rs` 中添加新的 RPC 方法。

## 🔍 监控和调试

### 日志级别

```bash
# 调试模式
RUST_LOG=debug ./target/release/legends-node --dev

# 特定模块日志
RUST_LOG=sc_consensus_pow=trace ./target/release/legends-node --dev
```

### Prometheus 监控

```bash
# 启用 Prometheus 监控
./target/release/legends-node --prometheus-external
```

## 🤝 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📞 联系方式

- **作者**: health
- **公司**: 广西红蚂蚁网络科技有限公司
- **邮箱**: 请通过 GitHub Issues 联系

## 📄 许可证

本项目采用 GPL-3.0 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🎉 致谢

- [Polkadot SDK](https://github.com/paritytech/polkadot-sdk) - 提供强大的区块链开发框架
- [Substrate](https://substrate.io/) - 模块化区块链框架
- Bitcoin - PoW 算法灵感来源

---

**BitNice** - 连接未来的去中心化价值网络 💎

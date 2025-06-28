# BitNice 区块链 - 基于 Polkadot SDK 的比特币风格 PoW 区块链

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](https://opensource.org/licenses/GPL-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Polkadot SDK](https://img.shields.io/badge/Polkadot_SDK-Latest-green.svg)](https://github.com/paritytech/polkadot-sdk)

## 📋 项目概述

BitNice 是一个基于 [Polkadot SDK](https://github.com/paritytech/polkadot-sdk) 构建的区块链项目，采用比特币风格的工作量证明（PoW）共识算法。该项目旨在提供一个安全、去中心化且可扩展的区块链解决方案。

### 🎯 项目特性

- **共识算法**: 比特币风格的 SHA-256 工作量证明（PoW）
- **区块时间**: 6 秒
- **代币名称**: BitNice (BN)
- **代币总量**: 21 亿 BN
- **技术栈**: Rust + Polkadot SDK
- **最终性**: GRANDPA 快速最终性
- **架构**: 模块化运行时设计

### 🏢 团队信息

- **作者**: health
- **公司**: 广西红蚂蚁网络科技有限公司
- **许可证**: GPL-3.0

## 🚀 快速开始

### 📋 环境要求

在开始之前，请确保您的系统满足以下要求：

#### 系统要求
- **操作系统**: Linux、macOS 或 Windows (推荐 Ubuntu 20.04+)
- **内存**: 最少 8GB RAM (推荐 16GB+)
- **存储**: 最少 50GB 可用空间
- **网络**: 稳定的互联网连接

#### 必需软件

1. **Rust 工具链** (1.75.0 或更高版本)
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

2. **WASM 编译目标**
```bash
# 安装 WebAssembly 编译目标
rustup target add wasm32-unknown-unknown
rustup target add wasm32v1-none

# 更新工具链
rustup update
```

3. **构建依赖** (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    git \
    clang \
    curl \
    libclang-dev \
    protobuf-compiler
```

4. **构建依赖** (macOS)
```bash
# 安装 Homebrew (如果未安装)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装依赖
brew install openssl pkg-config protobuf
```

### 📥 克隆和编译

1. **克隆仓库**
```bash
git clone https://github.com/your-org/bitnice.git
cd bitnice
```

2. **编译项目**
```bash
# 清理之前的构建缓存（如果有）
cargo clean

# 编译发布版本
cargo build --release

# 或编译调试版本（更快，但性能较低）
cargo build
```

3. **验证编译**
```bash
# 检查二进制文件
ls -la target/release/
```

编译成功后，您将在 `target/release/` 目录中找到以下重要文件：
- `node-template`: BitNice 节点可执行文件
- `bitnice-runtime.wasm`: 运行时 WebAssembly 模块

## 🔧 使用指南

### 启动节点

#### 1. 开发模式（单节点测试）
```bash
# 启动开发节点
./target/release/node-template --dev

# 或指定数据目录
./target/release/node-template --dev --base-path /tmp/bitnice-dev
```

#### 2. 生产模式
```bash
# 启动验证节点
./target/release/node-template \
    --validator \
    --name "BitNice-Node-1" \
    --chain local \
    --base-path ./data \
    --port 30333 \
    --rpc-port 9933 \
    --rpc-cors all \
    --rpc-methods unsafe
```

#### 3. 挖矿模式
```bash
# 启动挖矿节点
./target/release/node-template \
    --validator \
    --mining \
    --name "BitNice-Miner" \
    --chain local \
    --base-path ./miner-data
```

### 命令行参数详解

| 参数 | 描述 | 示例 |
|------|------|------|
| `--dev` | 开发模式，使用预定义账户 | `--dev` |
| `--validator` | 启用验证器模式 | `--validator` |
| `--mining` | 启用挖矿功能 | `--mining` |
| `--name` | 节点名称 | `--name "MyNode"` |
| `--chain` | 链规范 | `--chain local` |
| `--base-path` | 数据存储路径 | `--base-path ./data` |
| `--port` | P2P 网络端口 | `--port 30333` |
| `--rpc-port` | RPC 端口 | `--rpc-port 9933` |
| `--rpc-cors` | CORS 设置 | `--rpc-cors all` |

### 📊 监控和日志

#### 启用详细日志
```bash
# 启用所有模块的调试日志
RUST_LOG=debug ./target/release/node-template --dev

# 启用特定模块的日志
RUST_LOG=sc_consensus_pow=debug,sc_basic_authorship=debug ./target/release/node-template --dev

# 输出日志到文件
./target/release/node-template --dev 2>&1 | tee bitnice.log
```

#### 性能监控
```bash
# 启用性能追踪
RUST_LOG=trace ./target/release/node-template --dev --tracing-targets="sc_consensus_pow,pallet_balances"
```

## 🔌 API 使用

### RPC 接口

BitNice 节点提供标准的 Substrate RPC 接口：

- **HTTP RPC**: `http://localhost:9933`
- **WebSocket RPC**: `ws://localhost:9944`

#### 基本 RPC 调用示例

```bash
# 获取链信息
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain", "params":[]}' \
     http://localhost:9933

# 获取最新区块号
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getBlock", "params":[]}' \
     http://localhost:9933

# 获取账户余额
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_accountNonceApi", "params":["5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]}' \
     http://localhost:9933
```

### 使用 Polkadot.js 连接

1. 打开 [Polkadot.js Apps](https://polkadot.js.org/apps/)
2. 点击左上角网络选择器
3. 选择 "Development" -> "Local Node"
4. 确保 WebSocket 地址为 `ws://127.0.0.1:9944`

## 💰 钱包和交易

### 创建账户

```bash
# 使用 subkey 工具创建新账户
cargo install --force subkey --git https://github.com/paritytech/polkadot-sdk

# 生成新的 SR25519 密钥对
subkey generate --scheme sr25519

# 生成新的 Ed25519 密钥对  
subkey generate --scheme ed25519
```

### 转账示例

使用 Polkadot.js Apps 进行转账：

1. 进入 "Accounts" 页面
2. 点击 "Send" 按钮
3. 选择发送账户
4. 输入接收地址
5. 输入转账金额
6. 提交交易

## 🔧 开发指南

### 项目结构

```
bitnice/
├── Cargo.toml              # 工作空间配置
├── README.md               # 项目文档
├── node/                   # 节点实现
│   ├── Cargo.toml         # 节点依赖配置
│   ├── build.rs           # 构建脚本
│   └── src/
│       ├── main.rs        # 主入口
│       ├── cli.rs         # 命令行接口
│       ├── command.rs     # 命令处理
│       ├── service.rs     # 核心服务
│       ├── chain_spec.rs  # 链规范
│       └── rpc.rs         # RPC 配置
├── runtime/                # 运行时
│   ├── Cargo.toml         # 运行时依赖
│   ├── build.rs           # WASM 构建
│   └── src/
│       └── lib.rs         # 运行时实现
└── pallets/                # 自定义 Pallets
    └── template/          # 模板 Pallet
```

### 添加新的 Pallet

1. **创建 Pallet 结构**
```bash
mkdir -p pallets/my-pallet/src
```

2. **创建 Pallet Cargo.toml**
```toml
[package]
name = "pallet-my-pallet"
version = "0.1.0"
edition = "2021"

[dependencies]
codec = { workspace = true, default-features = false }
scale-info = { workspace = true, default-features = false }
frame-support = { workspace = true, default-features = false }
frame-system = { workspace = true, default-features = false }
sp-std = { workspace = true, default-features = false }

[features]
default = ["std"]
std = [
    "codec/std",
    "scale-info/std",
    "frame-support/std",
    "frame-system/std",
    "sp-std/std",
]
```

3. **实现 Pallet 逻辑**
```rust
// pallets/my-pallet/src/lib.rs
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::storage]
    pub(super) type Something<T: Config> = StorageValue<_, u32>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SomethingStored { something: u32, who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Something::<T>::put(something);
            Self::deposit_event(Event::SomethingStored { something, who });
            Ok(())
        }
    }
}
```

4. **集成到运行时**
```rust
// runtime/src/lib.rs 中添加
impl pallet_my_pallet::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
}

// 在 construct_runtime! 宏中添加
construct_runtime!(
    pub struct Runtime {
        // ... 其他 pallets
        MyPallet: pallet_my_pallet,
    }
);
```

### 自定义挖矿算法

如果需要修改挖矿算法，可以编辑 `runtime/src/lib.rs` 中的 PoW 配置：

```rust
// 自定义挖矿难度调整
impl sp_consensus_pow::DifficultyApi<Block, Difficulty> for Runtime {
    fn difficulty() -> Difficulty {
        // 自定义难度计算逻辑
        U256::from(1000000)
    }
}
```

### 测试

#### 单元测试
```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test -p pallet-my-pallet

# 运行运行时测试
cargo test -p bitnice-runtime
```

#### 集成测试
```bash
# 运行节点测试
cargo test -p node-template
```

#### 基准测试
```bash
# 运行基准测试
cargo test --features runtime-benchmarks
```

## 📈 性能优化

### 编译优化

1. **发布版本编译**
```bash
# 使用最高优化级别
CARGO_PROFILE_RELEASE_LTO=true cargo build --release
```

2. **针对目标 CPU 优化**
```bash
# 针对本机 CPU 优化
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

### 运行时优化

1. **调整缓存大小**
```bash
./target/release/node-template \
    --dev \
    --state-cache-size 1073741824  # 1GB 状态缓存
```

2. **调整数据库配置**
```bash
./target/release/node-template \
    --dev \
    --database rocksdb \
    --db-cache 512  # 512MB 数据库缓存
```

## 🐛 故障排除

### 常见问题

#### 1. 编译错误

**问题**: `error: could not find Cargo.toml`
```bash
# 解决方法：确保在项目根目录
cd /path/to/bitnice
cargo build --release
```

**问题**: `error: linking with cc failed`
```bash
# 解决方法：安装构建依赖
sudo apt install build-essential pkg-config libssl-dev
```

#### 2. 运行时错误

**问题**: `Error: Service(Other("Genesis block not found"))`
```bash
# 解决方法：清理数据目录重新初始化
rm -rf ./data
./target/release/node-template --dev
```

**问题**: `Error: Network("Transport error")`
```bash
# 解决方法：检查端口是否被占用
netstat -tuln | grep 30333
# 使用不同端口
./target/release/node-template --dev --port 30334
```

#### 3. PoW 挖矿问题

**问题**: 挖矿速度过慢
```bash
# 解决方法：降低挖矿难度或使用更多线程
RUST_LOG=debug ./target/release/node-template --dev --mining --threads 4
```

### 日志分析

#### 启用详细日志
```bash
# 启用所有 PoW 相关日志
RUST_LOG="sc_consensus_pow=debug,sc_basic_authorship=debug" ./target/release/node-template --dev
```

#### 常见日志模式
- `💤 Idle`: 节点空闲，等待交易
- `🙌 Starting consensus session`: 开始新的共识会话  
- `⛏️  Starting mining operation`: 开始挖矿操作
- `✨ Imported`: 成功导入新区块

### 性能监控

```bash
# 监控资源使用
htop

# 监控网络连接
ss -tuln | grep -E "(30333|9933|9944)"

# 监控磁盘使用
du -sh ./data
```

## 🤝 贡献指南

我们欢迎社区贡献！请遵循以下步骤：

### 1. 环境设置
```bash
# Fork 并克隆仓库
git clone https://github.com/your-username/bitnice.git
cd bitnice

# 创建开发分支
git checkout -b feature/my-feature
```

### 2. 代码规范

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 命名约定
- 添加充分的中文注释

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy -- -D warnings

# 运行测试
cargo test
```

### 3. 提交规范

提交信息使用中文，格式如下：
```
类型(范围): 简短描述

详细描述说明变更内容和原因

Closes #issue_number
```

示例：
```
功能(挖矿): 添加自适应难度调整算法

- 实现基于区块时间的难度自动调整
- 优化挖矿性能和稳定性
- 添加相关单元测试

Closes #123
```

### 4. 拉取请求

1. 确保所有测试通过
2. 更新相关文档
3. 详细描述变更内容
4. 关联相关 Issue

## 📚 更多资源

### 文档
- [Polkadot SDK 文档](https://docs.substrate.io/)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Polkadot.js API](https://polkadot.js.org/docs/)

### 社区
- [Substrate Stack Exchange](https://substrate.stackexchange.com/)
- [Polkadot Discord](https://discord.gg/polkadot)
- [Substrate GitHub](https://github.com/paritytech/polkadot-sdk)

### 工具
- [Polkadot.js Apps](https://polkadot.js.org/apps/)
- [Substrate Contracts UI](https://contracts-ui.substrate.io/)
- [SubQuery](https://subquery.network/)

## 📄 许可证

本项目基于 [GPL-3.0 许可证](LICENSE) 开源。

## 🙋‍♂️ 支持

如有问题或需要帮助，请：

1. 查看 [FAQ 部分](#故障排除)
2. 搜索现有 [Issues](https://github.com/your-org/bitnice/issues)
3. 创建新的 Issue 并详细描述问题
4. 联系维护团队

---

**BitNice 区块链** - 由广西红蚂蚁网络科技有限公司开发维护 
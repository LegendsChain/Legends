# BitNice 区块链项目使用指南

## 🎯 项目目标
- 使用最新的 polkadot-sdk 重构项目
- 共识算法采用 BTC 的共识算法
- 区块时间设置为 6s 
- 代币名称： BitNice
- 代币符号： BN
- 代币总量： 2100000000

## 📋 快速使用指南

### 1. 环境准备
确保您的系统已安装：
- Rust 1.75+ 工具链
- WASM 编译目标（wasm32-unknown-unknown, wasm32v1-none）
- 构建依赖（build-essential, pkg-config, libssl-dev等）

### 2. 编译项目
```bash
# 清理缓存
cargo clean

# 编译发布版本
cargo build --release

# 验证编译结果
ls -la target/release/
```

### 3. 启动节点

#### 开发模式（单节点测试）
```bash
./target/release/node-template --dev
```

#### 挖矿模式
```bash
./target/release/node-template \
    --validator \
    --mining \
    --name "BitNice-Miner" \
    --base-path ./miner-data
```

#### 生产模式
```bash
./target/release/node-template \
    --validator \
    --name "BitNice-Node" \
    --chain local \
    --base-path ./data \
    --port 30333 \
    --rpc-port 9933
```

### 4. 基本操作

#### 查看节点状态
```bash
# 获取链信息
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain", "params":[]}' \
     http://localhost:9933

# 获取节点健康状态
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
     http://localhost:9933
```

#### 连接Polkadot.js Apps
1. 打开 https://polkadot.js.org/apps/
2. 选择 Development -> Local Node
3. 确保连接到 ws://127.0.0.1:9944

### 5. 故障排除

#### 编译问题
```bash
# 如果遇到编译错误，尝试：
rustup update
rustup target add wasm32v1-none
cargo clean && cargo build --release
```

#### 运行时问题
```bash
# 如果节点无法启动：
rm -rf ./data  # 清理数据目录
./target/release/node-template --dev  # 重新启动
```

#### 检查日志
```bash
# 启用详细日志
RUST_LOG=debug ./target/release/node-template --dev
```

## 🔧 高级配置

### 自定义链规范
项目支持自定义链配置，包括：
- 创世区块设置
- 验证节点配置
- 网络参数调整

### PoW 挖矿优化
- 调整挖矿线程数
- 配置难度参数
- 监控挖矿性能

### 性能调优
- 编译优化参数
- 系统级优化
- 网络配置优化

## 📚 更多资源

详细文档请参考：
- [README.md](../README.md) - 完整使用指南
- [architecture.md](./architecture.md) - 技术架构文档
- [Polkadot SDK 文档](https://docs.substrate.io/)

## ⚠️ 注意事项

1. **开发环境**: 使用 `--dev` 标志仅适用于开发和测试
2. **数据备份**: 生产环境请定期备份数据目录
3. **安全考虑**: 生产部署时注意网络安全和密钥管理
4. **版本兼容**: 确保使用兼容的 Rust 和 polkadot-sdk 版本

---

**BitNice 区块链** - 由广西红蚂蚁网络科技有限公司开发

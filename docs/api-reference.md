# BitNice API 参考手册

## RPC 接口文档

### 基础 RPC 方法

#### system_chain
获取链名称
```bash
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_chain", "params":[]}' \
     http://localhost:9933
```

#### system_health  
获取节点健康状态
```bash
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
     http://localhost:9933
```

### PoW 相关接口

#### pow_difficulty
获取当前挖矿难度
```bash
curl -H "Content-Type: application/json" \
     -d '{"id":1, "jsonrpc":"2.0", "method": "pow_difficulty", "params":[]}' \
     http://localhost:9933
``` 
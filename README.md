# WalletBot 钱包机器人

一个基于 Rust 和 Teloxide 的 Telegram 钱包管理机器人。

## 功能特性

- 📱 **智能消息解析**: 自动识别并解析钱包交易消息
- 💰 **实时余额计算**: 智能计算钱包余额并自动更新
- 🗄️ **数据库存储**: 使用 SQLite 存储交易记录和钱包信息
- 🔄 **重复消息处理**: 防止重复处理相同的消息
- 📊 **支持多种交易类型**: 支持出账、入账等多种交易类型
- 🛡️ **错误处理**: 完善的错误处理和重试机制

## 快速开始

### 环境要求

- Rust 1.70+
- Telegram Bot Token

### 安装

```bash
git clone https://github.com/your-username/WalletBot.git
cd WalletBot
cargo build --release
```

### 配置

1. 复制配置文件：
```bash
cp config.example .env
```

2. 编辑 `.env` 文件，设置你的 Telegram Bot Token：
```
TELEGRAM_BOT_TOKEN=your_bot_token_here
DATABASE_URL=wallet.db
BOT_NAME=WalletBot
MAX_RETRY_ATTEMPTS=3
PROCESSING_TIMEOUT=30
```

### 运行

```bash
cargo run
```

## 集成测试

本项目包含了完整的集成测试系统，使用 Mock 对象来模拟 Telegram API。

### 运行测试

```bash
# 运行所有集成测试
cargo test --test integration_tests

# 查看详细输出
cargo test --test integration_tests -- --nocapture

# 运行特定测试
cargo test --test integration_tests test_message_parser
```

### 测试覆盖范围

#### 基础功能测试
- ✅ **消息解析器测试**: 测试钱包消息格式解析
- ✅ **数据库操作测试**: 测试钱包创建、更新、交易记录
- ✅ **Mock Bot API测试**: 测试消息发送、编辑、删除

#### 业务逻辑测试
- ✅ **完整消息流程测试**: 测试端到端的消息处理流程
- ✅ **错误处理测试**: 测试各种无效消息格式的处理
- ✅ **重复消息处理测试**: 测试消息去重机制

#### 性能和并发测试
- ✅ **性能测试**: 测试消息解析和数据库操作的性能
- ✅ **并发操作测试**: 测试多线程环境下的安全性

### 测试结果示例

```
running 8 tests
✅ 并发操作测试通过
✅ 性能测试结果:
  - 1000次消息解析耗时: 28.3446ms
  - 100次数据库操作耗时: 1.0065758s
  - 平均单次解析耗时: 28.344µs
  - 平均单次数据库操作耗时: 10.065758ms
✅ 重复消息处理测试通过
✅ 完整消息处理流程测试通过
✅ Mock Bot API测试通过
✅ 数据库操作测试通过
✅ 消息解析器测试通过
✅ 错误处理测试通过

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 使用方法

### 支持的消息格式

```
#钱包名称 #月份 #年份
#交易类型 金额元
```

#### 示例

```
#支付宝 #12月 #2024年
#出账 150.00元
```

机器人会自动计算余额并更新消息：

```
#支付宝 #12月 #2024年
#出账 150.00元
#总额 1000.00元
```

### 支持的命令

- `/start` - 开始使用机器人
- `/help` - 显示详细帮助信息
- `/status` - 查看机器人运行状态  
- `/reprocess` - 重新处理消息（回复特定消息使用）

## 🎯 实际使用指南

### 第一次使用

1. **配置并启动机器人**
   ```bash
   # 复制配置文件
   cp config.example .env
   
   # 编辑 .env 文件，设置你的 Bot Token
   # TELEGRAM_BOT_TOKEN=your_token_here
   
   # 启动机器人
   cargo run --release
   ```

2. **机器人启动成功后会显示**
   ```
   ✅ Bot connected successfully:
     - Username: @your_bot_username  
     - Name: WalletBot
     - ID: 123456789
   🎯 Starting message processing...
   💡 Bot is now ready to receive messages!
   ```

3. **在 Telegram 中开始使用**
   - 找到你的机器人
   - 发送 `/start` 获取欢迎消息
   - 发送 `/help` 查看使用说明

### 交易记录流程

1. **发送交易消息**
   ```
   #支付宝 #12月 #2024年
   #出账 150.00元
   ```

2. **机器人自动处理**
   - ✅ 解析消息格式
   - 💾 记录到数据库  
   - 📊 计算新余额
   - ✏️ 编辑原消息添加总额
   - 📨 发送确认消息

3. **处理结果**
   - 原消息被编辑为：
     ```
     #支付宝 #12月 #2024年
     #出账 150.00元
     #总额 850.00元
     ```
   - 收到确认消息：
     ```
     ✅ 交易已记录
     📊 钱包：支付宝
     💰 当前余额：850.00元
     ```

### 支持的交易类型 [[memory:3291148]]

- `#出账` / `#支出` - 资金流出（减少余额）
- `#入账` / `#收入` - 资金流入（增加余额）

### 错误处理

- **格式错误**: 发送详细的使用说明
- **重复消息**: 发送"消息已处理"提示  
- **处理失败**: 发送错误提示，建议重试

### 多聊天支持

钱包数据按聊天/频道完全隔离 [[memory:3291148]]：
- 私聊、群聊、频道中的同名钱包余额独立
- 每个聊天环境有独立的消息处理状态
- 支持无限个聊天同时使用
- `/help` - 显示帮助信息
- `/status` - 查看机器人状态
- `/reprocess` - 重新处理消息（回复目标消息）

## 架构设计

### 模块结构

```
src/
├── bot/              # 机器人核心
│   ├── handler.rs    # 消息处理器
│   ├── commands.rs   # 命令处理
│   └── traits.rs     # Bot API 抽象
├── database/         # 数据库相关
│   ├── models.rs     # 数据模型
│   └── operations.rs # 数据库操作
├── parser/           # 消息解析
│   ├── message.rs    # 消息解析器
│   └── regex.rs      # 正则表达式
├── calculator/       # 余额计算
│   └── balance.rs    # 余额计算器
├── config/           # 配置管理
│   └── settings.rs   # 配置设置
├── error.rs          # 错误处理
├── retry.rs          # 重试机制
├── utils.rs          # 工具函数
└── main.rs           # 主入口
```

### 测试架构

```
tests/
└── integration_tests.rs  # 集成测试
    ├── MockBotApi        # Mock Telegram API
    ├── 基础功能测试        # 消息解析、数据库、Mock API
    ├── 业务逻辑测试        # 完整流程、错误处理、去重
    └── 性能并发测试        # 性能基准、并发安全
```

## 开发指南

### 添加新功能

1. 在相应模块中添加功能代码
2. 在 `tests/integration_tests.rs` 中添加测试用例
3. 运行测试确保不会破坏现有功能

### 测试驱动开发

```rust
#[tokio::test]
#[serial]
async fn test_new_feature() -> Result<()> {
    let db = create_test_db().await?;
    let mock_bot = MockBotApi::new();
    
    // 测试逻辑
    
    println!("✅ 新功能测试通过");
    Ok(())
}
```

### 性能优化

- 使用 `cargo test --test integration_tests test_performance` 检查性能
- 优化数据库查询和消息解析逻辑
- 监控并发操作的安全性

## 贡献指南

1. Fork 本项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](LICENSE) 文件。

## 支持

如果您遇到问题或有建议，请：

1. 查看 [集成测试文档](INTEGRATION_TESTS.md)
2. 创建 Issue 描述问题
3. 提交 Pull Request 修复问题

---

**注意**: 这是一个演示项目，用于展示 Rust 中的集成测试实现。在生产环境中使用前，请确保正确配置所有安全设置。 
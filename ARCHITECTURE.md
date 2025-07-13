# WalletBot 项目架构和实现方案

## 1. 系统架构概述

### 1.1 整体架构
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Telegram      │    │   WalletBot     │    │    SQLite       │
│   Channel       │───▶│   Application   │───▶│   Database      │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   消息处理流程    │
                       │   1. 接收消息     │
                       │   2. 解析格式     │
                       │   3. 计算余额     │
                       │   4. 更新消息     │
                       └─────────────────┘
```

### 1.2 核心模块设计

#### Bot 模块 (`src/bot/`)
- **handler.rs**: 消息处理核心逻辑
- **commands.rs**: 命令处理（管理员命令）
- **mod.rs**: 模块定义和统一导出

#### Parser 模块 (`src/parser/`)
- **message.rs**: 消息解析逻辑
- **regex.rs**: 正则表达式模式定义
- **mod.rs**: 解析器模块定义

#### Database 模块 (`src/database/`)
- **models.rs**: 数据模型定义
- **operations.rs**: 数据库操作封装
- **mod.rs**: 数据库模块定义

#### Calculator 模块 (`src/calculator/`)
- **balance.rs**: 余额计算逻辑
- **mod.rs**: 计算器模块定义

#### Config 模块 (`src/config/`)
- **settings.rs**: 配置文件处理
- **mod.rs**: 配置模块定义

## 2. 数据模型设计

### 2.1 数据库表结构

#### wallets 表
```sql
CREATE TABLE wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,           -- 钱包名称（如"这个钱包的总存款"）
    current_balance REAL NOT NULL DEFAULT 0.0,  -- 当前余额
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);
```

#### transactions 表
```sql
CREATE TABLE transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_id INTEGER NOT NULL,
    transaction_type TEXT NOT NULL,  -- "出账" 或 "入账"
    amount REAL NOT NULL,           -- 金额
    month TEXT NOT NULL,            -- 月份
    year TEXT NOT NULL,             -- 年份
    message_id INTEGER,             -- Telegram 消息ID
    chat_id INTEGER,                -- Telegram 聊天ID
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);
```

#### messages 表
```sql
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id INTEGER NOT NULL,    -- Telegram 消息ID
    chat_id INTEGER NOT NULL,       -- Telegram 聊天ID
    wallet_id INTEGER NOT NULL,     -- 关联的钱包ID
    has_total BOOLEAN DEFAULT FALSE, -- 是否包含总额
    processed BOOLEAN DEFAULT FALSE, -- 是否已处理
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (wallet_id) REFERENCES wallets(id)
);
```

### 2.2 Rust 数据结构

```rust
// src/database/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Option<i64>,
    pub name: String,
    pub current_balance: f64,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Option<i64>,
    pub wallet_id: i64,
    pub transaction_type: String, // "出账" 或 "入账"
    pub amount: f64,
    pub month: String,
    pub year: String,
    pub message_id: Option<i64>,
    pub chat_id: Option<i64>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i64>,
    pub message_id: i64,
    pub chat_id: i64,
    pub wallet_id: i64,
    pub has_total: bool,
    pub processed: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedMessage {
    pub wallet_name: String,
    pub transaction_type: String,
    pub amount: f64,
    pub month: String,
    pub year: String,
    pub original_text: String,
}
```

## 3. 核心算法设计

### 3.1 消息解析算法

```rust
// src/parser/message.rs
use regex::Regex;
use crate::database::models::ParsedMessage;

pub struct MessageParser {
    wallet_regex: Regex,
    transaction_regex: Regex,
    amount_regex: Regex,
    time_regex: Regex,
}

impl MessageParser {
    pub fn new() -> Self {
        Self {
            wallet_regex: Regex::new(r"#([^#\s]+)(?=\s+#\d+月)").unwrap(),
            transaction_regex: Regex::new(r"#(出账|入账)").unwrap(),
            amount_regex: Regex::new(r"(\d+(?:\.\d+)?)元").unwrap(),
            time_regex: Regex::new(r"#(\d+)月\s+#(\d+)年").unwrap(),
        }
    }

    pub fn parse(&self, text: &str) -> Option<ParsedMessage> {
        // 解析钱包名称
        let wallet_name = self.wallet_regex.captures(text)?
            .get(1)?.as_str().to_string();

        // 解析交易类型
        let transaction_type = self.transaction_regex.captures(text)?
            .get(1)?.as_str().to_string();

        // 解析金额
        let amount: f64 = self.amount_regex.captures(text)?
            .get(1)?.as_str().parse().ok()?;

        // 解析时间
        let time_captures = self.time_regex.captures(text)?;
        let month = time_captures.get(1)?.as_str().to_string();
        let year = time_captures.get(2)?.as_str().to_string();

        Some(ParsedMessage {
            wallet_name,
            transaction_type,
            amount,
            month,
            year,
            original_text: text.to_string(),
        })
    }
}
```

### 3.2 余额计算算法

```rust
// src/calculator/balance.rs
use crate::database::models::{Wallet, Transaction};
use crate::database::operations::DatabaseOperations;

pub struct BalanceCalculator {
    db: DatabaseOperations,
}

impl BalanceCalculator {
    pub fn new(db: DatabaseOperations) -> Self {
        Self { db }
    }

    pub async fn calculate_new_balance(
        &self,
        wallet_name: &str,
        transaction_type: &str,
        amount: f64,
        month: &str,
        year: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        // 获取或创建钱包
        let wallet = self.db.get_or_create_wallet(wallet_name).await?;
        
        // 获取当前余额
        let current_balance = wallet.current_balance;
        
        // 计算新余额
        let new_balance = match transaction_type {
            "出账" => current_balance - amount,
            "入账" => current_balance + amount,
            _ => current_balance,
        };

        Ok(new_balance)
    }

    pub async fn get_latest_balance(
        &self,
        wallet_name: &str,
        month: &str,
        year: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        // 从历史交易中获取最新的余额信息
        self.db.get_latest_balance(wallet_name, month, year).await
    }
}
```

### 3.3 消息处理流程

```rust
// src/bot/handler.rs
use teloxide::{Bot, types::Message, RequestError};
use crate::parser::message::MessageParser;
use crate::calculator::balance::BalanceCalculator;
use crate::database::operations::DatabaseOperations;

pub struct MessageHandler {
    parser: MessageParser,
    calculator: BalanceCalculator,
    db: DatabaseOperations,
}

impl MessageHandler {
    pub fn new(db: DatabaseOperations) -> Self {
        let calculator = BalanceCalculator::new(db.clone());
        Self {
            parser: MessageParser::new(),
            calculator,
            db,
        }
    }

    pub async fn handle_message(
        &self,
        bot: &Bot,
        message: &Message,
    ) -> Result<(), RequestError> {
        // 检查消息是否包含总额
        if let Some(text) = &message.text {
            if text.contains("#总额") {
                return Ok(()); // 跳过已包含总额的消息
            }

            // 解析消息
            if let Some(parsed) = self.parser.parse(text) {
                // 计算新余额
                let new_balance = self.calculator.calculate_new_balance(
                    &parsed.wallet_name,
                    &parsed.transaction_type,
                    parsed.amount,
                    &parsed.month,
                    &parsed.year,
                ).await.unwrap_or(0.0);

                // 构建新消息文本
                let new_text = format!(
                    "{}\n#总额 {:.2}元",
                    text,
                    new_balance
                );

                // 编辑消息
                bot.edit_message_text(
                    message.chat.id,
                    message.id,
                    new_text,
                ).await?;

                // 记录交易
                self.db.record_transaction(
                    &parsed.wallet_name,
                    &parsed.transaction_type,
                    parsed.amount,
                    &parsed.month,
                    &parsed.year,
                    message.id.0 as i64,
                    message.chat.id.0,
                ).await.unwrap_or(());

                // 更新钱包余额
                self.db.update_wallet_balance(
                    &parsed.wallet_name,
                    new_balance,
                ).await.unwrap_or(());
            }
        }

        Ok(())
    }
}
```

## 4. 实现计划

### 4.1 开发阶段划分

#### 阶段 1: 基础框架搭建
- [ ] 配置 Cargo.toml 依赖
- [ ] 实现基本的项目结构
- [ ] 配置环境变量管理
- [ ] 设置日志系统

#### 阶段 2: 数据库层实现
- [ ] 实现 SQLite 数据库连接
- [ ] 创建数据库表结构
- [ ] 实现基本的 CRUD 操作
- [ ] 添加数据库迁移支持

#### 阶段 3: 消息解析器实现
- [ ] 实现正则表达式模式
- [ ] 编写消息解析逻辑
- [ ] 添加解析结果验证
- [ ] 编写单元测试

#### 阶段 4: 计算器模块实现
- [ ] 实现余额计算逻辑
- [ ] 添加历史余额查询
- [ ] 实现交易记录管理
- [ ] 添加计算正确性测试

#### 阶段 5: Telegram Bot 集成
- [ ] 实现 Bot 消息监听
- [ ] 添加消息处理逻辑
- [ ] 实现消息编辑功能
- [ ] 添加错误处理和重试机制

#### 阶段 6: 测试和优化
- [ ] 编写集成测试
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 文档完善

### 4.2 技术难点和解决方案

#### 4.2.1 消息识别准确性
**问题**: 确保正则表达式能够准确识别各种格式的消息
**解决方案**: 
- 使用多个正则表达式模式组合
- 添加消息格式验证
- 提供灵活的配置选项

#### 4.2.2 并发处理
**问题**: 多个消息同时处理可能导致余额计算错误
**解决方案**:
- 使用数据库事务确保数据一致性
- 实现消息处理队列
- 添加重试机制

#### 4.2.3 错误恢复
**问题**: Bot 重启后需要恢复未处理的消息
**解决方案**:
- 记录消息处理状态
- 实现启动时的数据恢复
- 添加手动重新处理命令

### 4.3 部署和维护

#### 4.3.1 配置管理
- 使用 `.env` 文件管理环境变量
- 支持多环境配置
- 敏感信息加密存储

#### 4.3.2 监控和日志
- 结构化日志记录
- 错误监控和告警
- 性能指标收集

#### 4.3.3 备份和恢复
- 定期数据库备份
- 配置文件备份
- 灾难恢复计划

## 5. 风险评估和缓解措施

### 5.1 技术风险
- **Telegram API 限制**: 实现请求频率控制
- **数据库性能**: 优化查询，添加索引
- **消息解析错误**: 完善测试用例，添加验证

### 5.2 业务风险
- **计算错误**: 添加多重验证机制
- **数据丢失**: 实现自动备份
- **权限管理**: 实现管理员命令系统

这个架构方案提供了一个完整的实现路径，确保系统的可靠性、可维护性和可扩展性。接下来等待您的审核，通过后即可开始实施。 
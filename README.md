# WalletBot - Telegram 钱包管理机器人

## 项目简介

WalletBot 是一个基于 Telegram Bot 的钱包管理应用，能够自动识别和处理频道中的钱包操作消息，并实时更新钱包余额。

## 功能特性

- 🤖 **自动消息识别**：识别频道中特定格式的钱包操作消息
- 💰 **余额计算**：根据历史记录自动计算和更新钱包总额
- 📝 **消息编辑**：自动编辑最新消息，添加计算后的总额信息
- 🗄️ **轻量数据库**：使用 SQLite 本地数据库，无需额外安装
- 🔍 **智能解析**：支持多种钱包标签和时间标签的识别

## 消息格式

### 输入格式
```
#这个钱包的总存款 #7月 #2025年
#出账 10000.00元
```

### 输出格式
```
#这个钱包的总存款 #7月 #2025年
#出账 10000.00元
#总额 215000元
```

## 技术栈

- **语言**: Rust
- **框架**: Tokio (异步运行时)
- **Telegram API**: teloxide
- **数据库**: SQLite (rusqlite)
- **正则表达式**: regex
- **序列化**: serde
- **日志**: log + env_logger

## 快速开始

### 环境要求
- Rust 1.70+
- Telegram Bot Token

### 配置
1. 创建 `.env` 文件：
```env
TELEGRAM_BOT_TOKEN=你的_Bot_Token
DATABASE_URL=wallet_bot.db
```

2. 编译运行：
```bash
cargo build --release
cargo run
```

### 使用方法
1. 将 Bot 添加到目标频道
2. 给予 Bot 消息读取和编辑权限
3. Bot 会自动监听并处理符合格式的消息

## 项目结构

```
WalletBot/
├── src/
│   ├── main.rs           # 主程序入口
│   ├── bot/              # Bot 相关模块
│   │   ├── mod.rs        # Bot 模块定义
│   │   ├── handler.rs    # 消息处理器
│   │   └── commands.rs   # 命令处理
│   ├── database/         # 数据库相关
│   │   ├── mod.rs        # 数据库模块定义
│   │   ├── models.rs     # 数据模型
│   │   └── operations.rs # 数据库操作
│   ├── parser/           # 消息解析器
│   │   ├── mod.rs        # 解析器模块定义
│   │   ├── message.rs    # 消息解析逻辑
│   │   └── regex.rs      # 正则表达式定义
│   ├── calculator/       # 计算器模块
│   │   ├── mod.rs        # 计算器模块定义
│   │   └── balance.rs    # 余额计算逻辑
│   └── config/           # 配置管理
│       ├── mod.rs        # 配置模块定义
│       └── settings.rs   # 配置文件处理
├── migrations/           # 数据库迁移文件
├── tests/               # 测试文件
├── .env.example         # 环境变量示例
├── Cargo.toml
└── README.md
```

## 开发指南

### 消息识别规则
- 钱包标签：`#钱包名称` 
- 时间标签：`#月份 #年份`
- 操作标签：`#出账` 或 `#入账`
- 金额格式：`数字.数字元`

### 数据库设计
- `wallets` 表：存储钱包信息
- `transactions` 表：存储交易记录
- `messages` 表：存储消息信息和关联

### 贡献指南
1. Fork 本项目
2. 创建特性分支
3. 提交更改
4. 发起 Pull Request

## 许可证

MIT License

## 联系方式

如有问题或建议，请通过 GitHub Issues 联系。 
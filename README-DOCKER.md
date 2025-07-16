# 🐳 Docker 部署指南

本文档介绍如何使用 Docker 运行 WalletBot。

## 🚀 快速开始

### 使用 Docker Compose（推荐）

1. **克隆仓库**
   ```bash
   git clone <repository-url>
   cd WalletBot
   ```

2. **配置环境变量**
   ```bash
   cp config.example .env
   # 编辑 .env 文件，设置你的 TELEGRAM_BOT_TOKEN
   ```

3. **启动服务**
   ```bash
   docker-compose up -d
   ```

4. **查看日志**
   ```bash
   docker-compose logs -f walletbot
   ```

5. **停止服务**
   ```bash
   docker-compose down
   ```

### 使用预构建镜像

如果你想使用 GitHub Container Registry 中的预构建镜像：

```bash
# 拉取最新镜像
docker pull ghcr.io/your-username/walletbot:latest

# 运行容器
docker run -d \
  --name walletbot \
  --restart unless-stopped \
  -e TELEGRAM_BOT_TOKEN=your_bot_token_here \
  -e DATABASE_URL=/app/data/wallet_bot.db \
  -v walletbot_data:/app/data \
  ghcr.io/your-username/walletbot:latest
```

## �� 环境变量

| 变量名 | 必需 | 默认值 | 说明 |
|--------|------|--------|------|
| `TELEGRAM_BOT_TOKEN` | ✅ | - | Telegram Bot Token |
| `DATABASE_URL` | ❌ | `/app/data/wallet_bot.db` | SQLite 数据库路径 |
| `RUST_LOG` | ❌ | `info` | 日志级别 |
| `BOT_NAME` | ❌ | `WalletBot` | 机器人名称 |
| `MAX_RETRY_ATTEMPTS` | ❌ | `3` | 最大重试次数 |
| `PROCESSING_TIMEOUT` | ❌ | `30` | 处理超时时间（秒） |

## 💾 数据持久化

数据库文件存储在 `/app/data/` 目录中，通过 Docker Volume 进行持久化。

## 🌐 多架构支持

我们的镜像支持以下架构：
- `linux/amd64` (x86_64)  
- `linux/arm64` (aarch64)

Docker 会自动选择适合你系统的架构。

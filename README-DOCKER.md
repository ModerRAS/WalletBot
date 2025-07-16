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

## 🔄 CI/CD 流程

Docker 镜像的构建和发布遵循严格的质量保证流程：

### 测试优先的部署策略

1. **自动化测试**：每次代码提交都会触发完整的测试套件
   - 单元测试：`cargo test`
   - 集成测试：Cucumber 测试套件
   - 代码质量检查：`cargo fmt` 和 `cargo clippy`

2. **构建流程**：只有在所有测试通过后才会构建 Docker 镜像
   - 多架构构建（AMD64 + ARM64）
   - 安全扫描：使用 Trivy 进行漏洞扫描
   - 缓存优化：使用 GitHub Actions 缓存

3. **发布策略**：
   - **主分支推送**：自动构建并推送到 `ghcr.io`，标记为 `latest`
   - **标签发布**：版本标签会生成语义化版本镜像（如 `v1.0.0`、`v1.0`、`v1`）
   - **Pull Request**：构建镜像但不推送，确保 PR 不会破坏构建

### 质量保证

- ✅ 35+ 项 Cucumber 集成测试确保功能完整性
- ✅ 自动化代码格式化和 lint 检查
- ✅ 容器安全扫描和漏洞报告
- ✅ 多平台兼容性验证

这确保了每个发布的 Docker 镜像都经过了完整的测试验证，可以安全地用于生产环境。

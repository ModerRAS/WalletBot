# ================================
# 运行时镜像 - 直接使用预构建的二进制文件
# ================================
FROM debian:12-slim

# 构建参数，用于指定目标架构
ARG TARGETARCH

# 安装运行时依赖（如果需要的话）
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r walletbot \
    && useradd -r -g walletbot walletbot

# 设置工作目录
WORKDIR /app

# 根据目标架构复制相应的预构建二进制文件
COPY artifacts/${TARGETARCH}/walletbot /app/walletbot

# 创建数据目录并设置权限
RUN mkdir -p /app/data && chown -R walletbot:walletbot /app

# 设置环境变量
ENV RUST_LOG=info
ENV DATABASE_URL=/app/data/wallet_bot.db

# 使用非root用户运行
USER walletbot:walletbot

# 启动应用
ENTRYPOINT ["/app/walletbot"] 
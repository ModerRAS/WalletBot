# ================================
# 构建阶段
# ================================
FROM rust:1.75-slim as builder

# 安装必要的系统依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./

# 创建虚拟的main.rs来预编译依赖
RUN mkdir src && echo "fn main() {}" > src/main.rs

# 预编译依赖以利用Docker缓存
RUN cargo build --release && rm -rf src target/release/walletbot target/release/deps/walletbot*

# 复制源代码
COPY src ./src

# 构建实际程序
RUN cargo build --release

# 在构建阶段创建数据目录结构
RUN mkdir -p /tmp/data

# ================================
# 运行阶段 - 使用极简的distroless镜像
# ================================
FROM gcr.io/distroless/cc-debian12

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件和数据目录
COPY --from=builder /app/target/release/walletbot /app/walletbot
COPY --from=builder /tmp/data /app/data

# 设置环境变量
ENV RUST_LOG=info
ENV DATABASE_URL=/app/data/wallet_bot.db

# 使用非root用户运行
USER 65534:65534

# 启动应用
ENTRYPOINT ["/app/walletbot"] 
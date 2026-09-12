##################################
# 第一阶段：构建 Rust 可执行文件
##################################
# 使用与本地工具链一致的 rust 镜像（Cargo.lock 依赖已按此版本解析）
FROM rust:1.98-bookworm AS builder

ARG APP_VERSION=0.1.0
WORKDIR /src

# 先只拷贝依赖清单，最大化利用构建缓存
COPY Cargo.toml Cargo.lock ./

# 预构建全部依赖（dummy main 仅用于缓存依赖层，随后会被真实源码覆盖）
RUN mkdir -p src \
    && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# 拷贝真实源码并构建
COPY src ./src
RUN cargo build --release --locked \
    && mv target/release/backendr /src/backendr

##################################
# 第二阶段：创建最终的运行时镜像
##################################
# 说明：rust 构建产物默认动态链接 glibc（libc/libm/libgcc_s），
# 运行镜像必须与构建镜像同源（bookworm），不能使用 musl 的 alpine。
FROM docker.io/debian:bookworm-slim

ARG SERVICE_NAME=backendr

# 证书与时区（TZ 与后端容器约定保持一致）
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Asia/Shanghai

WORKDIR /app

COPY --from=builder /src/backendr /app/bin/server

# 创建一个名为 appuser 的非 root 用户
RUN useradd --create-home --shell /usr/sbin/nologin appuser

# 创建可写数据目录（上传文件 + 备份/归档），供非 root 的 appuser 使用。
# 相关默认路径：GW_ADMIN_UPLOAD_DIR=uploads(→/app/uploads)、backup(./data/backup)、
# audit archive(./data/audit-archive)。若目录不可写，上传/头像/备份会 500。
RUN mkdir -p /app/uploads /app/data/backup /app/data/audit-archive \
    && chown -R appuser:appuser /app/uploads /app/data

# 切换到非特权用户
USER appuser

# 暴露服务端口（backendr 默认监听 7666，可用 GW_ADMIN_HTTP_ADDR 覆盖）
EXPOSE 7666

CMD ["/app/bin/server"]
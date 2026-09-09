##################################
# 第一阶段：构建 GO 可执行文件
##################################
# 使用官方的 Go 基础镜像作为构建环境（与 go.mod 的 go 1.26.4 对齐）
ARG GO_VERSION=1.26.4
FROM golang:${GO_VERSION}-alpine AS builder

ARG SERVICE_NAME=backendz
ARG APP_VERSION=1.0.0

# 设置工作目录
WORKDIR /src

# 先拷贝依赖清单，最大化利用构建缓存
COPY go.mod go.sum ./

# 下载依赖，国内代理链：首选 goproxy.cn，备选 direct（直接下载）
# 增加 GOSUMDB 配置避免校验服务器访问失败
RUN GOPROXY=https://goproxy.cn,direct GOSUMDB=off go mod download

# 拷贝真实源码并编译可执行文件（静态链接，便于精简运行镜像）
COPY . .
RUN CGO_ENABLED=0 \
    GOOS=linux \
    GOARCH=amd64 \
    go build -ldflags "-X main.version=$APP_VERSION" \
    -o /src/bin/${SERVICE_NAME} .

##################################
# 第二阶段：创建最终的运行时镜像
##################################
FROM docker.io/alpine:latest

ARG SERVICE_NAME=backendz

# 安装必要的证书（如果应用程序需要进行 HTTPS 请求）及时区
RUN apk --no-cache add ca-certificates tzdata

ENV TZ=Asia/Shanghai

# 设置工作目录
WORKDIR /app

# 从第一阶段的构建结果中复制可执行文件到当前工作目录
COPY --from=builder /src/bin/${SERVICE_NAME} /app/bin/server

# 拷贝配置文件
COPY etc/admin-api.yaml /app/etc/admin-api.yaml

# 创建一个名为 appuser 的非 root 用户
RUN adduser -D appuser

# 切换到非特权用户
USER appuser:appuser

# 健康检查：依赖 DevServer 的 /healthz（见 etc/admin-api.yaml 的 DevServer 段，端口 6060）
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
  CMD wget -q -O - http://127.0.0.1:6060/healthz >/dev/null 2>&1 || exit 1

# 暴露服务端口（backendz 默认监听 8888，见 etc/admin-api.yaml）
EXPOSE 8888 6060

CMD ["/app/bin/server", "-f", "/app/etc/admin-api.yaml"]
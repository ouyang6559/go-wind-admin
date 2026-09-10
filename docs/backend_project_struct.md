# 后端项目结构说明

```text
├── api
│   ├── gen
│   │   └── go
│   │       ├── admin/service/v1
│   │       ├── file/service/v1
│   │       ├── identity/service/v1
│   │       ├── permission/service/v1
│   │       └── ...
│   └── protos
│       ├── admin/service/v1
│       ├── file/service/v1
│       ├── identity/service/v1
│       ├── permission/service/v1
│       └── ...
├── app
│   └── admin/service
│       ├── cmd/server
│       │   └── assets
│       ├── configs
│       └── internal
│           ├── data
│           │   ├── ent
│           │   ├── gorm
│           │   └── providers
│           ├── middleware
│           │   ├── auth
│           │   └── logging
│           ├── server
│           │   └── providers
│           └── service
│               └── providers
├── pkg
├── scripts
│   ├── deploy
│   ├── docker
│   └── env
├── sql
├── app.mk
├── Makefile
├── docker-compose.yaml
├── docker-compose.libs.yaml
└── Dockerfile
```

1. `api`：存放 API 相关代码，API使用Protobuf定义，使用Buf进行编译管理。
    - `gen`：存放 API 服务生成的代码，目前只有 Go 语言的代码；
    - `protos`：存放 API 服务的 Protobuf的 proto 文件，它的目录结构是`{服务名}/service/{版本号}`。
    - `buf.gen.yaml`：buf 配置文件，用于生成 API 服务的 Go 代码。
    - `buf.admin.openapi.gen.yaml`：buf 配置文件，用于生成 Admin 服务的 OpenAPI 文档。
    - `buf.vue-vben.admin.typescript.gen.yaml`：buf 配置文件，用于生成 Vue Vben 版本的 TypeScript 代码。
    - `buf.vue-element.admin.typescript.gen.yaml`：buf 配置文件，用于生成 Vue Element 版本的 TypeScript 代码。
    - `buf.react.admin.typescript.gen.yaml`：buf 配置文件，用于生成 React 版本的 TypeScript 代码。
    - `buf.yaml`：buf 配置文件。
2. `app`：存放应用服务相关代码，它的目录结构是`{服务名}/service}`，目前只有 Admin 服务。
    - `admin/service`：存放 Admin 服务相关代码
        - `Makefile`：Makefile 文件，调用项目根目录下的`app.mk`，用于构建、运行、测试 Admin 服务。
        - `cmd`：存放 Admin 服务的命令行代码
            - `server`：存放 Admin 服务的入口代码
                - `assets`：存放 Admin 服务的静态资源文件，现在只存放了OpenAPI的静态资源文件。
        - `configs`：存放 Admin 服务的配置文件（server.yaml、data.yaml、auth.yaml、logger.yaml、oss.yaml、client.yaml）
        - `internal`：存放 Admin 服务的内部代码，使用internal目录是为了避免被外部代码引用。
            - `data`：存放 Admin 服务的数据访问代码
                - `ent`：存放 Admin 服务的 Ent 数据库 ORM 代码
                - `gorm`：存放 Admin 服务的 GORM 相关代码
            - `middleware`：存放 Admin 服务的中间件代码
                - `auth`：存放 Admin 服务的认证中间件代码
                - `logging`：存放 Admin 服务的日志中间件代码
            - `server`：存放 Admin 服务的服务端代码（HTTP/Asynq/SSE）
            - `service`：存放 Admin 服务的业务逻辑代码
3. `pkg`：存放通用公共包代码
4. `scripts`：存放部署脚本代码，用于项目的构建、部署、环境配置等。
    - `env/`：存放环境初始化脚本（支持 Ubuntu/CentOS/Rocky/macOS/Windows）
    - `docker/`：存放 Docker 部署脚本（full_deploy、libs_only）
    - `deploy/`：存放 PM2 进程管理脚本
5. `sql`：存放演示数据的 SQL 文件（MySQL 和 PostgreSQL 各一份）。系统默认数据（用户/角色/菜单/权限/语言等）**不在 SQL 里**——由服务启动时在 Go 侧自动播种（`pkg/constants/default_data.go` + 各 service 的 count==0 守卫；Api 表仅在空表时由启动期自动同步，后续靠管理页「接口同步」全量重建）。
6. `app.mk`：存放应用服务使用的 Makefile 文件，它由`app/{服务名}/service`下的`Makefile`调用，用于构建、运行、测试应用服务。
7. `Makefile`：项目根目录下的 Makefile 文件，可以用来安装 CLI 工具、生成 API 代码、构建 Docker 镜像等。
8. `docker-compose.yaml`：完整的 Docker Compose 配置文件，包含应用服务和所有依赖中间件。
9. `docker-compose.libs.yaml`：仅依赖中间件的 Docker Compose 配置文件，适合本地开发。
10. `Dockerfile`：用于构建后端服务的 Docker 镜像。

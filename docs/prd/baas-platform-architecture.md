# Indiebase BaaS Platform — Architecture PRD

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase Community — BaaS |
| Last updated | 2026-07-03 |

## 1. Overview

Indiebase BaaS 为独立开发者和小团队提供自托管的后端能力，包括项目管理、数据库 Schema 设计、数据 CRUD、文件存储与认证。本 PRD 定义平台的核心系统架构与 API 分层策略。

**设计原则：**

- CRUD 由 PostgREST 单一实现，Axum 不重复实现数据读写
- Dashboard 用户访问 Data API：`/api/data/*` + **Project Session**（Opaque Token + Redis，project 在 Session 内）
- SDK 访问 Data API：`/api/data/{project_id}/*` + **Project API Key**（鉴权 / role；**project 由 URL 指定**）
- 用户登录（Dashboard / Project）统一 **Opaque Token + Redis**；**不使用 JWT**
- **Manager API** 专指 Dashboard **管理面**（Platform API）；**Data API** 为独立 CRUD 网关；二者同属 Axum 进程，名称不混用
- PostgREST 仅内网可达，由 Axum **Data API 网关**代理

## 2. Goals

| Goal | Description |
|------|-------------|
| 职责分离 | Axum 负责平台管理（DDL、Metadata、Auth、Storage）；PostgREST 负责 CRUD |
| 能力复用 | Dashboard 数据浏览与 SDK/客户端共用同一 `/api/data/*` 网关 |
| 可替换性 | 后续替换 PostgREST 或新增 GraphQL 时，仅改 Axum **Data API 网关**，Dashboard（Manager API）无需变更 |
| 统一治理 | Opaque Token、Project API Key、search_path、权限、审计在 Axum 层统一处理 |

## 3. Non-Goals

- Axum 不实现 SELECT / INSERT / UPDATE / DELETE
- Dashboard、SDK、客户端不直连 PostgREST 或 PostgreSQL
- PostgREST 不对公网暴露；生产环境不 publish 端口，仅 Docker 内网或 localhost 供 Axum 代理
- 本 PRD 不包含 Financial Services 模块细节
- 本 PRD 不包含 Dashboard 前端 UI 规范

## 4. Technology Stack

### 4.1 Backend（Axum API Server）

Rust **Cargo workspace**；**Axum** 进程承载 **Manager API**（Platform）与 **Data API 网关** 两套路由，crate 技术选型：

| Layer | Crate / Tool | Usage |
|-------|--------------|-------|
| HTTP | **Axum** | Manager API（Platform）；Data API 网关；Auth 终止；PostgREST 透明代理 |
| PostgreSQL | **sqlx** | 异步 PG 访问；Platform 层直连（`public` schema、DDL、`api_keys` 等） |
| Query builder | **SeaQuery** | 类型安全 SQL / DDL 构建；配合 sqlx 执行 |
| IDs | **ulid** | Project ID（`public.projects.id`） |
| Object storage | **OpenDAL** | Storage Service；默认 `fs`，可选 S3（SeaweedFS） |
| HTTP client | *(TBD, e.g. reqwest)* | Axum → 内网 PostgREST 代理转发 |

**数据访问分工：**

| 数据 | 访问方式 | 说明 |
|------|----------|------|
| Platform 表（`public.*`） | **sqlx + SeaQuery** | users、projects、api_keys、metadata；DDL |
| 租户 CRUD（`proj_{ulid}.*`） | **PostgREST**（Axum 代理） | Axum **不**用 sqlx 做 SELECT / INSERT / UPDATE / DELETE |

Lint / test：`cargo fmt`、`cargo clippy`、`cargo test`。

### 4.2 Infrastructure

| Component | Version / Image | Role |
|-----------|-----------------|------|
| **PostgreSQL** | 17 | 平台 + 租户 schema 持久化 |
| **PostgREST** | postgrest/postgrest | 内网 REST CRUD 引擎 |
| **Redis** | 6.x | Dashboard / Project Session；API Key lookup 缓存 |
| **OpenDAL** | Rust crate | 文件存储（默认 local fs） |
| **SeaweedFS** | 可选，独立 compose | S3 兼容对象存储 |

本地栈：`docker compose up -d`（[docker-compose.yaml](../../docker-compose.yaml)）— Postgres、Redis、PostgREST。

### 4.3 Client SDK（MVP）

| Component | Technology | Role |
|-----------|------------|------|
| TS Data SDK | **`@supabase/postgrest-js`** + 薄封装 | MVP 仅 Data API；见 [mvp-sdk.md](./mvp-sdk.md) |

## 5. System Architecture

```
              Dashboard / SDK / Client

                              │
                              ▼

                    Axum API Server
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼

        Manager API                    Data API
      (Platform API)                   (CRUD 网关)
   /api/projects, /api/tables…      /api/data/*
   Dashboard 管理专用                  Session / API Key

              │                               │
              │                               ▼
              │                          PostgREST (internal)
              ▼                               │
      Project / Metadata / Storage            ▼
              │                          PostgreSQL CRUD
              ▼                          (proj_{ulid})
     PostgreSQL DDL
     (platform + tenant schemas)
              │
              ▼
           Redis
     (Dashboard / Project Session;
      API Key cache)
```

### 5.1 Components

| Component | Technology | Responsibility |
|-----------|------------|----------------|
| Dashboard | Web UI | 项目管理、表设计、数据浏览、文件管理、用户与 API Key 配置 |
| Axum API Server | Axum + sqlx + SeaQuery (Rust) | 承载 Manager API + Data API 网关 |
| Manager API | Axum 路由模块 | Platform 管理：DDL、Metadata、Auth、Storage、Project 生命周期 |
| Data API 网关 | Axum 路由模块 | `/api/data/*` → PostgREST 透明代理 |
| Project Service | Axum module | Project 生命周期、`proj_{ulid}` schema 创建与 PostgREST reload |
| Metadata Service | Axum module | Table / Column 元数据（platform schema）；设计器所需信息 |
| Storage Service | Axum + OpenDAL | 文件上传、下载、管理 |
| Redis | redis:6 | Dashboard / Project Session；Project API Key 查找缓存 |
| PostgREST | postgrest/postgrest | 内网 REST CRUD；由 **Data API 网关** 以 service role 代理 |
| PostgreSQL | Postgres 17 | 持久化存储（DDL + CRUD） |
| OpenDAL | Rust crate | 统一对象存储抽象；**默认 `fs`（本地目录）**；可切换 S3 等后端 |

## 6. API Architecture

**术语：**

| 名称 | 含义 | 路由前缀 | 主要调用方 |
|------|------|----------|------------|
| **Manager API** | Dashboard **管理面** API（即 **Platform API**） | `/api/projects`, `/api/tables`, `/api/auth/*`, … | Dashboard |
| **Data API** | 租户 **CRUD 网关**（PostgREST 代理） | `/api/data/*` | Dashboard Row Viewer、SDK |

二者由同一 **Axum** 进程暴露，但 **Manager API ≠ Data API**；文档中「Manager API」**不**包含 `/api/data/*`。

### 6.1 Manager API（Platform API）

Dashboard **管理专用**。负责平台治理，**不**代理 PostgREST CRUD。

```
/api/auth/login
/api/auth/logout
/api/auth/project/login
/api/auth/project/logout
/api/projects
/api/projects/{project_id}/api-keys
/api/tables
/api/columns
/api/files
/api/settings
```

Manager API 请求头（Dashboard 登录态）：

```
Authorization: Bearer <dashboard_session_token>
```

职责：

- Dashboard 用户登录 / 登出（Opaque Token + Redis，见 §11.3）
- 进入 / 退出 Project 上下文（Project 登录，见 §11.4）
- 创建 Project
- 签发 / 吊销 **Project API Key**（SDK 鉴权用，见 §11.2）
- 创建 Schema
- 创建 Table
- 修改 Column
- Metadata 读写
- Storage 操作

### 6.2 Data API（统一网关）

Axum 透明代理至内网 PostgREST，**不重复实现** CRUD 逻辑。Dashboard 与 SDK **路径不同**：用户侧 URL 不含 `project_id`（project 在 Project Session 内）；SDK 侧 URL **含** `{project_id}`（project 由路径指定，Key 只做鉴权）。

#### 6.2.1 Dashboard 用户（Row Viewer）

Dashboard 用户访问 Data API **只用 Project Session** — **Opaque Token + Redis**（见 §11.4）。**不是 API Key，也不是 JWT。**

**路径与请求头：**

```
GET /api/data/users?select=*
Authorization: Bearer <project_session_token>
```

**代理流程：**

```
Dashboard (Row Viewer)
    │
    │  Authorization: Bearer <project_session_token>
    ▼
GET /api/data/users?select=*
    │
    ▼
Data API 网关 (Axum)
    │  1. Redis  lookup  project_session:{token}
    │  2. 得到 { user_id, project_id, project_role, … }   ← project 来自 Session
    │  3. 映射 project_id → schema proj_{ulid}
    │  4. 校验用户在该 project 内的权限
    │  5. 注入 Accept-Profile: proj_{ulid}
    │  6. 以内网 service role 转发
    ▼
PostgREST → PostgreSQL (schema proj_{ulid})
```

前置：Dashboard 登录（§11.3）→ Project 登录（§11.4）。

#### 6.2.2 SDK / 程序化调用

SDK **不走用户 Session**。**Project 由 URL 中的 `{project_id}` 指定**；`Authorization` 中的 **Project API Key 仅用于鉴权**（验证调用方 + `anon` / `service` role），**不**负责解析 project 上下文（见 §11.2）。

**路径与请求头：**

```
GET /api/data/{project_id}/users?select=*
Authorization: Bearer <project_api_key>
```

**代理流程：**

```
SDK
    │
    │  URL 含 project_id；Authorization: Bearer <project_api_key>
    ▼
GET /api/data/01jcqz4sxf7k2m8n3p5r6t9vwx/users?select=*
    │
    ▼
Data API 网关 (Axum)
    │  1. 从 URL 解析 project_id (ULID)              ← project 来自 URL
    │  2. 校验 API Key（Postgres + Redis 缓存）     ← Key 只做鉴权
    │  3. 确认 Key 对该 project_id 有效 + role 足够
    │  4. 映射 → schema proj_{ulid}；Accept-Profile
    │  5. strip URL 前缀，转发至 PostgREST
    ▼
GET /users?select=*  →  PostgREST → PostgreSQL
```

MVP TypeScript SDK 见 [mvp-sdk.md](./mvp-sdk.md)。

**网关层统一处理：**

- Auth：Axum 终止
- Project 解析：**Session → Redis `project_id`**；**SDK → URL `{project_id}`**
- Schema 切换：`Accept-Profile: proj_{ulid}`（见 §11.1）
- 权限检查、审计日志、统一错误格式

**约束：**

- 外部调用方（含 SDK）**永远不知道 PostgREST 的地址**
- 客户端凭证 **不转发**至 PostgREST；PostgREST 仅接受 **Data API 网关** 的内网 service 凭证
- PostgREST 仅作为内网 CRUD 引擎；替换 PostgREST 或新增 GraphQL 时，仅改 **Data API 网关** 代理层

本地开发 compose 可能映射 PostgREST 端口（如 `:13000`）便于调试，**不属于支持的客户端接入方式**。

## 7. Dashboard Responsibilities

Dashboard 功能模块：

| Module | Primary API | Notes |
|--------|-------------|-------|
| Project | Manager API | 项目创建、配置 |
| Database | Manager API | Schema 管理 |
| Table Designer | Manager API | `CREATE TABLE` / `ALTER TABLE` |
| Column Designer | Manager API | Column 增删改 |
| Row Viewer | **Data API** `/api/data/*` | **Project Session**（Opaque Token + Redis）；见 §6.2.1 |
| File Manager | Manager API (Storage) | OpenDAL -backed |
| User Manager | Manager API | 用户与角色 |
| API Key | Manager API | 签发 Project API Key；供 SDK **鉴权** |
| Settings | Manager API | 平台配置 |

### 7.1 Table Designer vs Row Viewer

**Table Designer** 调用 **Manager API**，执行 DDL：

```
CREATE TABLE
ALTER TABLE
```

**Row Viewer** — 用户 **Project 登录**后，仅持 **Project Session Token**（Opaque Token + Redis）访问 Data API：

```
Dashboard → GET /api/data/users?…
            Authorization: Bearer <project_session_token>
            → Redis project_session:{token}
            → Data API 网关 → PostgREST → PostgreSQL
```

SDK 不走此路径；见 §6.2.2 / §11.2。

## 8. CRUD Strategy

| Layer | SELECT | INSERT | UPDATE | DELETE | DDL | Metadata | Storage | Auth |
|-------|--------|--------|--------|--------|-----|----------|---------|------|
| PostgREST | ✓ | ✓ | ✓ | ✓ | — | — | — | — |
| Manager API (Axum) | — | — | — | — | ✓ | ✓ | ✓ | ✓ |
| Data API 网关 (Axum) | — | — | — | — | — | — | — | ✓（代理层鉴权） |

Axum **不实现** CRUD。进程内分工：

- DDL（建表、改表）— **Manager API**
- Metadata — **Manager API**
- Storage — **Manager API**
- Auth — **Manager API** + **Data API 网关**
- Project Lifecycle — **Manager API**
- PostgREST 代理 — **Data API 网关**

实现职责完全分离。

## 9. Benefits

| Benefit | Detail |
|---------|--------|
| 能力一致 | Dashboard 与 SDK 共用同一套 CRUD 能力 |
| 单一实现 | CRUD 逻辑只有 PostgREST 一份实现，避免 Axum 重复维护 |
| 专注平台 | Axum 专注平台能力（DDL、Metadata、Auth、Storage） |
| 前端稳定 | 替换 PostgREST 时，仅改 **Data API 网关**；Dashboard 仍只依赖 **Manager API** |
| 安全统一 | Auth、审计、错误格式在 Axum 层集中治理 |

## 10. Storage Backend

| Backend | Scope | Notes |
|---------|-------|-------|
| OpenDAL `fs` (local) | **默认** | 应用侧本地目录；根目录 `docker compose up` 不依赖对象存储服务 |
| SeaweedFS S3 | 可选 | 独立栈：[docker/seaweedfs/docker-compose.yaml](../../docker/seaweedfs/docker-compose.yaml)；需单独启动，OpenDAL 配置为 S3 后端时接入 |

SeaweedFS **不在**根 [docker-compose.yaml](../../docker-compose.yaml) 中；仅作为 `docker/` 下的可选部署方案保留。

## 11. Project Isolation & Authentication

### 11.1 Schema-per-project

#### Project ID：ULID（不用 UUID）

`public.projects.id` 使用 **[ULID](https://github.com/ulid/spec)**（128-bit，Crockford Base32，26 字符），**不用 UUID v4**。

| 方案 | 索引表现 | 自托管复杂度 | 选用 |
|------|----------|--------------|------|
| UUID v4 | 随机 → B-tree 插入分散、页分裂多 | 低 | ✗ |
| UUID v7 | 时间有序，优于 v4 | 低 | △ |
| **ULID** | **时间前缀有序，索引局部性好** | **低（无 worker 协调）** | **✓** |
| Snowflake (i64) | 整数 PK 最紧凑 | 需 worker / machine id 配置 | △ |

**约定：**

- 生成：应用层（Rust `ulid` crate）在创建 Project 时签发
- 存储：`CHAR(26)` 或 `TEXT`，**小写** 26 字符（如 `01jcqz4sxf7k2m8n3p5r6t9vwx`）
- Project 上下文（Dashboard）：**Project Session**（Redis 内 `project_id`）；Data API URL 为 `/api/data/*`
- Project 上下文（SDK）：**URL** `/api/data/{project_id}/*`；API Key **不**承担 project 解析
- Schema 名：**`proj_{ulid}`**，与 `id` 同形，无需字符替换，可直接作 PostgreSQL unquoted identifier

每个 Project 对应一个 PostgreSQL schema：

```
PostgreSQL (indiebase-dev)
│
├── public                    ← 平台层（Axum 直连）
│   ├── projects              ← id CHAR(26) PRIMARY KEY (ULID)
│   ├── project_members
│   ├── api_keys              ← project_id, role (anon|service), key hash
│   ├── table_metadata
│   └── column_metadata
│
├── proj_01jcqz4sxf7k2m8n3p5r6t9vwx   ← 租户 CRUD
│   ├── users
│   └── …
│
└── proj_01jcqz4sxf8m9n4q6s7u0wxyz
    └── …
```

| 层 | Schema | 读写路径 |
|----|--------|----------|
| 平台 | `public` | Axum 直连（DDL 元数据、Project 生命周期） |
| 租户 CRUD | `proj_{ulid}` | Dashboard：`/api/data/*` + Session；SDK：`/api/data/{project_id}/*` + API Key |

**创建 Project 时：**

1. 生成 ULID，写入 `public.projects`
2. `CREATE SCHEMA proj_{ulid}`
3. 签发默认 **Project API Key** 对（`anon` + `service`）
4. 将新 schema 注册至 PostgREST（更新 `PGRST_DB_SCHEMAS` 或等效机制）并 **reload PostgREST**

Table Designer 的 DDL 在对应 `proj_{ulid}` 内执行；Metadata 存 `public`。

首版 **不使用 RLS** 做 project 隔离；隔离边界为 schema + 网关授权。

### 11.2 Project API Key（SDK 鉴权）

**Project API Key 不是用户登录凭证，也不负责确定 project。**

| 属性 | 说明 |
|------|------|
| 用途 | SDK / 脚本调用 Data API 时的 **鉴权凭证**（验证调用方 + `anon` / `service` role） |
| Project 上下文 | 由请求 URL `/api/data/{project_id}/*` 提供；网关校验 Key **对该 project 有效** |
| 绑定 | Key 签发时关联 `project_id` + `role`（用于校验，非 lookup 反推 project） |
| 传递 | `Authorization: Bearer <project_api_key>` |
| 持久化 | `public.api_keys` 主存；Redis 缓存 |
| 撤销 | 删库 + 删 Redis 缓存 |

Dashboard 用户访问 Data API 用 **Project Session**（§11.4），**不用 API Key**。

```
GET /api/data/{project_id}/users
Authorization: Bearer <project_api_key>
        │
        ▼
   1. project_id ← URL
   2. 校验 Key（Redis / public.api_keys）对该 project + role 有效
        │
        ▼
   proj_{ulid}  →  PostgREST
```

### 11.3 Dashboard Session（平台登录）

Dashboard **平台登录**使用 **Opaque Token + Redis**。**不使用 JWT。**

| 属性 | 说明 |
|------|------|
| 用途 | **Manager API**（Project 列表、DDL、Metadata、Key 管理、Storage） |
| 签发 | `POST /api/auth/login` |
| 撤销 | `POST /api/auth/logout` |
| Redis | `dashboard_session:{token}` → `{ user_id, exp, … }` |
| 传递 | `Authorization: Bearer <dashboard_session_token>` |

```
Manager API
Authorization: Bearer <dashboard_session_token>
        │
        ▼
   Redis  dashboard_session:{token}
        │
        ▼
   { user_id, exp }  →  平台鉴权 / project_members 校验
```

用户身份持久化在 `public.users`；Session 为 Redis 中的 opaque 指针。

### 11.4 Project Session（Project 登录 — Data API 用户凭证）

用户进入某一 Project 时 **Project 登录**，使用 **Opaque Token + Redis**（与 Dashboard 登录同一技术，**作用域为单个 project**）。**这是 Dashboard 用户访问 Data API 的唯一凭证。**

| 属性 | 说明 |
|------|------|
| 用途 | Dashboard 访问 **Data API**（Row Viewer 等） |
| 前置 | 通常需有效 **Dashboard Session**（平台已登录） |
| 签发 | `POST /api/auth/project/login`（body 含 `project_id`） |
| 撤销 | `POST /api/auth/project/logout` |
| Redis | `project_session:{token}` → `{ user_id, project_id, exp, project_role, … }` |
| 传递 | `Authorization: Bearer <project_session_token>` |

```
Data API (Dashboard)
Authorization: Bearer <project_session_token>
        │
        ▼
   Redis  project_session:{token}
        │
        ▼
   { user_id, project_id, … }  →  proj_{ulid}  →  PostgREST
```

Project Session 在 Redis 中携带 `project_id` 与用户在该 project 内的权限；**这是 Dashboard 用户侧 project 上下文的唯一来源。**

Auth **在 Axum 层完全终止**；PostgREST 不解析客户端凭证，**Data API 网关** 用固定内网 service 凭证转发。

### 11.5 凭证分离小结

| 场景 | 路径 | 凭证 | Project 从哪来 |
|------|------|------|----------------|
| 平台管理 | Manager API 路由 | Dashboard Session | — |
| Dashboard 数据 | `/api/data/*` | Project Session（Opaque + Redis） | **Redis Session** |
| SDK / 脚本 | `/api/data/{project_id}/*` | Project API Key | **URL `{project_id}`** |

**登录 / 访问流：**

```
Dashboard 登录  →  dashboard_session  →  Manager API
       │
       ▼
Project 登录    →  project_session    →  /api/data/*（project 在 Session 内）

SDK            →  /api/data/{project_id}/* + project_api_key（Key 仅鉴权）
```

### 11.6 PostgREST 代理

- Schema 切换：转发请求时注入 **`Accept-Profile: proj_{ulid}`**（写操作同理用 `Content-Profile`）
- 凭证：**Data API 网关** 持有 PostgREST service / authenticator 内网凭证
- 路径映射：Dashboard 请求 strip `/api/data`；SDK 请求 strip `/api/data/{project_id}`；余下路径与 query 透明转发
- PostgREST 兼容：转发 `Prefer`、`Range`、`Content-Type` 等 header，以支持 `@supabase/postgrest-js`（见 [mvp-sdk.md](./mvp-sdk.md)）

## 12. MVP Client SDK

TypeScript Data API SDK（`@supabase/postgrest-js` 薄封装）详见 **[mvp-sdk.md](./mvp-sdk.md)**。

## 13. Open Questions

- Data API 首版代理范围：MVP 对齐 [mvp-sdk.md](./mvp-sdk.md)（table CRUD）；RPC / 视图 / OpenAPI 二期
- PostgREST 新 schema 注册与 reload 机制（NOTIFY / SIGHUP / sidecar）
- Session TTL / 滑动续期默认值（Dashboard Session 与 Project Session 是否共用策略）

## 14. References

- Local stack: [docker-compose.yaml](../../docker-compose.yaml) — Postgres, Redis, PostgREST
- MVP TS SDK: [mvp-sdk.md](./mvp-sdk.md)
- OpenSpec config: [openspec/config.yaml](../../openspec/config.yaml)
- Product context: [README.md](../../README.md)

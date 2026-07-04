# Indiebase BaaS Platform — Architecture PRD

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase — BaaS |
| Last updated | 2026-07-04 |

## 1. Overview

Indiebase BaaS 为独立开发者和小团队提供自托管的后端能力，包括项目管理、数据库 Schema 设计、数据 CRUD、文件存储与认证。本 PRD 定义平台的核心系统架构与 API 分层策略。

**设计原则：**

- CRUD 由 PostgREST 单一实现，Axum 不重复实现数据读写
- Dashboard 用户访问 Data API：`/api/data/*` + **Project Session**（Opaque Token + Redis，project 在 Session 内）
- SDK 访问 Data API：`/api/data/{project_id}/*` + **Publishable Key**（**绑定** Project；**project 由 URL 指定**；Key **不做** project 解析）+ 可选 **App User Session**（终端用户身份，Opaque Token + Redis）
- 服务端 / 管理场景：`/api/data/{project_id}/*` 或 **Manager API** + **Secret Key**（管理员权限；严禁客户端暴露）
- **全部用户登录统一 Opaque Token + Redis**（Dashboard Session、Project Session、App User Session）；**不使用 JWT**
- **Manager API** 专指 Dashboard **管理面**（Platform API）；**Data API** 为独立 CRUD 网关；二者同属 Axum 进程，名称不混用
- PostgREST 仅内网可达，由 Axum **Data API 网关**代理

## 2. Goals

| Goal | Description |
|------|-------------|
| 职责分离 | Axum 负责平台管理（DDL、Metadata、Auth、Storage）；PostgREST 负责 CRUD |
| 能力复用 | Dashboard 数据浏览与 SDK/客户端共用同一 **Data API 网关**（Dashboard：`/api/data/*`；SDK：`/api/data/{project_id}/*`） |
| 可替换性 | 后续替换 PostgREST 或新增 GraphQL 时，仅改 Axum **Data API 网关**内部实现；**Manager API 路由**与 **Data API 对外路径**均无需变更 |
| 统一治理 | Opaque Token、Publishable / Secret Key、`project_id` → `proj_{ulid}` schema 推导、按 auth_mode **SET ROLE** + RLS、审计在 Axum 终止凭证后在 DB 求值 |

## 3. Non-Goals

- Axum 不实现 SELECT / INSERT / UPDATE / DELETE
- Dashboard、SDK、客户端不直连 PostgREST 或 PostgreSQL
- PostgREST 不对公网暴露；生产环境不 publish 端口，仅 Docker 内网或 localhost 供 Axum 代理
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
| **Redis** | 6.x | Dashboard / Project / App User Session；Publishable / Secret Key lookup 缓存 |
| **OpenDAL** | Rust crate | 文件存储（默认 local fs） |
| **SeaweedFS** | 可选，独立 compose | S3 兼容对象存储 |

本地栈：`docker compose up -d`（[docker-compose.yaml](../../docker-compose.yaml)）— Postgres、Redis、PostgREST。

### 4.3 Client SDK（MVP）

| Component | Technology | Role |
|-----------|------------|------|
| TS Data SDK | **`@supabase/postgrest-js`** + 薄封装 | MVP 客户端 SDK（Publishable Key + App User Session）；见 [mvp-sdk.md](./mvp-sdk.md) |

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
   /api/projects, /api/tables…      /api/data/*  ;  /api/data/{id}/*
   Dashboard 管理专用                  Session / Key（双路径）

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
     (Dashboard / Project / App User Session;
      Publishable / Secret Key cache)
```

### 5.1 Components

| Component | Technology | Responsibility |
|-----------|------------|----------------|
| Dashboard | Web UI | 项目管理、表设计、数据浏览、文件管理、用户与 API Key（Publishable / Secret）配置 |
| Axum API Server | Axum + sqlx + SeaQuery (Rust) | 承载 Manager API + Data API 网关 |
| Manager API | Axum 路由模块 | Platform 管理：DDL、Metadata、Auth、Storage、Project 生命周期 |
| Data API 网关 | Axum 路由模块 | `/api/data/*`（Dashboard）、`/api/data/{project_id}/*`（SDK）→ PostgREST 透明代理 |
| Project Service | Axum module | Project 生命周期、`proj_{ulid}` schema 创建与 PostgREST reload |
| Metadata Service | Axum module | Table / Column 元数据（platform schema）；设计器所需信息 |
| Storage Service | Axum + OpenDAL | 文件上传、下载、管理 |
| Redis | redis:6 | Dashboard / Project / App User Session；Publishable / Secret Key lookup 缓存 |
| PostgREST | postgrest/postgrest | 内网 REST CRUD；网关以固定 **authenticator** 连线，**每条请求**经 `db-pre-request` 按 auth_mode **SET ROLE** |
| PostgreSQL | Postgres 17 | 持久化存储（DDL + CRUD） |
| OpenDAL | Rust crate | 统一对象存储抽象；**默认 `fs`（本地目录）**；可切换 S3 等后端 |

## 6. API Architecture

**术语：**

| 名称 | 含义 | 路由前缀 | 主要调用方 |
|------|------|----------|------------|
| **Manager API** | Dashboard **管理面** API（即 **Platform API**） | `/api/projects`, `/api/tables`, `/api/auth/*`, … | Dashboard |
| **Data API** | 租户 **CRUD 网关**（PostgREST 代理） | `/api/data/*`（Dashboard）；`/api/data/{project_id}/*`（SDK） | Dashboard Row Viewer、SDK |

二者由同一 **Axum** 进程暴露，但 **Manager API ≠ Data API**；文档中「Manager API」**不**包含 `/api/data/*`。

### 6.1 Manager API（Platform API）

Dashboard **管理专用**。负责平台治理，**不**代理 PostgREST CRUD。

```
/api/auth/login
/api/auth/logout
/api/auth/project/login
/api/auth/project/logout
/api/auth/app/login
/api/auth/app/logout
/api/projects
/api/projects/{project_id}/api-keys          ← 见 §6.1.1
/api/tables
/api/columns
/api/projects/{project_id}/files             ← 见 §6.1.2、§10.1
/api/projects/{project_id}/files/{file_id}
/api/settings
```

#### 6.1.1 Manager API — API Key 管理

Dashboard Session **或** 对该 Project 有效的 **Secret Key**（Server-to-Server）均可调用下列接口。Secret Key 不得通过浏览器或移动端暴露。

| Method | Path | 说明 |
|--------|------|------|
| `GET` | `/api/projects/{project_id}/api-keys` | 列出 Key 元数据（**不含**完整 secret；见 §11.5） |
| `GET` | `/api/projects/{project_id}/api-keys/{key_id}` | 单条 Key 详情 |
| `POST` | `/api/projects/{project_id}/api-keys/{key_type}/rotate` | 轮换指定类型 Key（`publishable` \| `secret`） |
| `PATCH` | `/api/projects/{project_id}/api-keys/{key_id}` | 更新状态（如 `disabled`） |
| `DELETE` | `/api/projects/{project_id}/api-keys/{key_id}` | 删除非默认 Key（若适用；见 §11.5） |

**约束：**

- 每个 Project **始终保留**一对默认 Key（Publishable + Secret）；默认 Key **不可删除**，仅可 **轮换** 或 **禁用**（禁用后须提示恢复或轮换）
- 响应中完整 Key 明文 **仅在创建 / 轮换成功时返回一次**；后续查询仅返回前缀、类型、状态、时间戳
- Secret Key 相关写操作写入 **审计日志**（操作者、时间、IP / 调用方标识）

#### 6.1.2 Manager API — Storage（File）

文件存储 **不**经 Data API / PostgREST；统一走 **Manager API** + OpenDAL。路由、凭证与权限见 **§10.1**。

Manager API 请求头（二选一）：

```
# Dashboard 登录态
Authorization: Bearer <dashboard_session_token>

# Server-to-Server（Secret Key）
X-Indiebase-Api-Key: <secret_key>
```

Secret Key 调用 Manager API 时，网关校验 Key 类型为 `secret`、状态为 `active`，且绑定目标 `project_id`（路径参数须一致）。

职责：

- Dashboard 用户登录 / 登出（Opaque Token + Redis，见 §11.7）
- 进入 / 退出 Project 上下文（Project 登录，见 §11.8）
- App 终端用户登录 / 登出（App User Session，见 §11.9）
- 创建 Project（含自动创建 `proj_{ulid}` schema；见 §11.1）
- **API Key** 全生命周期管理（Publishable / Secret；见 §11.2、§11.5、§6.1.1）
- 创建 Table
- 修改 Column
- Metadata 读写
- Storage 操作（`/api/projects/{project_id}/files*`；见 §6.1.2、§10.1）

### 6.2 Data API（统一网关）

Axum 透明代理至内网 PostgREST，**不重复实现** CRUD 逻辑。Dashboard 与 SDK **路径不同**：用户侧 URL 不含 `project_id`（project 在 Project Session 内）；SDK 侧 URL **含** `{project_id}`（project 由路径指定，Key 只做鉴权）。

#### 6.2.1 Dashboard 用户（Row Viewer）

Dashboard 用户访问 Data API **只用 Project Session** — **Opaque Token + Redis**（见 §11.8）。**不是 API Key。**

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
    │  3. schema = proj_{project_id}（ULID 直接推导，无 lookup）
    │  4. 映射 project_role → auth_mode（owner/admin → project_operator；member → project_operator_readonly）
    │  5. member 写操作可在网关早拒（可选 fast-path）
    │  6. 注入 Accept-Profile: proj_{ulid} + X-Indiebase-Internal-Context（signed）
    │  7. 以内网 authenticator 转发 PostgREST
    ▼
PostgREST → db-pre-request → SET ROLE + RLS → PostgreSQL (proj_{ulid})
```

前置：Dashboard 登录（§11.7）→ Project 登录（§11.8）。

#### 6.2.2 SDK / 程序化调用

SDK **不走 Dashboard / Project Session**。**Project 由 URL 中的 `{project_id}` 指定**；凭证与权限分层见 §11.2、§11.3。

**客户端 SDK（Publishable Key + 可选 App User Session）：**

```
GET /api/data/{project_id}/users?select=*
X-Indiebase-Api-Key: <publishable_key>
Authorization: Bearer <app_user_session_token>    ← 用户已登录时携带；未登录可省略
```

**服务端 SDK（Secret Key — 管理员 / 绕过权限策略）：**

```
GET /api/data/{project_id}/users?select=*
X-Indiebase-Api-Key: <secret_key>
```

**代理流程（Publishable Key + App User Session）：**

```
Client SDK
    │
    │  URL 含 project_id
    │  X-Indiebase-Api-Key: <publishable_key>
    │  Authorization: Bearer <app_user_session_token>   （可选）
    ▼
GET /api/data/01jcqz4sxf7k2m8n3p5r6t9vwx/users?select=*
    │
    ▼
Data API 网关 (Axum)
    │  1. 从 URL 解析 project_id (ULID)                    ← project 来自 URL
    │  2. 校验 Publishable Key 与 URL project_id 绑定一致（Postgres + Redis 缓存）
    │  3. 若存在 app_user_session_token：Redis lookup app_user_session:{token}
    │  4. 得到 { end_user_id, project_id, role, … }；无 Session → auth_mode=anon，有 → authenticated
    │  5. schema = proj_{project_id}；Accept-Profile + Internal-Context
    │  6. strip URL 前缀，以内网 authenticator 转发 PostgREST
    ▼
GET /users?select=*  →  PostgREST → db-pre-request → SET ROLE → PostgreSQL（RLS）
```

**代理流程（Secret Key）：**

```
Server SDK
    │
    │  X-Indiebase-Api-Key: <secret_key>
    ▼
Data API 网关 (Axum)
    │  1. 校验 Secret Key（类型 secret、active、project 匹配）
    │  2. auth_mode=service；schema = proj_{project_id}；Accept-Profile + Internal-Context
    │  3. 以内网 authenticator 转发 PostgREST
    ▼
PostgREST → db-pre-request → SET ROLE service → PostgreSQL（bypass 租户 RLS）
```

#### 6.2.3 双路径路由与凭证互斥

Dashboard 与 SDK **物理上走不同 URL 前缀**，project 上下文**只从一个来源读取**，避免「Session 内 project A + URL 内 project B」混用。

**路由匹配（Axum 须按此顺序注册）：**

| 分支 | 匹配规则 | 示例 |
|------|----------|------|
| **SDK 路径** | `/api/data/{project_id}/*`，且 `{project_id}` 为 **26 字符小写 ULID** | `/api/data/01jcqz4sxf7k2m8n3p5r6t9vwx/users` |
| **Dashboard 路径** | `/api/data/*`，且第二路径段 **不是** ULID（或无第二段） | `/api/data/users`、`/api/data/rpc/foo` |

第二段形如 ULID 的请求 **一律**走 SDK 分支；不得回落到 Dashboard 分支。

**project 上下文来源（每条请求唯一）：**

| 分支 | project 来源 | 忽略 |
|------|--------------|------|
| Dashboard | Redis `project_session:{token}` → `project_id` | URL 中即使出现 ULID 形 segment 也不作 project 解析（SDK 分支已拦截） |
| SDK | URL `{project_id}` | **不**读取 Project Session / Dashboard Session |

**凭证互斥（非法组合 → `403` + 统一错误体）：**

| 分支 | 必须 | 禁止 |
|------|------|------|
| Dashboard `/api/data/*` | `Authorization: Bearer <project_session_token>` | `X-Indiebase-Api-Key`（任意 Key）；Bearer 为 App User / Dashboard Session |
| SDK `/api/data/{project_id}/*` | `X-Indiebase-Api-Key`（Publishable 或 Secret） | Project Session；Dashboard Session |

**SDK 路径一致性校验：**

1. **Publishable / Secret Key** — Key 绑定的 `project_id` **必须等于** URL `{project_id}`，否则 `403`
2. **App User Session**（可选 `Authorization`）— Redis `app_user_session:{token}` 内 `project_id` **必须等于** URL `{project_id}`，否则 `403`
3. 无 App User Session 时按 `auth_mode=anon` 继续（Key 校验已通过即可）

**Session 命名空间隔离（Redis lookup 不得混用）：**

| Bearer token 类型 | Redis 前缀 | 可用于 |
|-------------------|------------|--------|
| Project Session | `project_session:` | 仅 Dashboard `/api/data/*` |
| App User Session | `app_user_session:` | 仅 SDK `/api/data/{project_id}/*`（且须 + Publishable Key） |
| Dashboard Session | `dashboard_session:` | 仅 Manager API |

网关解析 `Authorization: Bearer` 时：**先按 URL 分支决定**允许哪种 Session；在 SDK 分支 **仅** lookup `app_user_session:`，**不得** fallback 到 `project_session:`。

**为何不会常态冲突：** Dashboard 客户端 **不**调用含 `{project_id}` 的 Data API URL；SDK 客户端 **不**持有 Project Session。混用仅可能来自错误集成或恶意请求，由上述规则 **拒绝**。

> **与旧稿差异：** 原仅描述「路径不同」，未写路由 ULID 判别与非法凭证 `403` 矩阵；实现与集成测试须对齐本节。

> **与旧稿差异：** 原 `anon` / `service` role 合并为 **Publishable Key** / **Secret Key** 两种 Key 类型；终端用户身份由 **App User Session**（Opaque Token + Redis）承载，不再由 Key role 隐含。Publishable Key **不决定**未登录用户能否访问资源，由 **权限策略** 决定（§11.3）。

MVP TypeScript SDK 见 [mvp-sdk.md](./mvp-sdk.md)。

**网关层统一处理：**

- Auth：Axum 终止（Publishable / Secret Key；Project / App User Session）
- **双路径互斥**：路由 ULID 判别 + 凭证组合校验（§6.2.3）；非法组合 `403`
- Project 解析：**Dashboard → Session `project_id`**；**SDK → URL `{project_id}`**（单源，不合并）
- Schema：`project_id`（ULID）→ **`Accept-Profile: proj_{ulid}`**（见 §11.1；**无 schema lookup 表**）
- 身份：`auth_mode` + signed `X-Indiebase-Internal-Context` → PostgREST `db-pre-request` → **SET ROLE**（见 §11.11）
- 权限：**PostgreSQL RLS**（MVP 用建表时 bootstrap 策略模板）；审计、统一错误格式

**约束：**

- 外部调用方（含 SDK）**永远不知道 PostgREST 的地址**
- 客户端凭证 **不转发**至 PostgREST；PostgREST 仅接受网关 **authenticator** + 网关签名的 Internal-Context
- **禁止**网关以 tenant `service` role 代理全部请求；每条请求须按 Session / Key 切换 DB role
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
| Row Viewer | **Data API** `/api/data/*` | **Project Session**（Opaque Token + Redis）；见 §6.2.1、§11.8 |
| File Manager | Manager API (Storage) | OpenDAL；`/api/projects/{project_id}/files*`；见 §10.1 |
| User Manager | Manager API | 平台用户（`public.users`）与 **project 成员**（`public.project_members`）；**非**租户 schema 内 App 终端用户 |
| API Key | Manager API + Dashboard UI | Publishable / Secret Key 全生命周期；见 §7.2、§11.2、§11.5 |
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

### 7.2 Dashboard — API Key 管理页

Project Settings 内提供 **API Keys** 模块，供项目管理员查看与管理 Publishable / Secret Key。

| 能力 | 要求 |
|------|------|
| 列表展示 | 每个 Project 固定展示 **Publishable Key**、**Secret Key** 两行（或卡片）；显示类型、名称、前缀（如 `ib_pub_…` / `ib_sec_…`）、状态（`active` / `disabled`）、创建时间、上次轮换时间 |
| 完整 Key 可见性 | **创建 Project** 或 **轮换** 成功后，以一次性 Modal 展示完整 Key +「复制」按钮；关闭后不可再次查看完整 Secret Key |
| Publishable Key | 标注「可公开嵌入客户端」；提供复制前缀 / 完整 Key（轮换时）；**无**「在浏览器中测试 Secret」类误导操作 |
| Secret Key | 标注「仅服务端、严禁客户端」；默认 **掩码** 显示；轮换前二次确认（说明旧 Key 将失效） |
| 轮换（Rotate） | 按 Key 类型独立操作；轮换后旧 Key 进入 **grace period**（建议 24h，可配置）内仍接受请求，便于滚动部署；grace 结束后仅新 Key 有效 |
| 禁用（Disable） | 可临时禁用某 Key；禁用后立即拒绝新请求（或可选 grace）；须提供「重新启用」或「轮换并启用」 |
| 删除 | 默认 Key **不可删除**；若未来支持附加 Key，附加 Key 可删除（首版一对默认 Key，无删除入口） |
| 权限 | 仅 Project **owner / admin** 可查看 Secret Key 元数据、执行轮换 / 禁用 |
| 审计 | 页面展示近期 Key 操作记录（轮换、禁用、谁、何时）；明细与 Manager API 审计一致 |
| 空状态 / 引导 | 新建 Project 后引导复制 Publishable Key 到客户端、Secret Key 到服务端环境变量 |

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
| 前端稳定 | 替换 PostgREST 时，仅改 **Data API 网关**内部实现；Dashboard 对外 API 路径不变（**Manager API** + **Data API** 路由均保持稳定） |
| 安全统一 | Auth、审计、错误格式在 Axum 层集中治理 |

## 10. Storage Backend

| Backend | Scope | Notes |
|---------|-------|-------|
| OpenDAL `fs` (local) | **默认** | 应用侧本地目录；根目录 `docker compose up` 不依赖对象存储服务 |
| SeaweedFS S3 | 可选 | 独立栈：[docker/seaweedfs/docker-compose.yaml](../../docker/seaweedfs/docker-compose.yaml)；需单独启动，OpenDAL 配置为 S3 后端时接入 |

SeaweedFS **不在**根 [docker-compose.yaml](../../docker-compose.yaml) 中；仅作为 `docker/` 下的可选部署方案保留。

### 10.1 Storage API 与调用方

Storage **独立于 Data API**：文件上传 / 下载 / 删除 / 列表均由 **Manager API** + **OpenDAL** 实现，**不**代理 PostgREST，**不**出现在 `/api/data/*`。

**路由（Manager API）：**

| Method | Path | 说明 |
|--------|------|------|
| `GET` | `/api/projects/{project_id}/files` | 列表（分页 / 前缀过滤） |
| `POST` | `/api/projects/{project_id}/files` | 上传（multipart 或 signed upload 初始化 — 首版 multipart） |
| `GET` | `/api/projects/{project_id}/files/{file_id}` | 下载 / 元数据 |
| `DELETE` | `/api/projects/{project_id}/files/{file_id}` | 删除 |

对象键命名空间：`{project_id}/…`（OpenDAL root 下按 project 隔离）。

**调用方与权限矩阵：**

| 场景 | API | 凭证 | MVP | 权限 |
|------|-----|------|-----|------|
| Dashboard File Manager | Manager `/api/projects/{project_id}/files*` | **Dashboard Session** | ✓ | `project_members.project_role`：`owner` / `admin` 读写删；`member` **只读**（list + download，拒绝 upload / delete） |
| 服务端自动化（S2S） | 同上 | **Secret Key** | ✓ | 全项目文件 CRUD；bypass App 用户策略；**须审计** |
| Dashboard Row Viewer | — | — | — | **不含**文件；表格 CRUD 走 Data API（§6.2.1） |
| 客户端 SDK（Storage） | — | — | **后续** | 见 [todo.md §6 客户端 Storage](./todo.md#6-客户端-storage) |
| Data API / PostgREST | — | — | — | **不支持** Storage |

**与 Data API 的边界：**

- 租户表内可存 `file_id` / URL 等 **引用**；二进制内容 **仅**经 Storage API
- Secret Key 调 Storage **不**等同于调 Data API bypass；二者均为 Manager / 网关侧独立鉴权
- Publishable Key **不能**直接调 Storage API（MVP）；避免客户端绕过 App User 身份

**SDK Storage（规划 — 见 [todo.md §6](./todo.md#6-客户端-storage)；客户端 SDK，非 Manager API）：**

| 模式 | 凭证 | 说明 |
|------|------|------|
| **Signed URL**（推荐） | Publishable Key + App User Session 向 Manager 申请短期 upload / download URL | 客户端不持有 Secret Key；权限随 App User + 网关策略 |
| **Server proxy** | 服务端 Secret Key 或 Publishable + App User Session | 服务端代传；适合 SSR |

首版 [mvp-sdk.md](./mvp-sdk.md) **不含** Storage；App 文件能力由开发者自建服务端或等二期 SDK。

**MVP 验收（Storage）：**

- Dashboard File Manager：三种 `project_role` 行为符合上表
- Secret Key：S2S 上传 / 下载 / 删除 + 审计
- 无 Publishable Key 直连接口

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
- Project 上下文（SDK）：**URL** `/api/data/{project_id}/*`；Publishable / Secret Key **绑定** `project_id`，**不承担** project 解析（网关校验 Key 与 URL 一致）
- Schema 名：**`proj_{ulid}`**，与 `id` 同形，无需字符替换，可直接作 PostgreSQL unquoted identifier

每个 Project 对应一个 PostgreSQL schema：

```
PostgreSQL (indiebase-dev)
│
├── public                    ← 平台层（Axum 直连）
│   ├── projects              ← id CHAR(26) PRIMARY KEY (ULID)
│   ├── project_members
│   ├── api_keys              ← project_id, key_type (publishable|secret), key hash, status
│   ├── table_metadata        ← 含 RLS bootstrap 字段如 allow_anon_read
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
| 租户 CRUD | `proj_{ulid}` | Dashboard：`/api/data/*` + Session；SDK：`/api/data/{project_id}/*` + Publishable / Secret Key |

**创建 Project 时：**

1. 生成 ULID，写入 `public.projects`
2. `CREATE SCHEMA proj_{ulid}`
3. 创建租户 DB roles 并授权：`anon`、`authenticated`、`service`（BYPASSRLS）、`project_operator`、`project_operator_readonly`（见 §11.11）
4. 签发默认 **API Key** 对（**Publishable** + **Secret**；见 §11.2）
5. 将新 schema 注册至 PostgREST（更新 `PGRST_DB_SCHEMAS` 或等效机制）并 **reload PostgREST**

Table Designer 的 DDL 在对应 `proj_{ulid}` 内执行；Metadata 存 `public`。建表时 Axum **同步应用 MVP bootstrap RLS 策略**（见 §11.3）。

首版 **project 隔离** 靠 schema + `Accept-Profile`；**表级 / 行级访问控制** 统一经 **按 auth_mode SET ROLE + PostgreSQL RLS** 求值（见 §11.3、§11.11）。

### 11.2 API Key 模型

每个 Project **默认自动生成一对 API Key**，类型固定为 **Publishable** 与 **Secret**（对齐 Supabase `anon` / `service_role` 语义，术语统一如下）。

| 类型 | 前缀（建议） | 定位 | 可用于 |
|------|--------------|------|--------|
| **Publishable Key** | `ib_pub_` | **绑定** Project；**非**管理员凭证；可公开嵌入客户端 | Data API（`/api/data/{project_id}/*`）；**不可**直调 Storage API（MVP）；须配合权限策略 |
| **Secret Key** | `ib_sec_` | 可信服务端专用；**严禁**出现在客户端 | Data API（管理员上下文）；**Manager API**（含 Storage S2S）；Server-to-Server |

**共同属性：**

| 属性 | 说明 |
|------|------|
| 绑定 | 签发时关联 `project_id` + `key_type`（`publishable` \| `secret`） |
| Project 上下文 | SDK 侧由 URL `/api/data/{project_id}/*` 提供；网关校验 Key **对该 project 有效** |
| 传递 | `X-Indiebase-Api-Key: <key>`（**不**放入 `Authorization`；见下「HTTP 请求头约定」） |
| 持久化 | `public.api_keys` 存 **hash**（不存明文）；Redis 缓存 lookup |
| 与用户凭证关系 | Key **不是**终端用户登录凭证；用户身份由 **App User Session**（SDK）或 **Project Session**（Dashboard）单独承载，走 `Authorization: Bearer`（RFC 6750） |

**HTTP 请求头约定：**

| 头 | 用途 | 典型值 |
|----|------|--------|
| `Authorization` | **用户 / Session 身份**（RFC 6750 Bearer Token） | Dashboard / Project / App User Session token |
| `X-Indiebase-Api-Key` | **Project API Key**（应用凭证，与用户身份分头传递） | Publishable Key 或 Secret Key |

SDK 双凭证场景：`X-Indiebase-Api-Key` 始终携带 Publishable Key；用户登录后 **额外** 在 `Authorization` 携带 App User Session。**禁止**将 API Key 放入 `Authorization` 以免挤占用户 token 的标准位置。

**Publishable Key 行为：**

- 用于 Web、React Native、iOS、Android、Desktop 等客户端 SDK 初始化
- 可以公开放在客户端代码或配置中
- 所有 Data API 请求在 Key 校验通过后，还须经过 **权限策略** 求值
- 未登录用户（无 App User Session）能否读写的 **唯一依据** 是权限策略，**不是** Publishable Key 本身

**Secret Key 行为：**

- 仅部署在可信服务端（环境变量、密钥管理服务）
- 调用 Data API 时注入 `auth_mode=service` → PostgREST `SET ROLE service`，**bypass 租户 RLS**
- 可调用 Manager API 执行项目管理、DDL、Key 轮换等（见 §6.1.1）
- 泄露视为 **严重安全事件**：须立即轮换并审计

Dashboard 用户访问 Data API 用 **Project Session**（§11.8），**不用 API Key**。

```
# 客户端
GET /api/data/{project_id}/users
X-Indiebase-Api-Key: <publishable_key>
Authorization: Bearer <app_user_session_token>   ← 可选

# 服务端（Data API）
GET /api/data/{project_id}/users
X-Indiebase-Api-Key: <secret_key>

# 服务端（Manager API）
POST /api/projects/{project_id}/api-keys/publishable/rotate
X-Indiebase-Api-Key: <secret_key>
```

> **与旧稿差异：** 原 `public.api_keys.role (anon|service)` 改为 `key_type (publishable|secret)`；`anon` 能力由 Publishable Key + App User Session + 策略承担；`service` 能力由 Secret Key 承担。

### 11.3 权限模型

访问控制分 **三层**，职责不重叠：

| 层 | 凭证 | 作用 |
|----|------|------|
| **Project 标识** | Publishable Key（或 Secret Key 校验 project 绑定） | 证明请求指向合法 Project；Publishable Key **不授予**管理员能力 |
| **用户身份** | App User Session（SDK）；Project Session（Dashboard Row Viewer） | SDK：租户 **App 用户**；Row Viewer：**平台成员** + `project_role` |
| **授权** | **PostgreSQL RLS** + 租户 DB role（`SET ROLE`） | 网关解析 `auth_mode` 并注入 Internal-Context；**权限在 DB 求值** |

**统一 Data API 权限链路（所有路径）：**

1. 网关校验凭证（Key / Session）→ 解析 `project_id` → `schema = proj_{project_id}`
2. 映射 **`auth_mode`**（见 §11.11 表）
3. 构造 signed `X-Indiebase-Internal-Context`（含 `auth_mode`、`user_id`、`project_id` 等）
4. 注入 `Accept-Profile: proj_{ulid}`，以内网 **authenticator** 转发 PostgREST
5. PostgREST **`db-pre-request`**：`SET LOCAL app.*` + **`SET ROLE`** → RLS 求值

**Publishable Key + App User Session（典型客户端）：**

1. 网关校验 Publishable Key → 确认 Key 与 URL 中的 `project_id` 绑定一致
2. 若带 `app_user_session_token` → Redis lookup → `auth_mode=authenticated`，注入 `app.user_id = end_user_id`
3. 无 Session → `auth_mode=anon`
4. PostgREST `SET ROLE` 对应租户 role → **RLS** 裁决读写

**Publishable Key、无 Session（匿名）：**

- `auth_mode=anon` → `SET ROLE` 租户 `anon` role
- 是否允许 `SELECT` / 写 **完全由 RLS** 定义（MVP bootstrap 见下节）

**Secret Key：**

- `auth_mode=service` → `SET ROLE` 租户 `service` role（BYPASSRLS）
- 仍受 **Manager API** 平台级鉴权（Key 须 active、project 匹配）
- 所有 Secret Key 请求 **必须审计**

**Dashboard Project Session（概要）：**

- 平台成员身份 + `project_role`（owner / admin / member），来自 `public.project_members`
- 与 SDK 终端用户 App User Session **命名空间隔离**（平台成员 ≠ 租户 schema 内 `users` 等 App 用户）
- Row Viewer 使用 **`project_operator` / `project_operator_readonly` DB role**，**不**走 App 用户 `anon` / `authenticated` 语义

#### Dashboard Row Viewer 权限

Row Viewer 供 **平台成员** 调试、浏览、编辑租户数据，**不**模拟 App 终端用户身份。

| `project_role` | `auth_mode` | DB role | 能力 |
|----------------|-------------|---------|------|
| **owner** | `project_operator` | `project_operator` | 租户 schema **全表 CRUD**；**跳过** App 用户 RLS |
| **admin** | `project_operator` | `project_operator` | 同 owner |
| **member** | `project_operator_readonly` | `project_operator_readonly` | **仅 SELECT**；写操作由 DB role + 可选网关 fast-path 拒绝 |

**规则：**

- Project Session 路径 **永不**读取租户 `users` 表来识别操作者；操作者始终是 `public.users` + `project_members.project_role`
- owner / admin 的「全表 CRUD」是 **Dashboard 运维能力**，与 Secret Key 数据面能力等价但凭证为 Project Session（须审计）
- member 只读由 **`project_operator_readonly` role 权限** 约束（网关可对写操作早拒以降低无效请求）
- 二期可选：Row Viewer「以某 App 用户身份查看」（impersonation → `auth_mode=authenticated` + 指定 `app.user_id`）— **首版不做**

#### MVP bootstrap RLS（建表时自动应用）

MVP **不要求** Policy 编辑器 UI，但 **要求** `db-pre-request` + `SET ROLE` 从首版上线。Manager API 创建表时，Axum 同步写入 **bootstrap RLS 策略模板**：

| auth_mode / role | MVP 默认策略 |
|------------------|--------------|
| **anon** | **默认拒绝** 读与写；`public.table_metadata.allow_anon_read=true` 的表 **仅允许 SELECT** |
| **authenticated** | **默认允许** 租户表 CRUD（后续见 [todo.md](./todo.md) Policy / ABAC） |
| **project_operator** | bypass App 用户 RLS（运维读写） |
| **project_operator_readonly** | 仅 SELECT |
| **service** | BYPASSRLS（Secret Key） |

**`allow_anon_read`（已决）：**

- **默认：拒绝** — 无 App User Session 时，未 opt-in 的表 anon **不可读、不可写**
- **Opt-in** — `public.table_metadata.allow_anon_read`；仅显式开启的表 anon 可 `SELECT`
- **无**「默认 anon 可读全库」；与 Publishable Key 不隐含匿名全开一致
- Manager API 提供按表切换（Table 设计器或 Settings；UI 可二期，**API / 元数据首版须有**）

**MVP 验收：**

- 无 App User Session：无 `allow_anon_read` 的表 anon 返回拒绝；开启后仅 `SELECT` 允许
- 有 App User Session（`authenticated`）：可对租户表 CRUD（**RLS bootstrap 默认允许**）
- Project Session owner/admin：全表 CRUD；member：只读
- Secret Key：`service` role bypass
- 与 [mvp-sdk.md](./mvp-sdk.md) 集成测试对齐

**后续演进：** 见 **[todo.md](./todo.md)**（多平台客户端 SDK、per-project ABAC、Policy 编辑器等）。

### 11.4 安全设计

| 主题 | 要求 |
|------|------|
| Key 存储 | 数据库只存 **hash + prefix**；明文仅创建 / 轮换时返回一次 |
| 传输 | 生产环境 **HTTPS**；禁止 Query String 传 Key |
| 客户端暴露 | Publishable Key **允许**公开；Secret Key **禁止**出现在前端 bundle、移动端二进制、公开仓库 |
| Session | 全部 Session（Dashboard / Project / App User）均为 Opaque Token + Redis；TTL / 滑动续期 / 登出即删 Redis；**分支互斥**见 §6.2.3（SDK：App User Session `project_id` 须与 URL 一致；Dashboard：仅 Project Session） |
| 轮换 | 支持 grace period，降低部署窗口风险（§11.5） |
| 禁用 | 立即或 grace 后拒绝；配合 WAF / 速率限制降低暴力尝试 |
| 审计 | Secret Key 使用、Manager API 写操作、Key 轮换 / 禁用 **必记** |
| 最小权限 | 客户端 SDK **仅**初始化 Publishable Key；管理操作 **仅** Secret Key + 服务端 |
| PostgREST 隔离 | 客户端凭证 **不转发**；PostgREST 仅接受网关 **authenticator** + signed Internal-Context；**每条请求 SET ROLE** |

### 11.5 API Key 生命周期

| 阶段 | 行为 |
|------|------|
| **创建** | Project 创建时 **自动** 生成 Publishable + Secret 各一；写入 `public.api_keys`；Dashboard 一次性展示明文 |
| **查看** | 列表 / 详情 API 与 Dashboard 仅返回 prefix、type、status、timestamps；**不回显**完整 Secret |
| **轮换（Rotate）** | 按 `key_type` 独立轮换；生成新 Key → 旧 Key 标记 `rotating` → grace period 后 `revoked`；新 Key 一次性展示 |
| **禁用（Disable）** | 状态 `disabled`；网关拒绝（或 grace 可配置）；可 `active` 恢复 **或** 直接轮换 |
| **删除** | 默认 Key **不可删**；首版每 Project 仅一对 Key，**无删除路径**；若未来支持附加 Key，附加项可 `DELETE` |

**状态机：**

```
active ──rotate──► rotating (old) + active (new)
   │                      │
   │                      └── grace expired ──► revoked
   │
   └── disable ──► disabled ── enable ──► active
                              └── rotate ──► (new active)
```

**Redis 缓存：** 创建 / 轮换 / 禁用 / 删除时 **立即失效** 对应 Key 的 lookup 缓存。

### 11.6 SDK 初始化

**客户端 SDK**（Web / Mobile / Desktop）：

| 参数 | 说明 |
|------|------|
| `projectUrl` | 平台根 URL，如 `https://indiebase.example.com` |
| `projectId` | ULID |
| `publishableKey` | Publishable Key |

用户通过 Project Auth 登录后，SDK **自动**在 Data API 请求上附加 `Authorization: Bearer <app_user_session_token>`（`X-Indiebase-Api-Key` 头始终携带 Publishable Key）；登出后移除 `Authorization`（删 Redis Session）。所有 CRUD 经权限策略校验。

**服务端 SDK**：

| 参数 | 说明 |
|------|------|
| `projectUrl` | 同上 |
| `projectId` | ULID |
| `apiKey` | **Publishable Key** 或 **Secret Key** |

| Key 类型 | 能力 |
|----------|------|
| Publishable Key | Data API + App User Session 代表用户；适合 SSR 代表用户请求 |
| Secret Key | Data API 管理员上下文 + Manager API 管理接口 |

MVP TypeScript 客户端 SDK 见 [mvp-sdk.md](./mvp-sdk.md)。

### 11.7 Dashboard Session（平台登录）

Dashboard **平台登录**使用 **Opaque Token + Redis**。

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

### 11.8 Project Session（Project 登录 — Data API 用户凭证）

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

Auth **在 Axum 层完全终止**；PostgREST 不解析客户端凭证。网关以固定 **authenticator** 连线 PostgREST，并按 Session / Key 注入 Internal-Context → **`SET ROLE`**（§11.11）。

### 11.9 App User Session（SDK 终端用户 — Data API 用户凭证）

App 终端用户（Project 租户 schema 内的 `users` 等）通过 **Project Auth** 登录，使用 **Opaque Token + Redis**（与 Dashboard / Project Session **同一技术**，**作用域为单个 project 的终端用户**）。**这是 SDK 客户端访问 Data API 时携带的用户身份凭证。**

| 属性 | 说明 |
|------|------|
| 用途 | SDK 客户端访问 **Data API**（已登录终端用户的 CRUD） |
| 前置 | 请求须同时携带有效 **Publishable Key**（`X-Indiebase-Api-Key` 头） |
| 签发 | `POST /api/auth/app/login`（或等价 Project Auth 路由；body 含 `project_id` + 凭证） |
| 撤销 | `POST /api/auth/app/logout`（删 Redis）；或 TTL 过期 |
| Redis | `app_user_session:{token}` → `{ end_user_id, project_id, role, exp, … }` |
| 传递 | `Authorization: Bearer <app_user_session_token>`（RFC 6750；与 `X-Indiebase-Api-Key` 分头传递） |

```
Data API (Client SDK)
X-Indiebase-Api-Key: <publishable_key>
Authorization: Bearer <app_user_session_token>
        │
        ▼
   Redis  app_user_session:{token}
        │
        ▼
   { end_user_id, project_id, role, … }  →  auth_mode=authenticated  →  SET ROLE  →  PostgREST
```

未携带 App User Session 时，网关注入 `auth_mode=anon` → PostgREST `SET ROLE` 租户 anon role。

### 11.10 凭证分离小结

| 场景 | 路径 | 凭证 | Project 从哪来 | 权限 |
|------|------|------|----------------|------|
| 平台管理（Dashboard） | Manager API | Dashboard Session | — | `project_members` 角色 |
| 平台管理（S2S） | Manager API | Secret Key | URL `project_id` | Key 绑定 project |
| Dashboard 数据 | `/api/data/*` | Project Session | **Redis Session** | `project_operator` / `project_operator_readonly`（§11.3） |
| 客户端 SDK | `/api/data/{project_id}/*` | Publishable Key + 可选 App User Session | **URL `{project_id}`** | `anon` / `authenticated` + RLS bootstrap |
| 服务端 SDK（用户态） | `/api/data/{project_id}/*` | Publishable Key + App User Session | **URL `{project_id}`** | 同 SDK 客户端 |
| 服务端 SDK（管理员） | `/api/data/{project_id}/*` | Secret Key | **URL `{project_id}`** | `service` role（bypass RLS） |
| Dashboard 文件 | Manager `/api/projects/{project_id}/files*` | Dashboard Session | URL `project_id` | `project_role`（§10.1） |
| 服务端 Storage（S2S） | Manager `/api/projects/{project_id}/files*` | Secret Key | URL `project_id` | 全 CRUD + 审计 |

**HTTP 请求头（§11.2）：** Session 类凭证 → `Authorization: Bearer`；API Key → `X-Indiebase-Api-Key`。客户端 SDK 双凭证 = `X-Indiebase-Api-Key`（Publishable Key）+ `Authorization`（App User Session，登录后）。

**登录 / 访问流：**

```
Dashboard 登录  →  dashboard_session  →  Manager API
       │
       ▼
Project 登录    →  project_session    →  /api/data/*（project 在 Session 内）

App 终端用户    →  Project Auth 登录  →  app_user_session（Opaque Token + Redis）
       │
       ▼
Client SDK      →  X-Indiebase-Api-Key + Authorization（登录后）  →  /api/data/{project_id}/*（策略裁决）

Server SDK      →  X-Indiebase-Api-Key（Secret Key）               →  /api/data/{project_id}/* 或 Manager API（管理员）
```

### 11.11 PostgREST 代理

#### 凭证与 DB role 两层模型

| 层 | 名称 | 说明 |
|----|------|------|
| **连线凭证** | PostgREST **authenticator** | 网关 → PostgREST 固定内网 JWT / secret；**仅**用于建立 PostgREST 连接，**不代表**终端用户权限 |
| **请求权限** | 租户 **DB role** | 每条请求按 `auth_mode` 在 `db-pre-request` 中 **`SET ROLE`** |

**Schema 推导（无 lookup）：** `project_id`（ULID）→ `Accept-Profile: proj_{project_id}`

**转发约定：**

- Schema：注入 **`Accept-Profile: proj_{ulid}`**（写操作同理 `Content-Profile`）
- 身份：signed **`X-Indiebase-Internal-Context`**（网关生成；客户端不可设）
- 连线：**authenticator** Bearer（客户端 Key / Session **不**转发）
- 路径：Dashboard strip `/api/data`；SDK strip `/api/data/{project_id}`；余下 path / query 透明转发
- 兼容：转发 `Prefer`、`Range`、`Content-Type` 等，以支持 `@supabase/postgrest-js`（见 [mvp-sdk.md](./mvp-sdk.md)）

#### PostgREST 身份注入（db-pre-request）

客户端凭证在 **Axum Data API 网关**终止。PostgREST 仅见 **authenticator** + 网关签名的 **Internal-Context**。

**转发链路：**

```
Client  →  Data API 网关（解析 Key / Session → auth_mode）
              │
              ├─ project_id → schema proj_{ulid}；Accept-Profile
              ├─ 构造 Internal-Context（auth_mode、user_id、project_id、project_role…）
              ├─ Authorization: Bearer <postgrest_authenticator_secret>
              ├─ X-Indiebase-Internal-Context: <gateway-signed payload>
              ▼
         PostgREST  →  db-pre-request（PL/pgSQL，全局绑定）
              │
              ├─ 校验签名；解析 Internal-Context
              ├─ SET LOCAL app.user_id / app.role / app.project_id / app.auth_mode
              ├─ SET ROLE <tenant role per auth_mode>
              ▼
         PostgreSQL  proj_{ulid}.*   （RLS 引用 current_setting('app.*', true)）
```

**`auth_mode` 映射（网关 → DB role）：**

| 来源 | `auth_mode` | `SET ROLE` | 说明 |
|------|-------------|------------|------|
| Publishable Key，无 App User Session | `anon` | 租户 `anon` | RLS bootstrap / 自定义策略 |
| Publishable Key + App User Session | `authenticated` | 租户 `authenticated` | 注入 `app.user_id = end_user_id` |
| Project Session，owner / admin | `project_operator` | 租户 `project_operator` | bypass App 用户 RLS |
| Project Session，member | `project_operator_readonly` | 租户 `project_operator_readonly` | 仅 SELECT |
| Secret Key | `service` | 租户 `service` | BYPASSRLS；须审计 |

**租户 DB roles（每个 `proj_{ulid}` 创建时 provisioning）：**

| Role | 属性 | 用途 |
|------|------|------|
| `anon` | 最小权限 | 匿名 SDK 请求 |
| `authenticated` | 普通租户用户 | App User Session |
| `project_operator` | 高权限；跳过 App RLS | Dashboard owner / admin |
| `project_operator_readonly` | SELECT only | Dashboard member |
| `service` | BYPASSRLS | Secret Key |

（实际 role 名可带 schema 前缀或置于 `proj_{ulid}` 下，实现时统一命名规范即可。）

**实现要点：**

- `db-pre-request` **MVP 即启用**；禁止「全请求 service role 转发」模式
- 仅信任网关 IP / mTLS 与 signed Internal-Context；忽略客户端同名 header
- `allow_anon_read` 变更时，Manager API **同步更新**对应表 RLS bootstrap 策略
- Secret Key / `service`：**必须审计**；`project_operator` 写操作建议审计


### 11.12 架构、认证与权限流程影响

**产品架构：**

| 区域 | 变更 |
|------|------|
| Data API 网关 | 增加 Key 类型分支（publishable / secret）、App User Session lookup、策略上下文注入 |
| Manager API | 支持 Secret Key 作为 Dashboard Session 的 S2S 替代凭证；扩展 Key CRUD |
| Auth 模块 | 新增 **Project Auth**（App User Session 签发 / 吊销）；与 Dashboard / Project Session **同一 Opaque Token 技术** |
| `public.api_keys` | `role` → `key_type`；增加 `status`、轮换元数据 |
| Dashboard | 新增 API Keys 管理页（§7.2） |
| SDK 产品线 | **客户端 SDK**（Publishable + Data API）；多平台见 [todo.md §5](./todo.md#5-多平台客户端-sdk)；**暂不做**官方 Manager / Secret Key SDK |

**认证流程（SDK 终端用户 — 新增）：**

```
App 注册 / 登录  →  POST /api/auth/app/login（Project Auth）
       │
       ▼
返回 app_user_session_token（Opaque Token；载荷存 Redis）
       │
       ▼
客户端 SDK 存储 token  →  每次 Data API 请求：X-Indiebase-Api-Key（Publishable Key）+ Authorization（App User Session，登录后）
```

Dashboard、Project、App User 登录 **统一** Opaque Token + Redis；**不使用 JWT**。

**权限流程（Data API — 统一 SET ROLE）：**

```
请求进入 Data API 网关
    │
    ├─ 解析 project_id → proj_{ulid}；Accept-Profile
    ├─ 校验 Key / Session → auth_mode + Internal-Context
    │
    ├─ Secret Key ──────────────► service
    ├─ Project Session ───────────► project_operator | project_operator_readonly
    └─ Publishable Key ─────────► anon | authenticated（视 App User Session）
              │
              ▼
         authenticator 转发 PostgREST
              │
              ▼
         db-pre-request → SET LOCAL app.* → SET ROLE → RLS
```

**与 Supabase 对齐点：** Publishable ≈ anon key；Secret ≈ service_role key；RLS + role 切换决定数据访问。**差异：** 终端用户身份用 **Opaque Token + Redis**；PostgREST **不对公网**；role 切换经 **网关 Internal-Context + db-pre-request**，非客户端 JWT claim。

## 12. MVP 切分

Phase 0–5 里程碑、验收标准与 OpenSpec change 建议见 **[mvp-phases.md](./mvp-phases.md)**。

## 13. MVP Client SDK

TypeScript Data API SDK（`@supabase/postgrest-js` 薄封装）详见 **[mvp-sdk.md](./mvp-sdk.md)**。

## 14. Open Questions

- Data API 首版代理范围：MVP 对齐 [mvp-sdk.md](./mvp-sdk.md)（table CRUD）；RPC / 视图 / OpenAPI 二期
- PostgREST 新 schema 注册与 reload 机制（NOTIFY / SIGHUP / sidecar）
- Session TTL / 滑动续期默认值（Dashboard / Project / App User Session 是否共用策略）
- Project Auth 登录端点路径与 App User Session TTL
- API Key 轮换 grace period 默认时长（建议 24h）
- `X-Indiebase-Internal-Context` 签名格式（HMAC vs JWT）与 tenant role 命名 / provisioning 脚本
- bootstrap RLS 模板 SQL 与 `allow_anon_read` 变更时的策略同步事务
- Row Viewer impersonation — 见 [todo.md §4](./todo.md#4-row-viewer-impersonation)
- SDK Storage（客户端）— 见 [todo.md §6](./todo.md#6-客户端-storage)
- ABAC / Policy DSL / Compiler — 见 [todo.md §1–§2](./todo.md#1-per-project-abac)

## 15. References

- Local stack: [docker-compose.yaml](../../docker-compose.yaml) — Postgres, Redis, PostgREST
- MVP phases: [mvp-phases.md](./mvp-phases.md)
- MVP TS SDK: [mvp-sdk.md](./mvp-sdk.md)
- 后续实现 Todo: [todo.md](./todo.md)
- OpenSpec config: [openspec/config.yaml](../../openspec/config.yaml)
- Product context: [README.md](../../README.md)

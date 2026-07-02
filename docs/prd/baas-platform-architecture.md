# Indiebase BaaS Platform — Architecture PRD

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase Community — BaaS |
| Last updated | 2026-07-02 |

## 1. Overview

Indiebase BaaS 为独立开发者和小团队提供自托管的后端能力，包括项目管理、数据库 Schema 设计、数据 CRUD、文件存储与认证。本 PRD 定义平台的核心系统架构与 API 分层策略。

**设计原则：**

- CRUD 由 PostgREST 单一实现，Axum 不重复实现数据读写
- Dashboard 与 SDK 共用同一套 Data API
- Dashboard 不直接感知 PostgREST 地址，统一经 Dashboard Manager API 代理

## 2. Goals

| Goal | Description |
|------|-------------|
| 职责分离 | Axum 负责平台管理（DDL、Metadata、Auth、Storage）；PostgREST 负责 CRUD |
| 能力复用 | Dashboard 数据浏览与 SDK/客户端使用同一 PostgREST 层 |
| 可替换性 | 后续替换 PostgREST 或新增 GraphQL 时，仅改 Dashboard Manager API，Dashboard 无需变更 |
| 统一治理 | JWT、API Key、search_path、权限、审计、错误格式在 Manager API 层统一处理 |

## 3. Non-Goals

- Axum 不实现 SELECT / INSERT / UPDATE / DELETE
- Dashboard 不直连 PostgREST 或 PostgreSQL
- 本 PRD 不包含 Financial Services 模块细节
- 本 PRD 不包含 Dashboard 前端 UI 规范

## 4. System Architecture

```
                          Dashboard

                              │
                              ▼

                  Dashboard Manager API
                         (Axum)

                              │
          ┌───────────────────┼──────────────────┐
          │                   │                  │
          ▼                   ▼                  ▼

      Project Service    Metadata Service   Storage Service

          │                                      │
          ▼                                      ▼

     PostgreSQL DDL                        OpenDAL

          │
          ▼

      PostgREST
          │
          ▼
     PostgreSQL CRUD
```

### 4.1 Components

| Component | Technology | Responsibility |
|-----------|------------|----------------|
| Dashboard | Web UI | 项目管理、表设计、数据浏览、文件管理、用户与 API Key 配置 |
| Dashboard Manager API | Axum (Rust) | 平台管理 API、DDL、Metadata、Auth、Storage、PostgREST 代理 |
| Project Service | Axum module | Project 生命周期、Schema 隔离 |
| Metadata Service | Axum module | Table / Column 元数据、设计器所需信息 |
| Storage Service | Axum + OpenDAL | 文件上传、下载、管理 |
| PostgREST | postgrest/postgrest | 自动生成 REST CRUD |
| PostgreSQL | Postgres 17 | 持久化存储（DDL + CRUD） |
| OpenDAL | Rust crate | 统一对象存储抽象（SeaweedFS S3 等） |

## 5. API Architecture

系统提供两层 API。

### 5.1 Dashboard Manager API（Platform API）

负责平台管理，路由示例：

```
/api/projects
/api/tables
/api/columns
/api/files
/api/settings
```

职责：

- 创建 Project
- 创建 Schema
- 创建 Table
- 修改 Column
- Metadata 读写
- Storage 操作
- Auth（JWT / API Key）
- PostgREST 代理（数据查询）

**数据查询路径** — Dashboard Manager API 不直接查询 PostgreSQL，而是调用 PostgREST：

```
Dashboard
    │
    ▼
GET /api/tables/{table}/rows
    │
    ▼
Dashboard Manager API
    │
    ▼
PostgREST
    │
    ▼
PostgreSQL
```

**代理示例：**

```
Dashboard
    │
    ▼
GET /api/data/users
    │
    ▼
Dashboard Manager API
    │  (inject JWT / API Key, set search_path, auth check, audit)
    ▼
GET /users?select=*
    │
    ▼
PostgREST
    │
    ▼
PostgreSQL
```

Dashboard Manager API 在代理层可执行：

- 注入 JWT
- 注入 API Key
- 设置 `search_path`（Project / Schema 隔离）
- 权限检查
- 审计日志
- 统一错误格式

Dashboard **永远不知道 PostgREST 的地址**。

### 5.2 Data API（CRUD API）

Data API 由 PostgREST 提供，供 SDK、客户端及 Dashboard 数据浏览复用。

示例：

```
GET    /rest/users
POST   /rest/users
PATCH  /rest/users?id=eq.1
DELETE /rest/users?id=eq.1
```

外部 SDK / 客户端可直接访问 Data API（经 Auth 网关）；Dashboard 经 Manager API 代理访问同一能力。

## 6. Dashboard Responsibilities

Dashboard 功能模块：

| Module | Primary API | Notes |
|--------|-------------|-------|
| Project | Manager API | 项目创建、配置 |
| Database | Manager API | Schema 管理 |
| Table Designer | Manager API | `CREATE TABLE` / `ALTER TABLE` |
| Column Designer | Manager API | Column 增删改 |
| Row Viewer | Manager API → PostgREST | 数据浏览、分页、筛选、排序 |
| File Manager | Manager API (Storage) | OpenDAL -backed |
| User Manager | Manager API | 用户与角色 |
| API Key | Manager API | 客户端凭证 |
| Settings | Manager API | 平台配置 |

### 6.1 Table Designer vs Row Viewer

**Table Designer** 调用 Dashboard Manager API，执行 DDL：

```
CREATE TABLE
ALTER TABLE
```

**Row Viewer** 调用 Dashboard Manager API，经 PostgREST 访问数据：

```
Dashboard Manager API → PostgREST → PostgreSQL
```

因此，Dashboard **所有数据浏览、分页、筛选、排序** 全部复用 PostgREST，与 SDK 行为一致。

## 7. CRUD Strategy

| Layer | SELECT | INSERT | UPDATE | DELETE | DDL | Metadata | Storage | Auth |
|-------|--------|--------|--------|--------|-----|----------|---------|------|
| PostgREST | ✓ | ✓ | ✓ | ✓ | — | — | — | — |
| Dashboard Manager API (Axum) | — | — | — | — | ✓ | ✓ | ✓ | ✓ |

Axum **不实现** CRUD；只负责：

- DDL（建表、改表）
- Metadata
- Storage
- Auth
- Project Lifecycle
- PostgREST 代理

实现职责完全分离。

## 8. Benefits

| Benefit | Detail |
|---------|--------|
| 能力一致 | Dashboard 与 SDK 共用同一套 CRUD 能力 |
| 单一实现 | CRUD 逻辑只有 PostgREST 一份实现，避免 Axum 重复维护 |
| 专注平台 | Axum 专注平台能力（DDL、Metadata、Auth、Storage） |
| 前端稳定 | 替换 PostgREST 或新增 GraphQL 时，只需修改 Dashboard Manager API，Dashboard 无需改动 |
| 安全统一 | Auth、审计、错误格式在 Manager API 层集中治理 |

## 9. Open Questions

- Data API 对外暴露路径：直连 PostgREST（`/rest/*`）还是统一经 Axum 网关（`/api/data/*`）？
- Project 级 Schema 隔离策略：database-per-project vs schema-per-project vs RLS？
- PostgREST role / JWT claim 与 Manager API Auth 的映射规则
- OpenDAL 后端默认选型（SeaweedFS vs 其他）

## 10. References

- Local stack: [docker-compose.yaml](../../docker-compose.yaml) — Postgres, Redis, PostgREST
- OpenSpec config: [openspec/config.yaml](../../openspec/config.yaml)
- Product context: [README.md](../../README.md)

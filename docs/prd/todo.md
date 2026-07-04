# Indiebase BaaS — 后续实现 Todo

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase — BaaS |
| Last updated | 2026-07-04 |
| Parent | [baas-platform-architecture.md](./baas-platform-architecture.md) |
| MVP scope | [mvp-sdk.md](./mvp-sdk.md) |

[MVP 架构](./baas-platform-architecture.md) 交付 bootstrap RLS + `SET ROLE`（§11.3、§11.11）。**本文件**记录 MVP 之后按依赖顺序推进的能力；实现状态在此维护。

**客户端 SDK 边界（全平台一致）：**

- **仅** Data API：`/api/data/{project_id}/*` + **Publishable Key** + 可选 **App User Session**
- **不含** Manager API、**不含** Secret Key、**不含** Dashboard Session
- 服务端 / S2S（Secret Key、DDL、Storage 管理）**不在本 Todo**；需要时由开发者直连 Manager API 或自建后端

## 总览

| # | 能力 | 概要 | 依赖 | 状态 |
|---|------|------|------|------|
| 1 | [Per-project ABAC](#1-per-project-abac) | 声明式属性策略 → `proj_{ulid}` RLS | MVP `db-pre-request`、`app.*` GUC | pending |
| 2 | [Policy 编辑器 / Compiler](#2-policy-编辑器--compiler) | Manager API + Dashboard UI；JSON/DSL → `CREATE POLICY` | #1 | pending |
| 3 | [authenticated bootstrap 收紧](#3-authenticated-bootstrap-收紧) | 默认 deny + opt-in policy | #1、#2 | pending |
| 4 | [Row Viewer impersonation](#4-row-viewer-impersonation) | 以指定 App 用户身份调试 ABAC | App User Session、#1 | pending |
| 5 | [多平台客户端 SDK](#5-多平台客户端-sdk) | Browser / RN / Android / iOS…；仅 Data API | [mvp-sdk.md](./mvp-sdk.md) 基线 | pending |
| 6 | [客户端 Storage](#6-客户端-storage) | App 侧上传 / 下载（Publishable Key + Session） | #5、Data API 或 Signed URL 协议 | pending |
| 7 | [Data API 扩展](#7-data-api-扩展) | RPC、视图、OpenAPI 文档 | MVP 代理稳定 | pending |
| 8 | [GraphQL 网关](#8-graphql-网关) | 可选第二 CRUD 面；对外路径不变 | Data API 网关抽象 | pending |

---

## 1. Per-project ABAC

**目标：** 每个 Project 在 `proj_{ulid}` 内独立配置 **基于属性的访问控制**（主体 / 资源 / 环境属性 + 动作 + 条件），与现有 `auth_mode` + `SET ROLE` 模型兼容。

**范围外（MVP 不做）：** Policy 元数据表、Policy Compiler、Dashboard 条件 builder、扩展 `app.*` 属性集、authenticated 默认 deny。

**架构落点（沿用主 PRD §11.11，不新增代理路径）：**

```text
Axum 网关（可信属性）→ Internal-Context → db-pre-request SET app.*
                                              → SET ROLE（auth_mode 不变）
                                              → proj_{ulid} RLS policies（ABAC 求值）
```

| 组件 | 位置 | 职责 |
|------|------|------|
| 主体属性 | Redis Session 载荷；`proj_{ulid}.users` 列 / JSONB | `user_id`、`role`、`groups`、`attrs` |
| 资源属性 | 租户表列（`owner_id`、`status`、`region`…） | RLS `USING` / `WITH CHECK` 引用 |
| 环境属性 | 网关注入 `app.client_ip`、`app.request_at`（可选） | 时间 / IP 条件 |
| 策略定义 | `public.abac_policies`（`project_id`, `table`, `action`, `conditions_json`, `version`, `enabled`） | 声明式存储 |
| 策略审计 | `public.abac_policy_revisions` | 版本 / 回滚 |
| 策略执行 | `proj_{ulid}` 表上 PostgreSQL RLS | Compiler 生成 / 更新 `CREATE POLICY` |

**`app.*` GUC 扩展契约**（本阶段定稿；MVP 仅保证 `user_id` / `auth_mode` / `project_id`）：

| GUC | 来源 | 用途 |
|-----|------|------|
| `app.user_id` | App User Session | 主体 ID |
| `app.user_role` | Session 或 `users.role` | 粗粒度角色 |
| `app.user_groups` | Session JSON | 组 membership |
| `app.user_attrs` | Session / `users.attrs` JSONB | 自定义主体属性 |
| `app.client_ip` | 网关（可选） | 环境 ABAC |

**与 `auth_mode` 关系：**

| auth_mode | ABAC |
|-----------|------|
| `anon` | 仅显式 policy 允许的读 / 写 |
| `authenticated` | **主战场** — 全量 ABAC 求值 |
| `project_operator` | bypass App 用户 ABAC（运维） |
| `service` | bypass（Secret Key；须审计） |

**验收：**

- 同一 Project 内不同 App 用户因 `role` / `groups` / 资源属性看到不同行集
- Project A 的策略不影响 Project B（schema 隔离）
- 策略变更可审计、可回滚；Compiler 输出 deterministic SQL

---

## 2. Policy 编辑器 / Compiler

**Policy Compiler（Manager API）：**

1. `POST/PUT/DELETE /api/projects/{project_id}/policies` — CRUD 策略元数据
2. 保存时编译 `conditions_json` → RLS SQL，`CREATE OR REPLACE POLICY` on `proj_{ulid}.{table}`（事务内）
3. 建表 / 改列 / 删表时同步 reconcile policies
4. 集成测试：subject × resource × action 矩阵

**Dashboard：** Policy 条件 builder UI（表级 / 全局 / 模板 — 信息架构待定，见主 PRD Open Questions）。

**实现顺序：**

1. 文档化并扩展 Internal-Context / `app.*` 契约
2. `public.abac_policies` migration + Manager API CRUD
3. Policy Compiler（JSON → RLS）+ 集成测试
4. Dashboard Policy 编辑器 UI

---

## 3. authenticated bootstrap 收紧

替换 MVP「authenticated 默认全表 CRUD」为 **默认 deny + opt-in policy**（依赖 #1、#2）。

---

## 4. Row Viewer impersonation

Dashboard Row Viewer「以某 App 用户身份查看」：`auth_mode=authenticated` + 指定 `app.user_id`；用于调试 ABAC（主 PRD §11.3 首版不做）。

---

## 5. 多平台客户端 SDK

**目标：** 为 **终端 App** 提供统一 Data API 体验；凭证模型与 [mvp-sdk.md](./mvp-sdk.md) 相同，各平台实现 HTTP + Session 持有。

**明确不做：**

- Manager API 封装（项目创建、DDL、Key 轮换等 — **暂不提供**官方 SDK）
- Secret Key / 服务端管理员 SDK
- Dashboard Project Session 客户端

**共享契约（全平台）：**

| 项 | 约定 |
|----|------|
| Base URL | `{projectUrl}/api/data/{projectId}` |
| 应用凭证 | `X-Indiebase-Api-Key: <publishable_key>` |
| 用户凭证 | `Authorization: Bearer <app_user_session_token>`（登录后） |
| CRUD 语义 | PostgREST 兼容（`select` / filter / `Prefer` / 分页） |
| Schema | **不传** `Accept-Profile` — 网关从 `projectId` 注入 |

**平台路线图：**

| 平台 | 包 / 形态 | 说明 | 状态 |
|------|-----------|------|------|
| **Browser / Web** | `@indiebase/sdk`（TS） | MVP：`postgrest-js` 薄封装 | MVP（[mvp-sdk.md](./mvp-sdk.md)） |
| **React Native** | `@indiebase/sdk-react-native` 或 monorepo `@indiebase/core` | 共享 core + RN `fetch` / 安全存储 Session | pending |
| **Android** | Kotlin（或 KMP `common` + Android） | OkHttp / Ktor；Publishable Key 可进 `BuildConfig` | pending |
| **iOS** | Swift | URLSession；Keychain 存 Session | pending |
| **Desktop** | 复用 Web SDK 或 Tauri/Electron 包装 | 与 Browser 同栈优先 | pending |

**建议 monorepo 结构：**

```text
packages/
  sdk-core/          # 共享：URL 构建、header、错误类型、PostgREST query 类型
  sdk/               # Web（MVP，postgrest-js）
  sdk-react-native/
  sdk-android/       # 或 KMP
  sdk-ios/           # 或 KMP Apple target
```

**实现顺序：**

1. 从 MVP Web SDK 抽取 **`sdk-core`**（配置、auth session、header 约定）
2. React Native（与 Web 共享最多逻辑）
3. Android + iOS 原生（或 KMP 统一 HTTP 层）
4. 各平台集成测试对齐同一 Data API 网关

**验收：**

- 各平台：`Publishable Key` only（anon）与 `+ App User Session`（authenticated）CRUD 通过
- **无**平台包暴露 Secret Key 或调用 Manager API 路径

---

## 6. 客户端 Storage

App 侧文件上传 / 下载；凭证仍为 **Publishable Key + App User Session**。

**约束：** 不走 Manager API 直调（主 PRD MVP 已规定 Publishable Key 不可直调 Storage）。需先定 **Data API 或 Signed URL** 面向客户端的协议（可能新增 `/api/data/{project_id}/files*` 类路由，而非 `/api/projects/...` Manager 路径）。

**Open：** 客户端 Storage 路由前缀、Signed URL TTL、与 ABAC 的关系。

---

## 7. Data API 扩展

PostgREST 代理扩展：RPC、数据库视图、OpenAPI 文档导出。各平台 SDK 同步暴露 `.rpc()` 等（仍 **仅** Data API）。

---

## 8. GraphQL 网关

可选第二 CRUD 面；**Manager / Data API 对外路径不变**（主 PRD §2 可替换性）。

---

## Open Questions（本 Todo 专用）

- ABAC Policy DSL 语法与 Compiler 实现细节
- Policy 编辑器 UI 信息架构（表级 vs 全局 vs 复用模板）
- 客户端 Storage 走 Data API 还是 Signed URL；是否新增 Data API 文件路由
- KMP vs 各平台独立原生 SDK 的选型
- React Native Session 持久化（SecureStore / Keychain 抽象）
- GraphQL 与 PostgREST 共存时的 schema 暴露策略

## References

- [BaaS Platform Architecture](./baas-platform-architecture.md)
- [MVP Client SDK（Web / TS）](./mvp-sdk.md)

# Indiebase BaaS — MVP 切分

| Field | Value |
|-------|-------|
| Status | Draft |
| Last updated | 2026-07-14 |
| Parent | [baas-platform-architecture.md](./baas-platform-architecture.md) |
| Client SDK | [mvp-sdk.md](./mvp-sdk.md) |
| 后续能力 | [todo.md](./todo.md)（**不在** MVP） |

从零到 **第一个可用 BaaS 闭环**：能建 Project → 设计表 → 用 Publishable Key（± App User Session）经 Data API 做 CRUD → Web TS SDK 调用。

```text
Phase 0 ──► Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5
  壳+栈      平台+Auth    Data API    表设计器     Storage      App Auth+SDK
```

---

## 总览

| Phase | 名称 | 交付物（摘要） | 验收一句话 |
|-------|------|----------------|------------|
| **0** | 工程壳 + 本地栈 | Cargo workspace、Axum `/health`、OpenAPI + Scalar 文档、compose 联调 | `cargo test` + `curl /health` + `/docs` + compose up |
| **1** | Platform + 登录 + Project | 平台表、Dashboard Session、`X-Indiebase-Project-Id`、创建 `proj_{ulid}` | 登录 → 建 Project → schema + Key 对已生成 |
| **2** | Data API 网关 | PostgREST 代理、`SET ROLE`、双路径 §6.2.3、Key 校验 | SDK URL CRUD 通；非法凭证组合 403 |
| **3** | Metadata / 表设计器 | 建表 API、bootstrap RLS、`allow_anon_read` | Manager 建表 → Data API 可 CRUD |
| **4** | Storage | Manager API + OpenDAL fs | Dashboard 上传/下载/列表 |
| **5** | App User Session + TS SDK | Project Auth、`@indiebase/sdk` 薄封装 | SDK 集成测试全绿 |

**MVP 边界：** 含 Phase 0–5。**不含** [todo.md](./todo.md)（多平台 SDK、ABAC Policy 编辑器、GraphQL 等）。

---

## Phase 0 — 工程壳 + 本地栈

**目标：** 可编译、可测、infra 就绪。

| 任务 | 说明 |
|------|------|
| Cargo workspace | 根 `Cargo.toml`；crate `crates/api` |
| Axum 壳 | `GET /health` → 200 |
| API 文档 | `utoipa` 生成 OpenAPI；`GET /openapi.json`；Scalar UI `GET /docs` |
| 开发方式 | 实现为主；测试按需补充（见 `.cursor/rules/backend-tdd-prd.mdc`） |
| 配置 | `INDIEBASE_ENV`（默认 `development`；另支持 `production`）；Vite 四层 dotenv；Postgres / Redis 离散字段 |
| 本地栈 | 已有 `docker compose`：Postgres 17、Redis 6、PostgREST |

**验收：**

- [x] `cargo fmt`、`cargo clippy`、`cargo test` 通过
- [x] `docker compose up -d` 后 Axum 能连 Postgres / Redis
- [x] `curl localhost:{port}/health` 返回 OK
- [x] `curl localhost:{port}/openapi.json` 返回含 `/health` 的 OpenAPI JSON
- [x] 浏览器打开 `localhost:{port}/docs` 可见 Scalar API 文档

**依赖：** 无。

---

## Phase 1 — Platform + 登录 + Project 生命周期

**目标：** 平台成员能登录、创建 Project，并用同一 token + project 头访问项目资源。

| 任务 | 说明 |
|------|------|
| Migrations | 开发：`db/schema.rs` SeaQuery synchronize；生产：sqlx `migrations/`；表均含 `deleted_at` |
| Dashboard Session | `POST /api/auth/login`、`logout`；Redis `dashboard_session:` |
| Project CRUD | `POST /api/projects` 等（Manager API） |
| 创建 Project | ULID → `CREATE SCHEMA proj_{ulid}` → 租户 DB roles → 默认 Key 对 → PostgREST schema 注册 + reload |
| Project 上下文 | `X-Indiebase-Project-Id` + `project_members`；`GET /api/auth/project-context` |

**验收：**

- [x] Dashboard 登录后调 Manager API 列出 projects
- [x] 创建 Project 后 DB 存在 `proj_{ulid}` + `api_keys` 两行
- [x] Dashboard token + `X-Indiebase-Project-Id` → `GET /api/auth/project-context` 返回正确 `role`
- [x] PostgREST reload 后新 schema 可达（内网 smoke）— 见 `docs/notes/postgrest-schema-reload.md`（`pg_notify` + schemas 文件）

**依赖：** Phase 0。

**参考：** 主 PRD §11.1、§11.7、§11.8。

---

## Phase 2 — Data API 网关

**目标：** PostgREST 透明代理 + 身份/权限链路打通（可先 anonymous Secret Key smoke，完整 Key 校验与 Phase 1 衔接）。

| 任务 | 说明 |
|------|------|
| 路由 | SDK：`/api/data/{project_id}/*`（ULID）；Dashboard：`/api/data/*` — §6.2.3 |
| 凭证互斥 | 非法 Key/Session 组合 → 403 |
| Key 校验 | Publishable / Secret 与 URL `project_id` 绑定 |
| 代理 | strip 前缀、`Accept-Profile: proj_{ulid}`、转发 Prefer/Range |
| PostgREST 身份 | authenticator + Internal-Context + `db-pre-request` + **SET ROLE** |
| Dashboard 路径 | Dashboard Session + `X-Indiebase-Project-Id` → `project_operator*` |

**验收：**

- [x] Secret Key：`GET /api/data/{id}/users` 代理成功（service role）
- [x] Publishable Key：同路径可用；Key 与 URL project 不一致 → 403
- [x] Dashboard + project 头：`GET /api/data/users` 可用；带 `X-Indiebase-Api-Key` → 403
- [x] SDK URL + Dashboard Session Bearer → 403
- [x] 内网 PostgREST 不对公网（生产不 publish 端口）

**依赖：** Phase 1（schema、Key、Session 存在）。

**参考：** 主 PRD §6.2、§11.11；[mvp-sdk.md §5](./mvp-sdk.md#5-gateway-compatibility-mvp)。

---

## Phase 3 — Metadata / 表设计器 API

**目标：** 在租户 schema 建表，Data API 可 CRUD，RLS bootstrap 生效。

| 任务 | 说明 |
|------|------|
| Metadata 表 | `public.table_metadata`、`column_metadata`（含 `allow_anon_read`） |
| Manager API | 创建/修改/删除表、列（DDL 在 `proj_{ulid}`） |
| bootstrap RLS | 建表时写入默认 policies（§11.3） |
| Row Viewer | Dashboard `/api/data/*` + Dashboard Session + `X-Indiebase-Project-Id` 可浏览新表 |

**验收：**

- [ ] Manager API 创建 `users` 表后，Metadata 与 `proj_{ulid}.users` 一致
- [ ] Publishable Key + 无 Session：默认不可读写；`allow_anon_read=true` 后仅 SELECT
- [ ] Publishable Key + App User Session（Phase 5 前可用手工 token smoke）：authenticated CRUD
- [ ] Dashboard Data API owner：全表 CRUD；member：只读

**依赖：** Phase 2。

**参考：** 主 PRD §11.3 MVP bootstrap RLS。

---

## Phase 4 — Storage

**目标：** Dashboard 文件管理（Manager API + OpenDAL 本地 fs）。

| 任务 | 说明 |
|------|------|
| OpenDAL | 默认 `fs` backend |
| Manager API | `/api/projects/{project_id}/files*`（upload / list / download / delete） |
| 权限 | Dashboard Session + `project_role`（§10.1） |

**验收：**

- [ ] owner/admin：上传、列表、下载、删除
- [ ] member：列表、下载；写操作拒绝
- [ ] Publishable Key **不能**直调 Storage API

**依赖：** Phase 1（Project、Session）。

**说明：** 客户端 SDK Storage 在 [todo.md §6](./todo.md#6-客户端-storage)，**不在** MVP SDK。

---

## Phase 5 — App User Session + TS SDK

**目标：** 终端用户登录 + 官方 Web SDK 闭环。

| 任务 | 说明 |
|------|------|
| Project Auth | `POST /api/auth/app/login`、`logout`；Redis `app_user_session:` |
| Session 校验 | `project_id` 与 URL 一致（§6.2.3） |
| TS SDK | `packages/sdk-ts`：`postgrest-js` 薄封装 — [mvp-sdk.md](./mvp-sdk.md) |
| 集成测试 | 网关 + PostgREST + SDK：select/insert/update/delete |

**验收：**

- [ ] App 登录后 SDK 自动带 `Authorization` + `X-Indiebase-Api-Key`
- [ ] anon / authenticated / 跨路径 403 用例全过（mvp-sdk §7）
- [ ] 与 Phase 3 bootstrap RLS 行为一致

**依赖：** Phase 2、Phase 3。

---

## MVP 完成定义（Definition of Done）

同时满足：

1. **平台：** 登录 → 建 Project → Project 登录 → 表设计器建表  
2. **Dashboard 数据：** Row Viewer CRUD（Dashboard Session + project 头）  
3. **SDK：** TS 客户端 Publishable Key ± App User Session CRUD  
4. **Storage：** Dashboard 文件管理（Phase 4）  
5. **质量：** `cargo test`、关键路径集成测试、无已知 P0 安全洞（双路径互斥、Key 不转发 PostgREST）

---

## 与 OpenSpec

OpenSpec **按能力（capability）** 组织，**不**按 Phase 编号命名 change。

主规格见 `openspec/specs/`（已落地）：`server-bootstrap`、`env-config`、`api-docs`、`platform-auth`、`project-lifecycle`。

后续工作建议的 change / capability 名（与本文件交付切分对应，但命名不带 Phase）：

| 能力 | 建议 change 名 |
|------|----------------|
| Data API 网关 | `data-api-gateway` |
| Metadata / 表设计器 | `metadata-table-designer` |
| Storage (OpenDAL fs) | `storage-opendal-fs` |
| App User Session | `app-user-session` |
| TS SDK | `ts-sdk` |

大范围可拆多个 change（例如 PostgREST `db-pre-request` spike 与网关主路径分开）。PRD 仍用本文件跟踪验收勾选。

## References

- [BaaS Platform Architecture](./baas-platform-architecture.md)
- [MVP Client SDK](./mvp-sdk.md)
- [后续 Todo](./todo.md)

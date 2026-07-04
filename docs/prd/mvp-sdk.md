# Indiebase MVP Client SDK — TypeScript

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase — BaaS |
| Last updated | 2026-07-04 |
| Parent | [baas-platform-architecture.md](./baas-platform-architecture.md) · [mvp-phases.md](./mvp-phases.md) |

## 1. Scope

首版 TypeScript SDK **仅覆盖客户端 Data API（SDK 路径）**：

| In scope | Out of scope |
|----------|--------------|
| `/api/data/{project_id}/*` + **Publishable Key** + 可选 **App User Session** | **Manager API**（暂不提供官方 SDK；S2S 自建） |
| table CRUD + PostgREST filters | Dashboard Project Session 客户端 |
| `@supabase/postgrest-js` 薄封装（Web / Browser MVP） | **Secret Key** / 服务端管理员 SDK |
| | 其他平台 SDK（RN / Android / iOS — 见 [todo.md §5](./todo.md#5-多平台客户端-sdk)） |

架构背景见主 PRD [§6.2.2 Data API — SDK](./baas-platform-architecture.md#622-sdk--程序化调用)、[§11.2 API Key 模型](./baas-platform-architecture.md#112-api-key-模型)、[§11.6 SDK 初始化](./baas-platform-architecture.md#116-sdk-初始化)、[§11.9 App User Session](./baas-platform-architecture.md#119-app-user-sessionsdk-终端用户--data-api-用户凭证)。

## 2. Approach

Data API 网关对 PostgREST 做**透明代理**（URL query、body、常用 header），MVP SDK **复用** [`@supabase/postgrest-js`](https://github.com/supabase/postgrest-js) 作为 HTTP 层，Indiebase 只提供配置封装。

| Item | Convention |
|------|--------------|
| Dependency | **`@supabase/postgrest-js`**（standalone）；**not** `@supabase/supabase-js` |
| Base URL | `{projectUrl}/api/data/{projectId}` |
| Project Key | `X-Indiebase-Api-Key: <publishable_key>` |
| User Session | `Authorization: Bearer <app_user_session_token>` — SDK 在用户登录后自动附加（Opaque Token + Redis；RFC 6750） |
| `schema` option | **Do not pass** — `Accept-Profile` is injected by the Data API gateway from URL `project_id` |
| MVP capabilities | table **CRUD** + PostgREST filters (`select`, `eq`, `order`, etc.) |

## 3. Target DX

```typescript
import { createIndiebaseClient } from '@indiebase/sdk';

const db = createIndiebaseClient({
  projectUrl: 'https://indiebase.example.com',
  projectId: '01jcqz4sxf7k2m8n3p5r6t9vwx',
  publishableKey: process.env.NEXT_PUBLIC_INDIEBASE_PUBLISHABLE_KEY!,
});

// 匿名（策略允许时）
const { data, error } = await db.from('posts').select('*').eq('published', true);

// 用户登录后（Project Auth 返回 Opaque Session Token，由 SDK 持有并自动附带）
await db.auth.setSession({ accessToken: '<app_user_session_token>' });
const { data: mine } = await db.from('posts').select('*'); // auth_mode=authenticated → SET ROLE → RLS
```

## 4. Implementation Sketch

```typescript
import { PostgrestClient } from '@supabase/postgrest-js';

export function createIndiebaseClient(opts: {
  projectUrl: string;
  projectId: string;
  publishableKey: string;
}) {
  const root = opts.projectUrl.replace(/\/$/, '');
  let userSessionToken: string | null = null;

  const client = new PostgrestClient(`${root}/api/data/${opts.projectId}`, {
    headers: {
      'X-Indiebase-Api-Key': opts.publishableKey,
    },
    fetch: (input, init) => {
      const headers = new Headers(init?.headers);
      headers.set('X-Indiebase-Api-Key', opts.publishableKey);
      if (userSessionToken) {
        headers.set('Authorization', `Bearer ${userSessionToken}`);
      } else {
        headers.delete('Authorization');
      }
      return fetch(input, { ...init, headers });
    },
  });

  return Object.assign(client, {
    auth: {
      setSession: ({ accessToken }: { accessToken: string }) => {
        userSessionToken = accessToken;
      },
      signOut: () => {
        userSessionToken = null;
      },
    },
  });
}
```

Suggested layout:

```text
packages/sdk-ts/
  src/
    client.ts
    index.ts
  package.json
```

## 5. Gateway Compatibility (MVP)

For `postgrest-js` to work, the Data API gateway must:

- Forward query strings as-is (`select`, `order`, `limit`, filter operators)
- Forward request bodies (JSON) as-is
- Return response bodies and status codes as-is
- Forward `Prefer` (e.g. `return=representation`), `Range` / `Content-Range` (pagination)
- Accept `X-Indiebase-Api-Key` for Publishable Key lookup；Accept `Authorization` for App User Session lookup（Redis）；不将客户端 token 原样转发至 PostgREST
- Resolve `auth_mode` → signed `X-Indiebase-Internal-Context` → PostgREST `db-pre-request` → **SET ROLE**
- Enforce **§6.2.3** path/credential mutual exclusion（ULID 路由 + 非法凭证 `403`）
- **Not** require clients to send `Accept-Profile` / `Content-Profile`

## 6. vs Supabase SDK

| | Supabase | Indiebase MVP SDK |
|--|----------|-------------------|
| Hosting | supabase.co | Self-hosted `projectUrl` |
| Project | URL / subdomain | Path `{projectId}` |
| Client credential | anon (publishable) key → `X-Indiebase-Api-Key` header | **Publishable Key** → `X-Indiebase-Api-Key` header |
| Server credential | service_role key → `X-Indiebase-Api-Key` header | **不提供**官方 SDK；Secret Key 仅服务端自建 |
| User auth | Supabase Auth JWT → `Authorization` | **App User Session**（Opaque Token + Redis）→ `Authorization` |
| Access control | RLS + role switch | MVP：bootstrap RLS；后续：[todo.md §1 ABAC](./todo.md#1-per-project-abac) |
| Platforms | Web + mobile + RN 等 | MVP：**Browser / TS**；后续：[todo.md §5 多平台](./todo.md#5-多平台客户端-sdk) |
| Backend | PostgREST direct | **Data API gateway** proxy |
| Package | `@supabase/supabase-js` | `@supabase/postgrest-js` + thin wrapper |

## 7. Verification

- Integration tests: Axum Data API gateway + PostgREST + `postgrest-js` for `select` / `insert` / `update` / `delete`
- Do not mock PostgREST semantics; validate against the real proxy
- Add cases: Publishable Key only（`auth_mode=anon`）, Publishable Key + App User Session（`auth_mode=authenticated`）
- Validate **SET ROLE** + bootstrap RLS behavior（anon default deny；`allow_anon_read` opt-in SELECT）
- Reject cross-path credentials per主 PRD §6.2.3（e.g. Project Session on SDK URL → `403`；Key on Dashboard `/api/data/*` → `403`）

## 8. 后续实现（out of MVP scope）

见 **[todo.md §5](./todo.md#5-多平台客户端-sdk)** — React Native、Android、iOS 等；**仅 Data API**，不含 Manager API。

## 9. Open Questions

- Package name / publish target (`@indiebase/sdk` vs scoped private name)
- Node vs browser `fetch` polyfill requirements
- TypeScript types for dynamic tables (generic `from()` vs generated types — 见 [todo.md](./todo.md))
- Project Auth 登录 API 与 App User Session TTL（依赖主 PRD Auth 模块）

## 10. References

- [BaaS Platform Architecture](./baas-platform-architecture.md)
- [后续实现 Todo](./todo.md)
- [postgrest-js](https://github.com/supabase/postgrest-js)
- [OpenSpec config](../../openspec/config.yaml)

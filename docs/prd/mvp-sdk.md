# Indiebase MVP Client SDK — TypeScript

| Field | Value |
|-------|-------|
| Status | Draft |
| Product | Indiebase Community — BaaS |
| Last updated | 2026-07-03 |
| Parent | [baas-platform-architecture.md](./baas-platform-architecture.md) |

## 1. Scope

首版 TypeScript SDK **仅覆盖 Data API（SDK 路径）**：

| In scope | Out of scope |
|----------|--------------|
| `/api/data/{project_id}/*` + Project API Key | Manager API |
| table CRUD + PostgREST filters | Dashboard Project Session 客户端 |
| `@supabase/postgrest-js` 薄封装 | `.rpc()`、Realtime、Auth、Storage |

架构背景见主 PRD [§6.2.2 Data API — SDK](./baas-platform-architecture.md#622-sdk--程序化调用)、[§11.2 Project API Key](./baas-platform-architecture.md#112-project-api-keysdk-鉴权)。

## 2. Approach

Data API 网关对 PostgREST 做**透明代理**（URL query、body、常用 header），MVP SDK **复用** [`@supabase/postgrest-js`](https://github.com/supabase/postgrest-js) 作为 HTTP 层，Indiebase 只提供配置封装。

| Item | Convention |
|------|--------------|
| Dependency | **`@supabase/postgrest-js`**（standalone）；**not** `@supabase/supabase-js` |
| Base URL | `{host}/api/data/{project_id}` |
| Auth | `Authorization: Bearer <project_api_key>` |
| `schema` option | **Do not pass** — `Accept-Profile` is injected by the Data API gateway from URL `project_id` |
| MVP capabilities | table **CRUD** + PostgREST filters (`select`, `eq`, `order`, etc.) |

## 3. Target DX

```typescript
import { createIndiebaseClient } from '@indiebase/sdk';

const db = createIndiebaseClient({
  baseUrl: 'https://indiebase.example.com',
  projectId: '01jcqz4sxf7k2m8n3p5r6t9vwx',
  apiKey: process.env.INDIEBASE_API_KEY!,
});

const { data, error } = await db.from('users').select('*').eq('status', 'active');
// → GET /api/data/01jcqz4sxf7k2m8n3p5r6t9vwx/users?select=*&status=eq.active
```

## 4. Implementation Sketch

```typescript
import { PostgrestClient } from '@supabase/postgrest-js';

export function createIndiebaseClient(opts: {
  baseUrl: string;
  projectId: string;
  apiKey: string;
}) {
  const root = opts.baseUrl.replace(/\/$/, '');
  return new PostgrestClient(`${root}/api/data/${opts.projectId}`, {
    headers: { Authorization: `Bearer ${opts.apiKey}` },
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
- **Not** require clients to send `Accept-Profile` / `Content-Profile`

## 6. vs Supabase SDK

| | Supabase | Indiebase MVP SDK |
|--|----------|-------------------|
| Hosting | supabase.co | Self-hosted `baseUrl` |
| Project | URL / subdomain | Path `{projectId}` |
| Credential | anon / service key | Project API Key |
| Backend | PostgREST direct | **Data API gateway** proxy |
| Package | `@supabase/supabase-js` | `@supabase/postgrest-js` + thin wrapper |

## 7. Verification

- Integration tests: Axum Data API gateway + PostgREST + `postgrest-js` for `select` / `insert` / `update` / `delete`
- Do not mock PostgREST semantics; validate against the real proxy

## 8. Open Questions

- Package name / publish target (`@indiebase/sdk` vs scoped private name)
- Node vs browser `fetch` polyfill requirements
- TypeScript types for dynamic tables (generic `from()` vs generated types — post-MVP)

## 9. References

- [BaaS Platform Architecture](./baas-platform-architecture.md)
- [postgrest-js](https://github.com/supabase/postgrest-js)
- [OpenSpec config](../../openspec/config.yaml)

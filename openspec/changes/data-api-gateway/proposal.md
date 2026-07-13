## Why

Platform auth and project lifecycle are live, but clients still cannot reach tenant data through PostgREST. Without a Data API gateway, Table Designer, App User Session, and the TS SDK have no CRUD path — this is the next MVP dependency.

## What Changes

- Dual-path Data API routes: SDK `/api/data/{project_id}/*` (ULID) and Dashboard `/api/data/*` (§6.2.3).
- Credential mutual exclusion: illegal Key/Session combinations → `403`.
- Publishable / Secret Key validation bound to URL `project_id`; Dashboard path uses Dashboard Session + `X-Indiebase-Project-Id`.
- Transparent PostgREST proxy: strip prefix, inject `Accept-Profile` / `Content-Profile`, forward Prefer/Range/body/status.
- Signed `X-Indiebase-Internal-Context` + PostgREST `db-pre-request` → `SET LOCAL app.*` + **SET ROLE** per `auth_mode`.
- App User Session lookup on SDK path when Bearer present (full Project Auth issuance deferred to `app-user-session`; gateway MUST accept Redis `app_user_session:` if present).
- OpenAPI docs for Data API surface; sync `docs/prd/mvp-phases.md` Phase 2 acceptance.

## Capabilities

### New Capabilities

- `data-api-gateway`: Dual-path PostgREST proxy, Key/Session auth termination, Internal-Context, SET ROLE via db-pre-request.

### Modified Capabilities

- `api-docs`: Document `/api/data/*` and `/api/data/{project_id}/*` paths and error responses.
- `env-config`: New env for PostgREST URL, authenticator secret, Internal-Context signing key (if not already present).

## Non-goals

- Manager table DDL / bootstrap RLS templates (`metadata-table-designer`).
- Storage / OpenDAL (`storage-opendal-fs`).
- Project Auth login/logout routes and App User provisioning (`app-user-session`).
- TypeScript SDK package (`ts-sdk`).
- ABAC Policy editor, GraphQL, client Storage (todo.md).
- Exposing PostgREST to the public internet.

## Impact

- `crates/api`: new Data API modules (routing, auth matrix, proxy, context signing); OpenAPI; tests.
- Postgres: global `db-pre-request` function; tenant roles already provisioned at project create must be exercised.
- Docker/PostgREST: `db-pre-request` config; keep PostgREST internal-only in production.
- `.env.example` + PRD Phase 2 checkboxes.

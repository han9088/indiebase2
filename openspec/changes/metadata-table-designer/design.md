## Context

Manager has projects but no table/column Designer APIs. Data API (once live) needs real tables plus bootstrap RLS so anon/authenticated/operator behave per §11.3. Metadata lives in `public`; DDL in `proj_{ulid}`.

## Goals / Non-Goals

**Goals:**

- `table_metadata` / `column_metadata` with `allow_anon_read`.
- Manager CRUD for tables/columns; transactional metadata + DDL + RLS bootstrap.
- Align grants so authenticated can CRUD and anon is deny-by-default except opt-in SELECT.

**Non-Goals:** Dashboard UI, ABAC Policy editor, App Auth, Storage, SDK package.

## Decisions

### 1. Route shape

- **Decision:** `POST/GET/PATCH/DELETE /api/projects/{project_id}/tables` and nested `/columns` (or `/api/tables` + project header). Prefer path `{project_id}` for Manager clarity, consistent with Storage.
- **Auth:** Dashboard Session + membership (`owner`/`admin` write; `member` read-only metadata).

### 2. DDL generation

- **Decision:** Server-side whitelist of column types (text, int, bool, timestamptz, jsonb, …); identifiers quoted/validated; never accept raw SQL from clients.
- **Alternatives:** SQL string from client — rejected.

### 3. Bootstrap RLS

- **Decision:** On CREATE TABLE: `ENABLE ROW LEVEL SECURITY`; policies for anon (deny or SELECT if `allow_anon_read`), authenticated (ALL), rely on role privileges for operators/service (BYPASSRLS / grants).
- **Toggle:** Changing `allow_anon_read` replaces anon SELECT policy.

### 4. Schema sync

- **Decision:** SeaQuery synchronize in development + sqlx migration for production, matching Phase 1 pattern.

### 5. PostgREST reload

- **Decision:** After DDL, `NOTIFY pgrst, 'reload schema'` (existing helper) so new tables appear.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Incomplete grants for INSERT on authenticated | Explicit GRANT on new tables in create path |
| Policy name collisions | Deterministic policy names per table |
| Destructive DROP | Soft-delete metadata + DROP TABLE only for owner/admin; document |

## Migration Plan

1. Add metadata tables.
2. Ship Manager APIs + RLS templates.
3. Integration: create `users` → Data API CRUD matrix.

## Open Questions

- Exact Manager path prefix (`/api/projects/.../tables` vs `/api/tables`) — default to project-scoped path.
- Primary key default: ULID text vs bigserial — prefer ULID text `id` for Indiebase consistency.

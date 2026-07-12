## Why

Phase 0 delivered a runnable Axum shell. Phase 1 unlocks the first product loop: platform members can log in, create a Project (`proj_{ulid}`), and obtain Project Session context — the foundation for Data API (Phase 2).

## What Changes

- Platform migrations in `public`: `users`, `projects`, `project_members`, `api_keys` (Publishable + Secret hashes).
- Dashboard Session: `POST /api/auth/login`, `POST /api/auth/logout`; Redis `dashboard_session:{token}` (opaque token, no JWT).
- Project Session: `POST /api/auth/project/login`, `POST /api/auth/project/logout`; Redis `project_session:{token}` with `project_id`, `project_role`.
- Manager Project APIs: create/list (at minimum) under `/api/projects*`.
- On Project create: ULID → `CREATE SCHEMA proj_{ulid}` → tenant DB roles → default Key pair → register schema with PostgREST + reload.
- Wire sqlx pool + Redis client from existing config; document new routes in OpenAPI/Scalar.
- Sync `docs/prd/mvp-phases.md` Phase 1 acceptance checkboxes when done.

## Capabilities

### New Capabilities

- `platform-auth`: Dashboard and Project opaque-token sessions (Redis); Manager auth routes.
- `project-lifecycle`: Platform tables, Project CRUD, schema provisioning, default API keys, PostgREST schema reload.

### Modified Capabilities

- `api-docs`: Document new Manager `/api/auth/*` and `/api/projects*` paths in OpenAPI.
- `server-bootstrap`: Server connects to Postgres/Redis at startup (pools), beyond Phase 0 liveness-only health.

## Non-goals

- Data API gateway / PostgREST proxy (Phase 2).
- Table Designer / bootstrap RLS (Phase 3).
- Storage (Phase 4).
- App User Session / TS SDK (Phase 5).
- OAuth / social login; email verification polish beyond minimal password (or seed) login for MVP.
- SeaweedFS / object storage.

## Impact

- `crates/api`: migrations, auth modules, project service, Redis session store, sqlx.
- Postgres DDL + tenant roles; PostgREST `PGRST_DB_SCHEMAS` / reload mechanism.
- `.env.example` if new vars (e.g. session TTL, PostgREST admin URL).
- PRD: `docs/prd/mvp-phases.md` Phase 1; architecture already specifies §11.1 / §11.7 / §11.8.

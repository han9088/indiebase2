## Why

SDK clients need an App User identity (Opaque Token + Redis) distinct from Dashboard Session. Without Project Auth login/logout and `app_user_session:` payloads, Data API stays stuck at `anon` / Secret Key and cannot satisfy authenticated RLS or the TS SDK contract.

## What Changes

- Project Auth: `POST /api/auth/app/login`, `POST /api/auth/app/logout` (or equivalent).
- Redis `app_user_session:{token}` → `{ end_user_id, project_id, role, exp, … }`.
- Tenant App user store in `proj_{ulid}` (minimal password/email or seed path for MVP).
- Session `project_id` MUST match Data API URL `{project_id}` (gateway already enforces; this change issues correct payloads).
- OpenAPI for App Auth routes; sync `docs/prd/mvp-phases.md` Phase 5 App Auth acceptance (SDK package is separate `ts-sdk`).
- Depends on `data-api-gateway` for end-to-end authenticated proxy; pairs with `metadata-table-designer` for bootstrap RLS verification.

## Capabilities

### New Capabilities

- `app-user-session`: Project Auth login/logout, Redis App User Session, tenant end-user records, TTL/logout revoke.

### Modified Capabilities

- `api-docs`: Document `/api/auth/app/*`.
- `data-api-gateway`: Only if gateway delta needed once App Auth lands (e.g. clearer authenticated scenarios); prefer no delta if gateway already specifies optional App User Bearer.

## Non-goals

- OAuth / social login / email verification polish.
- Dashboard Session changes.
- TS SDK package (`ts-sdk`).
- ABAC / Policy editor; authenticated default-deny (`todo.md`).
- Impersonation in Row Viewer.

## Impact

- `crates/api`: app auth routes, session store mirror of dashboard sessions, tenant users DDL/bootstrap.
- Redis namespace `app_user_session:`.
- PRD Phase 5 (App Auth portion) + architecture §11.9.

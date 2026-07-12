## Context

Phase 0 (`crates/api`) has Axum, config (`INDIEBASE_ENV` + Vite dotenv), health, OpenAPI/Scalar. No sqlx pool, Redis client, migrations, or Manager auth yet. Architecture §11.1 / §11.7 / §11.8 define schema-per-project, Dashboard Session, and Project Session.

## Goals / Non-Goals

**Goals:**

- Platform migrations + Project create provisioning (`proj_{ulid}`, roles, keys, PostgREST reload).
- Opaque Dashboard / Project sessions in Redis.
- Manager routes under `/api/auth/*` and `/api/projects*`.
- sqlx + Redis wired at startup; OpenAPI updated; Phase 1 PRD acceptance updated.

**Non-Goals:** Data API proxy, Table Designer, Storage, App User Session, OAuth.

## Decisions

### 1. Auth model

- **Decision:** Opaque tokens in Redis only (no JWT), matching architecture.
- **Alternatives:** JWT — rejected by project rules.
- **Keys:** `dashboard_session:{token}`, `project_session:{token}`.

### 2. Password / bootstrap user

- **Decision:** MVP password login with Argon2 (or equivalent) hashes in `public.users`; seed a local admin via migration or `just` recipe for development.
- **Alternatives:** Magic-link only — defer.

### 3. Migrations

- **Decision:** sqlx migrate (`migrations/` under `crates/api` or workspace), applied on startup or via `just migrate`.
- **Alternatives:** SeaORM migrate — prefer sqlx already chosen in architecture.

### 4. Project create transaction

- **Decision:** Single logical transaction for `projects` + `project_members` + `api_keys`; DDL (`CREATE SCHEMA`, roles) may run outside PG transactional DDL limits — document ordering and compensate (delete project row) on failure after DDL.
- **Roles:** Create per-schema or shared role names with schema grants as in §11.11; prefer schema-qualified grants on shared role names if PRD allows, else `proj_{ulid}_*` roles — **follow §11.11 naming** (tenant roles `anon` / `authenticated` / `service` / `project_operator*`) with grants limited to that schema.

### 5. API key storage

- **Decision:** Store only hashes (`ib_pub_` / `ib_sec_` prefixes on plaintext); return plaintext once on create.
- **Redis:** Optional key lookup cache can wait until Phase 2 if not needed for Manager.

### 6. PostgREST reload

- **Decision:** Prefer PostgREST `NOTIFY pgrst, 'reload schema'` (or documented admin endpoint) after updating schema list. If compose uses static `PGRST_DB_SCHEMAS`, Phase 1 may use a writable config file + container signal **or** `PGRST_DB_SCHEMAS=*` / dynamic discovery if supported — spike early; document chosen approach in README.
- **Fallback:** Document manual restart for local smoke if automatic reload is blocked.

### 7. Module layout

```text
crates/api/src/
  routes/auth.rs, routes/projects.rs
  auth/session.rs, auth/password.rs
  projects/service.rs
  db/ (pool, migrations hook)
  redis_client.rs
```

Nest Manager under `/api` in `app.rs`; keep Data API nest commented for Phase 2.

### 8. Tests

- Integration tests with testcontainers or shared compose (preferred when available); otherwise unit-test session helpers and handler tests with mocked Redis/sqlx where practical. Tests optional per repo rules but covering login + create project is strongly recommended for Phase 1 acceptance.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| PostgREST schema reload unreliable in compose | Spike in first tasks; document manual reload fallback |
| DDL + row insert partial failure | Ordered steps + cleanup; integration test |
| Password seeding security | Dev-only seed; never commit production secrets |
| Phase 1 scope creep into Data API | Hard non-goal; no `/api/data` routes |

## Migration Plan

1. Land migrations; `just migrate` / startup migrate.
2. Deploy/restart API with Redis + Postgres.
3. Create project; verify schema + keys; smoke PostgREST.
4. Rollback: drop new schemas/roles carefully; reverse migrations if unused.

## Open Questions

- Exact PostgREST reload mechanism for Orb/compose (NOTIFY vs restart).
- Whether Project login requires live Dashboard Session cookie/header in the same request (PRD: usually yes) — **default: require Dashboard Session Bearer for `project/login`**.

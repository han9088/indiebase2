## Context

Phase 1 delivered Dashboard Session, projects, `proj_{ulid}` provisioning (including grants to shared roles `anon` / `authenticated` / `project_operator*` / `service`), API keys, and PostgREST schema registration. No `/api/data/*` routes exist. Architecture §6.2 and §11.11 define the dual-path gateway and SET ROLE model.

## Goals / Non-Goals

**Goals:**

- Dual-path Data API proxy with §6.2.3 credential mutual exclusion.
- Terminate Publishable/Secret Key and Dashboard Session; never forward client credentials to PostgREST.
- Inject `Accept-Profile` / signed Internal-Context; enable `db-pre-request` + SET ROLE.
- Accept optional App User Bearer on SDK path when Redis key exists (issuance in `app-user-session`).

**Non-Goals:** Table DDL/RLS templates, Storage, App Auth routes, TS SDK.

## Decisions

### 1. Route registration order

- **Decision:** Register SDK catch `/api/data/{project_id}/{*path}` only when `project_id` matches 26-char lowercase ULID; otherwise Dashboard `/api/data/{*path}`.
- **Alternatives:** Single handler with runtime branch — same outcome; prefer Axum routing that fails closed for ULID-shaped segments.

### 2. Proxy HTTP client

- **Decision:** `reqwest` (or existing HTTP client if present) to `POSTGREST_URL`; stream or buffer body; preserve status and selected headers (`Prefer`, `Range`, `Content-Range`, `Content-Type`).
- **Alternatives:** Hyper reverse-proxy middleware — heavier; start with request/response rebuild for clarity.

### 3. Internal-Context signing

- **Decision:** HMAC-SHA256 (or Ed25519) over canonical JSON payload (`auth_mode`, `user_id`, `project_id`, `project_role`, `exp`); shared secret in env (`INDIEBASE_INTERNAL_CONTEXT_SECRET`). PL/pgSQL `db-pre-request` verifies and `SET LOCAL` + `SET ROLE`.
- **Alternatives:** Unsigned header + network trust only — rejected for defense in depth.

### 4. PostgREST authenticator

- **Decision:** Gateway sends PostgREST JWT/secret already configured in compose (`PGRST_JWT_SECRET` / authenticator role). Client Authorization is replaced, never forwarded.
- **Document:** Align with current `docker/postgrest/postgrest.conf`.

### 5. Key lookup

- **Decision:** Hash incoming `X-Indiebase-Api-Key`, lookup `api_keys` (active + project bind); optional Redis cache later.
- **Secret Key:** `auth_mode=service`; audit log at minimum (structured tracing).

### 6. Dashboard path auth_mode

- **Decision:** Reuse `ProjectScopedAuth` / membership: owner/admin → `project_operator`; member → `project_operator_readonly`. Optional fast-path reject writes for member.

### 7. Module layout

```text
crates/api/src/
  routes/data.rs
  data/auth_matrix.rs
  data/proxy.rs
  data/internal_context.rs
```

### 8. Smoke without tenant tables

- Secret Key / operator against empty schema returns PostgREST 404/empty — acceptable for Phase 2 acceptance if a minimal smoke table is seeded in tests OR acceptance waits for metadata change. Prefer integration test that creates a throwaway table in `proj_*` inside the test.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| db-pre-request misconfig → all requests fail | Spike early; local compose fixture; fail-closed tests |
| ULID false-positive on table names | Table names rarely 26-char Crockford; document reserved pattern |
| App User Session not yet issued | Gateway still implements lookup; mint test tokens in Redis for tests |
| Forwarding client Internal-Context | Strip/overwrite always |

## Migration Plan

1. Add env secrets + PostgREST `db-pre-request` SQL migration.
2. Ship gateway routes; verify Secret Key smoke.
3. Rollback: remove routes; keep SQL function harmless if unused.

## Open Questions

- Exact PostgREST authenticator JWT claims already used in compose — confirm during spike.
- Whether authenticated INSERT grants need expansion beyond current DEFAULT PRIVILEGES (likely yes in metadata change).

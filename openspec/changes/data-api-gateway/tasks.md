## 1. Config & PostgREST pre-request

- [ ] 1.1 Add env vars for Internal-Context signing secret and PostgREST authenticator credentials; update `.env.example` + config validation
- [ ] 1.2 Implement/verify Postgres `db-pre-request` function: verify signature, `SET LOCAL app.*`, `SET ROLE` per `auth_mode`
- [ ] 1.3 Wire PostgREST config (`db-pre-request`) in `docker/postgrest` and document reload

## 2. Auth matrix & context

- [ ] 2.1 Implement ULID path detection and dual-path router registration order
- [ ] 2.2 Implement Key lookup (hash → `api_keys`) with project bind checks
- [ ] 2.3 Implement Dashboard Data path using Dashboard Session + `X-Indiebase-Project-Id` → `project_operator` / `project_operator_readonly`
- [ ] 2.4 Implement §6.2.3 mutual-exclusion (403 matrix) and App User Redis lookup on SDK path only
- [ ] 2.5 Implement signed `X-Indiebase-Internal-Context` builder; strip client-supplied copies

## 3. Proxy

- [ ] 3.1 Implement PostgREST HTTP proxy: strip prefix, inject profiles + authenticator + context, forward Prefer/Range/body/status
- [ ] 3.2 Register `/api/data/*` routes; OpenAPI annotations + tag

## 4. Tests & PRD

- [ ] 4.1 Integration tests: Secret Key smoke; Publishable mismatch 403; Dashboard+Key 403; SDK+Dashboard Session 403
- [ ] 4.2 Sync `docs/prd/mvp-phases.md` Phase 2 acceptance when criteria met
- [ ] 4.3 `cargo fmt`, `cargo clippy -p api -- -D warnings`, `cargo test -p api`

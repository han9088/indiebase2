## 1. Dependencies & infrastructure clients

- [ ] 1.1 Add workspace deps: `sqlx` (runtime-tokio, postgres, migrate), `redis`/`fred`, `ulid`, `argon2` (or equivalent), `rand`/`sha2` for tokens and key hashing
- [ ] 1.2 Create Postgres pool + Redis client from `Config` at startup; fail fast if unreachable
- [ ] 1.3 Thread `AppState` (pool, redis, config) through Axum router

## 2. Migrations & seed

- [ ] 2.1 Add sqlx migrations for `public.users`, `projects`, `project_members`, `api_keys`
- [ ] 2.2 Apply migrations on startup or via `just migrate`
- [ ] 2.3 Dev seed: at least one platform user for local login (document password in README / `.env.example` notes)

## 3. Platform auth (Dashboard + Project sessions)

- [ ] 3.1 Implement opaque token mint + Redis `dashboard_session:` store/load/delete
- [ ] 3.2 `POST /api/auth/login` and `POST /api/auth/logout` with Bearer extraction middleware for Manager routes
- [ ] 3.3 Implement `project_session:` store; `POST /api/auth/project/login` (require Dashboard Session + membership) and `project/logout`
- [ ] 3.4 Integration or handler tests for login success/401 and project login membership 403

## 4. Project lifecycle

- [ ] 4.1 Spike PostgREST schema registration + reload (NOTIFY vs config); document chosen approach
- [ ] 4.2 Implement project create: ULID, insert rows, `CREATE SCHEMA proj_{ulid}`, tenant roles/grants, default Publishable+Secret key hashes, return one-time plaintext keys
- [ ] 4.3 Trigger PostgREST reload / schema registration after create
- [ ] 4.4 `GET /api/projects` list for current Dashboard user
- [ ] 4.5 Test or smoke: after create, schema + two `api_keys` rows exist; project session Redis shape correct

## 5. OpenAPI, docs, PRD

- [ ] 5.1 Annotate new routes with `#[utoipa::path]` and register in `ApiDoc`
- [ ] 5.2 Update README / `just` recipes for migrate + login smoke
- [ ] 5.3 Sync `docs/prd/mvp-phases.md` Phase 1 acceptance checkboxes when criteria met
- [ ] 5.4 Update `.env.example` for any new vars (session TTL, PostgREST admin/reload)

## 6. Verification

- [ ] 6.1 `cargo fmt`, `cargo clippy -p api -- -D warnings`, `cargo test -p api`
- [ ] 6.2 Manual smoke: login → create project → list projects → project login → Redis key present; PostgREST sees schema

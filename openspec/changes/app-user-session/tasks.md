## 1. Tenant app users

- [ ] 1.1 Provision `proj_{ulid}.app_users` (and grants) in project create / schema provisioning
- [ ] 1.2 Helpers to create/verify app user passwords (Argon2, mirror platform users)

## 2. App Auth sessions

- [ ] 2.1 Redis `app_user_session:` mint/load/delete with TTL (payload: end_user_id, project_id, role, exp)
- [ ] 2.2 `POST /api/auth/app/login` requiring Publishable Key + project credentials
- [ ] 2.3 `POST /api/auth/app/logout` revoking Bearer App User Session
- [ ] 2.4 OpenAPI for App Auth routes

## 3. Tests & PRD

- [ ] 3.1 Tests: login/logout; session project bind; Data API authenticated path when gateway present
- [ ] 3.2 Sync `docs/prd/mvp-phases.md` Phase 5 App Auth acceptance items
- [ ] 3.3 `cargo fmt`, `cargo clippy -p api -- -D warnings`, `cargo test -p api`

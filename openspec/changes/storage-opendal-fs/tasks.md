## 1. Config & dependency

- [ ] 1.1 Add `opendal` dependency and `STORAGE_ROOT` (plus optional max upload size) to config + `.env.example`
- [ ] 1.2 Add `public.files` (or equivalent) metadata table via SeaQuery sync + migration

## 2. Storage service & routes

- [ ] 2.1 Implement OpenDAL fs operator namespaced by `project_id`
- [ ] 2.2 Implement list / multipart upload / download / delete Manager routes
- [ ] 2.3 Enforce role matrix (owner/admin full; member read-only writes denied)
- [ ] 2.4 Reject Publishable Key; allow Secret Key full CRUD with audit logging
- [ ] 2.5 OpenAPI annotations for Storage paths

## 3. Tests & PRD

- [ ] 3.1 Integration tests for role matrix and Publishable Key rejection
- [ ] 3.2 Sync `docs/prd/mvp-phases.md` Phase 4 acceptance
- [ ] 3.3 `cargo fmt`, `cargo clippy -p api -- -D warnings`, `cargo test -p api`

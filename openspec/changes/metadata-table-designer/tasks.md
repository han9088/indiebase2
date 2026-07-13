## 1. Schema & metadata

- [ ] 1.1 Add `public.table_metadata` / `column_metadata` (SeaQuery sync + production migration); include `allow_anon_read` default false
- [ ] 1.2 Define whitelisted column types and identifier validation helpers

## 2. Manager DDL APIs

- [ ] 2.1 Implement create/list/update/delete table endpoints under project-scoped Manager routes (Dashboard Session + membership)
- [ ] 2.2 Implement column add/alter/drop endpoints
- [ ] 2.3 On create: DDL in `proj_{ulid}`, metadata rows, GRANT as needed, ENABLE RLS + bootstrap policies
- [ ] 2.4 Implement `allow_anon_read` toggle that updates anon SELECT policy
- [ ] 2.5 Trigger PostgREST schema reload after DDL

## 3. Docs, tests, PRD

- [ ] 3.1 OpenAPI for table/column routes
- [ ] 3.2 Integration tests: create table → metadata match; anon deny / allow_anon_read SELECT; Dashboard operator CRUD / member read (requires Data API)
- [ ] 3.3 Sync `docs/prd/mvp-phases.md` Phase 3 acceptance
- [ ] 3.4 `cargo fmt`, `cargo clippy -p api -- -D warnings`, `cargo test -p api`

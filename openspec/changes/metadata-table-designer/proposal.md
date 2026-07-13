## Why

Data API can proxy CRUD only after tenant tables exist with bootstrap RLS. Platform members need Manager APIs to create/alter tables in `proj_{ulid}` and record metadata (`allow_anon_read`) so anon/authenticated/operator roles behave as specified in architecture §11.3.

## What Changes

- Platform metadata tables: `public.table_metadata`, `public.column_metadata` (including `allow_anon_read`).
- Manager APIs to create/modify/delete tables and columns; DDL executed in tenant schema `proj_{ulid}`.
- On create table: apply MVP bootstrap RLS policies (anon deny + opt-in SELECT; authenticated CRUD; operator roles).
- Toggle `allow_anon_read` updates RLS accordingly.
- OpenAPI for table/column Manager routes; sync `docs/prd/mvp-phases.md` Phase 3 acceptance.
- Depends on `data-api-gateway` for Row Viewer / SDK smoke against new tables.

## Capabilities

### New Capabilities

- `metadata-table-designer`: Manager table/column DDL, metadata persistence, bootstrap RLS templates, `allow_anon_read`.

### Modified Capabilities

- `api-docs`: Document `/api/tables*` (or equivalent Manager paths) and metadata schemas.
- `project-lifecycle`: Ensure create-project provisioning remains compatible with bootstrap RLS role grants (delta only if grants/permissions change).

## Non-goals

- Dashboard UI for Table Designer (API-only MVP).
- Policy / ABAC editor (`todo.md`).
- Row Viewer impersonation as App user.
- App User Session issuance (`app-user-session`) — authenticated smoke may use hand-minted Redis tokens until that change lands.
- Storage, TS SDK package.

## Impact

- `crates/api`: metadata schema (SeaQuery sync + production migrations), DDL service, RLS templates, routes.
- Postgres: `public.*_metadata`; per-table policies in `proj_{ulid}`.
- PRD Phase 3; architecture §11.3 already defines behavior.

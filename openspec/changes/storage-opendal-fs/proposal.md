## Why

Dashboard needs file upload/list/download/delete without going through PostgREST. Storage is a Manager API concern (OpenDAL `fs`) and unblocks Phase 4 acceptance while staying out of the Data API path.

## What Changes

- OpenDAL default `fs` backend rooted under a configurable local directory, namespaced by `{project_id}/…`.
- Manager routes: `GET/POST /api/projects/{project_id}/files`, `GET/DELETE /api/projects/{project_id}/files/{file_id}`.
- Auth: Dashboard Session + `project_members` role matrix (owner/admin full; member list+download only). Optional Secret Key S2S full CRUD with audit (architecture §10.1).
- Reject Publishable Key on Storage routes.
- File metadata persistence as needed for list/download/delete by `file_id`.
- OpenAPI + sync `docs/prd/mvp-phases.md` Phase 4 acceptance.
- Depends on Phase 1 project/session; **not** blocked by Data API for Manager-only smoke (may land in parallel after gateway if preferred).

## Capabilities

### New Capabilities

- `storage-opendal-fs`: Manager file APIs, OpenDAL fs backend, project-scoped keys, role-based access.

### Modified Capabilities

- `api-docs`: Document Storage Manager paths.
- `env-config`: Storage root path / OpenDAL config vars.

## Non-goals

- SeaweedFS / S3 backend (optional later).
- Client SDK Storage / Signed URL protocol (`todo.md` §6).
- Serving files through `/api/data/*` or PostgREST.
- Multipart signed-upload URL flow beyond first-version multipart upload.

## Impact

- `crates/api`: storage module, OpenDAL dependency, file metadata table/migration, routes.
- Local filesystem volume for dev; `.env.example`.
- PRD Phase 4.

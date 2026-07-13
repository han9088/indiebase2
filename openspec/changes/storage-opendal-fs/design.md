## Context

Architecture §10 defines Storage as Manager API + OpenDAL, not Data API. Phase 1 auth/project exist; no file routes yet. Default backend is local `fs`.

## Goals / Non-Goals

**Goals:**

- OpenDAL `fs` with root from env; keys `{project_id}/{file_id}` (or nested path).
- Upload (multipart), list, download, delete under `/api/projects/{project_id}/files*`.
- Role matrix: owner/admin full; member read; Publishable Key rejected; Secret Key optional full + audit.

**Non-Goals:** SeaweedFS, client Signed URL Storage, Data API file routes.

## Decisions

### 1. Metadata store

- **Decision:** `public.files` (or `storage_objects`) with `id`, `project_id`, `object_key`, `filename`, `content_type`, `size`, `created_by`, timestamps, `deleted_at`.
- **Alternatives:** Filesystem-only listing — weaker for `file_id` API.

### 2. Auth extractors

- **Decision:** Reuse Dashboard Session + membership for project path; separate Secret Key extractor for S2S if implemented in same change (recommended for §10.1 completeness).

### 3. OpenDAL

- **Decision:** Add `opendal` crate; `Operator` in `AppState` or per-request from config root.

### 4. Upload size limits

- **Decision:** Configurable max body size (env); reject oversized with 413.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Path traversal | Normalize keys; never accept client absolute paths |
| Orphan objects | Delete DB row and object in ordered cleanup |
| Disk fill | Document limits; optional quota later |

## Migration Plan

1. Migration + env `STORAGE_ROOT`.
2. Routes + OpenAPI.
3. Role matrix tests.

## Open Questions

- Whether Secret Key Storage is mandatory in first merge or follow-up task — include in tasks as required for architecture MVP Storage acceptance.

## Context

Dashboard Session exists; App User Session does not. Data API gateway (when present) maps Publishable + App User Bearer → `auth_mode=authenticated`. Architecture §11.9 defines Redis `app_user_session:` and Project Auth routes.

## Goals / Non-Goals

**Goals:**

- Mint/revoke App User Sessions; store end users in tenant schema (minimal).
- Login/logout Manager Auth routes under `/api/auth/app/*`.
- Payload includes `end_user_id`, `project_id`, `role` for gateway SET ROLE path.

**Non-Goals:** OAuth, email verification, TS SDK, ABAC, Dashboard Session changes.

## Decisions

### 1. Where end users live

- **Decision:** Table `proj_{ulid}.app_users` (or `users` if reserved carefully) created at project provision **or** lazily on first App Auth enable. Prefer provisioning in project create (delta to lifecycle) or metadata bootstrap — **lazy create on first login setup** is weaker; **add to `provision_schema`** in this change.
- **Fields:** `id` ULID, `email`, `password_hash`, timestamps, soft delete.

### 2. Login body

- **Decision:** `{ project_id, email, password }` + require Publishable Key header so anonymous clients cannot enumerate without Key (align with SDK: Key always present).
- **Alternatives:** Login without Key — weaker; reject.

### 3. Token minting

- **Decision:** Same opaque random token + Redis TTL pattern as Dashboard Session; prefix `app_user_session:`.

### 4. Seed for tests

- **Decision:** Integration test creates project, inserts app user via SQL/helper, logs in, hits Data API.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Confusing platform `users` vs app users | Separate table name `app_users`; docs |
| Login without gateway | Still ship Auth; E2E after gateway |
| Password policy | Minimal length check for MVP |

## Migration Plan

1. Extend provisioning for `app_users` + grants.
2. Auth routes + Redis.
3. E2E with Data API authenticated mode.

## Open Questions

- Table name `app_users` vs `users` — prefer `app_users` to avoid clashing with customer-created `users` tables from Table Designer.

# PostgREST schema registration + reload (Phase 1)

## Chosen approach

1. **Schemas list file** — `POSTGREST_SCHEMAS_FILE` (default `./docker/postgrest/db-schemas`), comma-separated (`public,proj_…`). Mounted into the PostgREST container as `/config/db-schemas`.
2. **File-based PostgREST config** — `POSTGREST_CONFIG_PATH` (default `./docker/postgrest/postgrest.conf`). Entrypoint writes `db-uri` / `db-schemas` / `db-anon-role` on start; the API **only updates the `db-schemas` line** after project create (never drops `db-uri`) so `NOTIFY reload config` re-reads the list.
3. **Reload** — after appending the new `proj_{ulid}` schema, the API runs:
   - `SELECT pg_notify('pgrst', 'reload config');`
   - `SELECT pg_notify('pgrst', 'reload schema');`

If the schemas/config file update fails (permissions, path), create still succeeds and logs a warning; operators can restart the `postgrest` compose service as a fallback.

**Compose pitfall:** do not set `PGRST_DB_SCHEMAS` on the PostgREST service — env vars override the config file, so `NOTIFY reload config` would keep exposing only the env value (e.g. `public`).

## Alternatives considered

| Option | Why not for Phase 1 |
|--------|---------------------|
| Static `PGRST_DB_SCHEMAS` only | New project schemas never appear without container restart + env edit |
| `PGRST_DB_SCHEMAS=*` | Not supported as a dynamic “all schemas” wildcard for this use case |
| Admin HTTP reload only | Less reliable across compose networking; NOTIFY is the documented PostgREST path |

## Local smoke

After `just up` + `just run` and creating a project:

```bash
# schemas file should include proj_{ulid}
cat docker/postgrest/db-schemas

# PostgREST OpenAPI lists the schema (may take a moment after NOTIFY)
curl -s "$POSTGREST_URL/" | head
```

Manual fallback: `docker compose --env-file .env.development restart postgrest`.

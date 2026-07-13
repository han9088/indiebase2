# PostgREST schema sync + reload

## Model

- **Source of truth:** `public.projects` (`deleted_at IS NULL` → schema `proj_{id}`).
- **Delivery:** PostgREST in-DB config via `db-pre-config = public.indiebase_pre_config`, which runs:

```sql
PERFORM set_config('pgrst.db_schemas', 'public,proj_…', true);
```

derived from `projects`. After create / API startup we only:

```sql
NOTIFY pgrst, 'reload config';
NOTIFY pgrst, 'reload schema';
```

No `db-schemas` file and no API rewriting of schema lists in conf.

## Entrypoint conf (secrets only)

`docker/postgrest/entrypoint.sh` still writes a **minimal** `postgrest.conf` for `db-uri`, JWT, `db-pre-config`, `db-pre-request`. Bootstrap `db-schemas = "public"` is overridden by `indiebase_pre_config` on load/reload.

**Compose pitfall:** do not set `PGRST_DB_SCHEMAS` on the PostgREST service — env would override in-DB config.

## Local smoke

```bash
just up && just run
# create a project, then:
curl -s "$POSTGREST_URL/" | head
```

Manual fallback: `docker compose --env-file .env.development restart postgrest`.

## Env (Data API)

| Variable | Purpose |
|----------|---------|
| `POSTGREST_JWT_SECRET` | HS256 secret shared by API (authenticator JWT) and PostgREST `jwt-secret` |
| `INDIEBASE_INTERNAL_CONTEXT_SECRET` | HMAC secret for `X-Indiebase-Internal-Context` (API + `gateway_config` row) |

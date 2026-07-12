## Context

Config must follow [Vite env + modes](https://cn.vite.dev/guide/env-and-mode): file layers by environment name, process env wins. Compose and API share `.env.development` for local Orb / docker stacks.

## Goals / Non-Goals

**Goals:**

- `INDIEBASE_ENV` selects which mode files load (`development` default).
- Vite four-layer load; process env never overwritten by files.
- Compose: only `--env-file .env.development` for local.
- gitignore matches Vite local-secret convention.

**Non-Goals:** Separate NODE_ENV axis; staging; vault.

## Decisions

### 1. Variable name

- **Decision:** `INDIEBASE_ENV` (not `INDIEBASE_MODE` / `APP_ENV`).
- **Rationale:** Namespaced; matches `INDIEBASE_HTTP_ADDR`; one knob for MVP.

### 2. Allowed values

- `development` (default), `production`.
- Unknown values still accepted as opaque strings for forward compat (file name = value).

### 3. Load order (Vite)

```text
.env → .env.local → .env.{INDIEBASE_ENV} → .env.{INDIEBASE_ENV}.local
```

Later files override earlier among files. Keys already in the process environment are skipped.

### 4. Compose

- Sole local command: `docker compose --env-file .env.development up -d`.
- Do not document `.env.dev` or bare `docker compose` without `--env-file`.

### 5. gitignore

```gitignore
.env.local
.env.*.local
```

Do **not** ignore `.env.*` wholesale (that would hide `.env.development`). Root `.env` may remain ignored if present as a personal catch-all; committed templates are `.env.example` + `.env.development` (and later `.env.production` without live secrets).

### 6. Connection settings

Discrete `POSTGRES_*` / `REDIS_*` / `POSTGREST_URL`; `database_url()` / `redis_url()` percent-encode passwords.

## Risks

| Risk | Mitigation |
|------|------------|
| Committing real prod secrets in `.env.production` | Only commit non-secret templates; secrets in `*.local` |
| Docs still say `.env.dev` | Sweep README / AGENTS / PRD / compose comment |

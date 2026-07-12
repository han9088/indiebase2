## Why

Local API and Docker Compose need Vite-style per-environment env files (`development` / `production`) so developers can switch profiles without editing one shared secret file. An earlier short-name layout (`dev` / `.env.dev`) and a temporary `INDIEBASE_MODE` rename drifted from the agreed convention.

## What Changes

- Use **`INDIEBASE_ENV`** as the sole selector (Vite “mode” role). Default: `development`. Supported values for now: `development` | `production`.
- Load env files like Vite: `.env` → `.env.local` → `.env.[env]` → `.env.[env].local`. Process env already set is never overwritten.
- Expose `Config.env` as the active environment string.
- **Compose sole convention:** `docker compose --env-file .env.development up -d`.
- **gitignore:** Vite-style — ignore only `*.local` (`.env.local`, `.env.*.local`); allow committing `.env.development` / `.env.production` templates.
- Docs: README, AGENTS.md, Phase 0 PRD, `.env.example`, `openspec/config.yaml`.
- Postgres / Redis stay as discrete fields; URIs derived in code.

## Capabilities

### New Capabilities

- `env-config`: Vite-style dotenv layers selected by `INDIEBASE_ENV`.

### Modified Capabilities

- (none — `server-bootstrap` not yet in main specs)

## Non-goals

- Separate `NODE_ENV` / `INDIEBASE_NODE_ENV` dual axis (defer).
- Staging mode / custom env names beyond `development` | `production`.
- Secrets manager / vault.
- Multiple compose env-file conventions.

## Impact

- `crates/api/src/config.rs`, `.gitignore`, `.env.example`, `.env.development`, README, `AGENTS.md`, `docs/prd/mvp-phases.md`, `docker-compose.yaml`, `openspec/config.yaml`.

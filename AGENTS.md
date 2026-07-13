# AGENTS.md — Indiebase

Instructions for AI coding agents working in this repository.

## Project

**Indiebase** is a self-hosted BaaS for indie developers and small teams (Rust rewrite).

| Area | Detail |
|------|--------|
| Stack | Axum, sqlx, SeaQuery, OpenDAL, ulid; Postgres 17, Redis, PostgREST |
| Server crate | `crates/api` — run with `cargo run -p api` |
| Manager API | Dashboard routes (`/api/projects`, `/api/tables`, `/api/auth/*`, …) — **not** `/api/data/*` |
| Data API | CRUD gateway `/api/data/*`, PostgREST proxy; Dashboard Session or SDK API Key |
| Isolation | Schema-per-project `proj_{ulid}`; platform tables in `public` |
| Auth | No JWT; Opaque Token + Redis (Dashboard / App User); project via `X-Indiebase-Project-Id` |
| Local infra | `docker compose --env-file .env.development up -d`; Vite-style env via `INDIEBASE_ENV` |

## Language

- **Reply to users** in Simplified Chinese.
- **Source code, comments, commit messages** in English.

## How to work (default)

**Ship the requested change directly.** Do not require a failing test or OpenSpec before editing unless the user asks.

### PRD sync (mandatory when behavior changes)

If you change **observable product behavior**, update PRD in the **same change**:

| Topic | File |
|-------|------|
| MVP phases & acceptance | `docs/prd/mvp-phases.md` |
| Architecture, sessions, Data API | `docs/prd/baas-platform-architecture.md` |
| TS SDK contract | `docs/prd/mvp-sdk.md` |
| Post-MVP backlog | `docs/prd/todo.md` |

**Triggers:** new/changed routes, env vars, auth rules, phase scope, SDK contract, PostgREST/RLS semantics.

**Not PRD:** refactors, internal layout, test-only changes.

Also update `.env.example` when adding env vars.

Details: `.cursor/rules/backend-tdd-prd.mdc`.

### OpenSpec

Use OpenSpec when the user asks (`/opsx:*`, "用 openspec", etc.) — see `openspec/` and `openspec/config.yaml`.

- Main specs are **capability-based** (`server-bootstrap`, `platform-auth`, …), not Phase-numbered.
- Name changes by capability/intent (e.g. `data-api-gateway`); never `phase-N-*`.
- Product delivery checklist stays in `docs/prd/mvp-phases.md`; architecture in `docs/prd/baas-platform-architecture.md`.

## API documentation

- **OpenAPI:** code-first via `utoipa`; served at `GET /openapi.json`
- **Interactive docs:** [Scalar](https://scalar.com/) at `GET /docs`
- **Conventions:** `.cursor/rules/openapi-utoipa.mdc` — `summary` / `description` / `operation_id`, schema field docs + examples
- New routes: follow that rule; add a covering test when useful

## Code organization

- Split constants by domain (`constants/http.rs`, not one giant file).
- One concern per file; split when layers mix (types, constants, I/O).
- Colocate types until the file is hard to scan; then extract `*.types.rs` / `schema.rs`.
- Pure helpers in dedicated modules; keep route handlers thin.
- Names read as intent (`build_session_cookie`, not `do_stuff`).
- Prefer early returns over deep nesting.

Details: `.cursor/rules/code-organization.mdc`.

## Tooling

```bash
just              # list recipes
just run          # INDIEBASE_ENV=development cargo run -p api
just watch        # cargo watch -x 'run -p api' (development)
just run-prod     # INDIEBASE_ENV=production
just up           # docker compose --env-file .env.development up -d
just test         # cargo test -p api
```

Install: `brew install just`. Watch needs `cargo install cargo-watch`.

Or directly:

```bash
cargo fmt
cargo clippy -- -D warnings   # when verifying
cargo test -p api
```

Local API: copy `.env.example` → `.env.development`, then `cargo run -p api`.
`INDIEBASE_ENV` defaults to `development` (`production` also supported). Loads Vite layers: `.env` → `.env.local` → `.env.[env]` → `.env.[env].local`.
Compose: `docker compose --env-file .env.development up -d` (sole convention).

- Health: `GET /health`
- OpenAPI: `GET /openapi.json`
- Docs: `GET /docs`

## Key docs

| Doc | Purpose |
|-----|---------|
| `docs/prd/mvp-phases.md` | MVP delivery checklist + OpenSpec capability naming |
| `docs/prd/baas-platform-architecture.md` | Full platform architecture |
| `docs/prd/mvp-sdk.md` | MVP TypeScript SDK |
| `openspec/config.yaml` | OpenSpec context; main specs under `openspec/specs/` |
| `.cursor/rules/` | Persistent Cursor rules |

## Guardrails

- Change only what the task requires.
- Do not commit unless the user asks.
- No JWT; Dashboard Session + `X-Indiebase-Project-Id` for project scope; respect §6.2.3 mutual-exclusion.
- Project API Key is SDK auth only — it does **not** resolve project context.

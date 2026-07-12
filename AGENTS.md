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
| Auth | No JWT; Opaque Token + Redis (Dashboard / Project / App User sessions) |
| Local infra | `docker compose --env-file .env.dev up -d`; env in `.env.{INDIEBASE_ENV}` |

## Language

- **Reply to users** in Simplified Chinese.
- **Source code, comments, commit messages** in English.

## How to work (default)

**Ship the requested change directly.** Do not require OpenSpec, a failing test, or PRD updates before editing unless the user asks.

| Tooling | When |
|---------|------|
| OpenSpec | Only if the user asks (`/opsx:*`, "用 openspec", etc.) — see `.cursor/rules/openspec-workflow.mdc` |
| Tests | Recommended for regressable backend behavior; TDD optional — see `.cursor/rules/backend-tdd-prd.mdc` |
| PRD | Update when observable product behavior changes and it is practical — not a blocker for small fixes |

## API documentation

- **OpenAPI:** code-first via `utoipa`; served at `GET /openapi.json`
- **Interactive docs:** [Scalar](https://scalar.com/) at `GET /docs`
- New routes: prefer `#[utoipa::path]` + a covering test

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
cargo fmt
cargo clippy -- -D warnings   # when verifying
cargo test -p api
```

Local API: copy `.env.example` → `.env.dev`, then `cargo run -p api`.
`INDIEBASE_ENV` defaults to `dev` and loads `.env.{env}` only (no `.env`).
Compose: `docker compose --env-file .env.dev up -d`.

- Health: `GET /health`
- OpenAPI: `GET /openapi.json`
- Docs: `GET /docs`

## Key docs

| Doc | Purpose |
|-----|---------|
| `docs/prd/mvp-phases.md` | MVP Phase 0–5 breakdown |
| `docs/prd/baas-platform-architecture.md` | Full platform architecture |
| `docs/prd/mvp-sdk.md` | MVP TypeScript SDK |
| `openspec/config.yaml` | OpenSpec project context (when using OpenSpec) |
| `.cursor/rules/` | Persistent Cursor rules |

## Guardrails

- Change only what the task requires.
- Do not commit unless the user asks.
- No JWT; respect session / credential mutual-exclusion rules in architecture §6.2.3.
- Project API Key is SDK auth only — it does **not** resolve project context.

<a href='https://indiebase.deskbtm.com' target="_blank">
<img src="https://user-images.githubusercontent.com/45007226/255768134-e4d4a832-3979-4534-9b81-34fbfa91aab3.svg" />
</a>
<br />

[![X (formerly Twitter) Follow](https://img.shields.io/twitter/follow/deskbtm?style=social)](https://twitter.com/intent/follow?screen_name=deskbtm)

> [!TIP]
> Early-stage rewrite: Indiebase is being rebuilt on Rust. Phase 1 (`crates/api`) adds Dashboard/Project opaque sessions, project create (`proj_{ulid}`), and Manager routes under `/api/auth/*` and `/api/projects`.

Indiebase is a self-hosted BaaS platform for indie developers and small teams.

## Local development (API)

1. Start infrastructure:

   ```bash
   just up
   # or: docker compose --env-file .env.development up -d
   ```

2. Configure environment:

   ```bash
   cp .env.example .env.development
   ```

   Vite-style files via `INDIEBASE_ENV` (default `development`): `.env` → `.env.local` → `.env.[env]` → `.env.[env].local`. Process env wins.

3. Migrate + seed (also runs on API startup):

   ```bash
   just migrate
   ```

   Development: SeaQuery **synchronize** (TypeORM-like) — editing `crates/api/src/db/schema.rs` recreates platform tables when DDL hash changes.  
   Production: sqlx migrations under `crates/api/migrations/`.

   Dev seed user: `dev@indiebase.com` / `dev@indiebase.com`.

4. Run the API server:

   ```bash
   just run
   # or: just watch          # reload on change (needs cargo-watch)
   # or: just run-prod
   ```

   Other tasks: `just` (`up` / `down` / `migrate` / `smoke-login` / `test` / `clippy` / `fmt`). Install: `brew install just`.

5. Health check:

   ```bash
   curl -s http://localhost:8080/health
   ```

6. API docs:

   ```bash
   curl -s http://localhost:8080/openapi.json | head
   open http://localhost:8080/docs   # Scalar UI
   ```

7. Login smoke (API must be running):

   ```bash
   just smoke-login
   # or manually:
   curl -s -X POST http://localhost:8080/api/auth/login \
     -H 'content-type: application/json' \
     -d '{"email":"dev@indiebase.com","password":"dev@indiebase.com"}'
   ```

PostgREST schema reload after project create: [docs/notes/postgrest-schema-reload.md](./docs/notes/postgrest-schema-reload.md).

MVP phase breakdown: [docs/prd/mvp-phases.md](./docs/prd/mvp-phases.md).

Agent / AI workflow: [AGENTS.md](./AGENTS.md).

## FAQ

- What's Indiebase?  
  Indiebase is a self-hosted BaaS for indie hackers and small teams — similar to a private Firebase. This repository is the FOSS edition, available under [AGPL-3.0](./LICENSE). The initial purpose of Indiebase was to serve [deskbtm](https://deskbtm.com), used for managing [Nawb](https://nawb.deskbtm.com/), [PlugKit](https://github.com/deskbtm-plugkit/plugkit), etc.
- Does Indiebase provide an online service?  
  Nope, Indiebase only provides the self-hosted service. Ensure the functionality while making it capable of running on low-configured server environments as much as possible.

## Profits

- For indie developers/small teams: Let passionate developers collaborate on profitable independent projects during their spare time. Reducing development costs through BaaS.
- For programming geeks: By discovering projects you love in Indiebase, contributing code, and earning rewards from it.
- For companies: Allow developers with spare time to participate in improving open-source projects and provide compensation to the developers.

> [!IMPORTANT]
>
> - Indiebase highly depends on Github and considers it as the default code management.
> - `Indiebase` (this repo) is available under the [AGPL-3.0](./LICENSE) license.


## Join Indiebase

`Indiebase` is PR welcome.  
<br />
Email: [indiebase@deskbtm.com](mailto:indiebase@deskbtm.com)

## Discussion Etiquette

In order to keep the conversation clear and transparent, please limit the discussion to English and keep things on topic with the issue. Non-compliant issues may be closed directly. Be considerate to others and try to be courteous and professional at all times.

## License

If a directory has a LICENSE file, the project is governed by that LICENSE file. The rest of the parts are licensed under the GNU AFFERO GENERAL PUBLIC LICENSE (AGPL-3.0) license.

Copyright (C) 2021 Han <indiebase@deskbtm.com>

## 1. TDD — Config environment

- [x] 1.1 Write failing tests: default `INDIEBASE_ENV` → `development`; explicit `production`; `env` field on Config
- [x] 1.2 Write failing test for Vite layered load override + process-env wins
- [x] 1.3 Implement `INDIEBASE_ENV` + Vite four-layer dotenv loading in `config.rs`

## 2. Env files, gitignore & docs

- [x] 2.1 Align `.gitignore` with Vite (`*.local` only for env secrets)
- [x] 2.2 Update `.env.example` + `.env.development` (`INDIEBASE_ENV`, Vite layout)
- [x] 2.3 Compose sole convention: `--env-file .env.development`
- [x] 2.4 Update README, AGENTS.md, `docs/prd/mvp-phases.md`, `openspec/config.yaml`

## 3. Verification

- [x] 3.1 `cargo test -p api` and `cargo clippy -p api -- -D warnings`

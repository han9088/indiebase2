# Indiebase — task runners (npm scripts equivalent)
# Usage: just <recipe>
# Install: brew install just
# Watch needs: cargo install cargo-watch

set dotenv-load := false
set shell := ["zsh", "-cu"]

default:
    @just --list

# API with INDIEBASE_ENV=development
run:
    INDIEBASE_ENV=development cargo run -p api

# API with INDIEBASE_ENV=production
run-prod:
    INDIEBASE_ENV=production cargo run -p api

# Watch + reload API (development)
watch:
    INDIEBASE_ENV=development cargo watch -x 'run -p api'

# Watch + reload API (production env)
watch-prod:
    INDIEBASE_ENV=production cargo watch -x 'run -p api'

# Start Postgres / Redis / PostgREST
up:
    docker compose --env-file .env.development up -d

# Stop compose stack
down:
    docker compose --env-file .env.development down

# Apply platform schema + dev seed (also runs on API startup)
# development: SeaQuery synchronize; production: sqlx migrations
migrate:
    INDIEBASE_ENV=development cargo run -p api -- --migrate-only

# cargo test -p api
test:
    cargo test -p api

# Watch tests
test-watch:
    cargo watch -x 'test -p api'

# cargo clippy -p api
clippy:
    cargo clippy -p api -- -D warnings

# cargo fmt
fmt:
    cargo fmt

# Smoke: login → create project → list → project-context (API must be running)
smoke-login:
    #!/usr/bin/env zsh
    set -euo pipefail
    TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
      -H 'content-type: application/json' \
      -d '{"email":"dev@indiebase.com","password":"dev@indiebase.com"}' \
      | python3 -c 'import sys,json; print(json.load(sys.stdin)["token"])')
    echo "dashboard token: ${TOKEN:0:16}…"
    CREATE=$(curl -s -X POST http://localhost:8080/api/projects \
      -H "authorization: Bearer $TOKEN" \
      -H 'content-type: application/json' \
      -d '{"name":"smoke-project"}')
    echo "$CREATE" | python3 -m json.tool
    PID=$(echo "$CREATE" | python3 -c 'import sys,json; print(json.load(sys.stdin)["id"])')
    curl -s http://localhost:8080/api/projects -H "authorization: Bearer $TOKEN" | python3 -m json.tool
    curl -s http://localhost:8080/api/auth/project-context \
      -H "authorization: Bearer $TOKEN" \
      -H "x-indiebase-project-id: $PID" | python3 -m json.tool

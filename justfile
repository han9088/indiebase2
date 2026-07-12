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

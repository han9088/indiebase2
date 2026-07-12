#!/bin/sh
set -eu

# Build PGRST_DB_URI from Docker secret + env (no hardcoded user/db/host/port).
pass_raw=$(tr -d '\n\r' < /run/secrets/pg_password)

urlencode() {
	_s=$1
	_s=$(printf '%s' "$_s" | sed \
		-e 's/%/%25/g' \
		-e 's/@/%40/g' \
		-e 's/:/%3A/g' \
		-e 's#/#%2F#g' \
		-e 's/?/%3F/g' \
		-e 's/#/%23/g' \
		-e 's/\[/%5B/g' \
		-e 's/\]/%5D/g' \
		-e 's/+/%2B/g' \
		-e 's/ /%20/g' \
		-e 's/&/%26/g' \
		-e 's/=/%3D/g')
	printf '%s' "$_s"
}

: "${POSTGRES_USER:?POSTGRES_USER is required}"
: "${POSTGRES_DB:?POSTGRES_DB is required}"
: "${PGRST_DB_HOST:?PGRST_DB_HOST is required}"
: "${PGRST_DB_PORT:?PGRST_DB_PORT is required}"

pass_enc=$(urlencode "$pass_raw")
export PGRST_DB_URI="postgres://${POSTGRES_USER}:${pass_enc}@${PGRST_DB_HOST}:${PGRST_DB_PORT}/${POSTGRES_DB}?sslmode=disable"

exec postgrest "$@"

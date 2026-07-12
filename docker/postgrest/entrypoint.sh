#!/bin/sh
set -eu

# Build PostgREST config file from Docker secret + env.
# File-based config enables `NOTIFY pgrst, 'reload config'` to pick up schema list changes
# written by the API into the mounted db-schemas / postgrest.conf files.
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
db_uri="postgres://${POSTGRES_USER}:${pass_enc}@${PGRST_DB_HOST}:${PGRST_DB_PORT}/${POSTGRES_DB}?sslmode=disable"

schemas_file="/config/db-schemas"
conf_file="/config/postgrest.conf"

if [ -f "$schemas_file" ]; then
	schemas=$(tr -d '\n\r' < "$schemas_file" | sed 's/[[:space:]]//g')
else
	schemas="${PGRST_DB_SCHEMAS:-public}"
	printf '%s\n' "$schemas" > "$schemas_file"
fi

anon_role="${PGRST_DB_ANON_ROLE:-anon}"
openapi_proxy="${PGRST_OPENAPI_SERVER_PROXY_URI:-}"

{
	printf 'db-uri = "%s"\n' "$db_uri"
	printf 'db-schemas = "%s"\n' "$schemas"
	printf 'db-anon-role = "%s"\n' "$anon_role"
	if [ -n "$openapi_proxy" ]; then
		printf 'openapi-server-proxy-uri = "%s"\n' "$openapi_proxy"
	fi
} > "$conf_file"

exec postgrest "$conf_file"

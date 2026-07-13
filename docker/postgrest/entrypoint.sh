#!/bin/sh
set -eu

# Minimal PostgREST conf from Docker secret + env.
# Schema list is NOT file-based: public.indiebase_pre_config sets pgrst.db_schemas
# from public.projects on every config load / NOTIFY reload config.
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
: "${POSTGREST_JWT_SECRET:?POSTGREST_JWT_SECRET is required}"

pass_enc=$(urlencode "$pass_raw")
db_uri="postgres://${POSTGRES_USER}:${pass_enc}@${PGRST_DB_HOST}:${PGRST_DB_PORT}/${POSTGRES_DB}?sslmode=disable"

conf_file="/config/postgrest.conf"
anon_role="${PGRST_DB_ANON_ROLE:-anon}"
openapi_proxy="${PGRST_OPENAPI_SERVER_PROXY_URI:-}"
jwt_secret="${POSTGREST_JWT_SECRET}"

{
	printf 'db-uri = "%s"\n' "$db_uri"
	# Bootstrap only — overridden by db-pre-config from projects on load/reload.
	printf 'db-schemas = "public"\n'
	printf 'db-anon-role = "%s"\n' "$anon_role"
	printf 'db-pre-config = "public.indiebase_pre_config"\n'
	printf 'db-pre-request = "public.indiebase_pre_request"\n'
	printf 'jwt-secret = "%s"\n' "$jwt_secret"
	if [ -n "$openapi_proxy" ]; then
		printf 'openapi-server-proxy-uri = "%s"\n' "$openapi_proxy"
	fi
} > "$conf_file"

exec postgrest "$conf_file"

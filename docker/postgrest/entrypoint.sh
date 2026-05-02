#!/bin/sh
set -eu

# Build PGRST_DB_URI from Docker secret; percent-encode password for URI safety.
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

pass_enc=$(urlencode "$pass_raw")
export PGRST_DB_URI="postgres://postgres:${pass_enc}@postgres:5432/indiebase-dev?sslmode=disable"

exec postgrest "$@"

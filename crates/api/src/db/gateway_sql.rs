//! PostgREST `db-pre-request` + gateway config SQL fragments (each is one statement).

pub const CREATE_PGCRYPTO: &str = "CREATE EXTENSION IF NOT EXISTS pgcrypto";

pub const CREATE_GATEWAY_CONFIG: &str = r#"
CREATE TABLE IF NOT EXISTS public.gateway_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
)
"#;

pub const CREATE_PRE_REQUEST_FN: &str = r#"
CREATE OR REPLACE FUNCTION public.indiebase_pre_request()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    headers json;
    raw_ctx text;
    payload_b64 text;
    sig_hex text;
    payload_bytes bytea;
    payload_json json;
    expected_sig text;
    secret text;
    auth_mode text;
    role_name text;
    user_id text;
    project_id text;
    project_role text;
    exp_ts bigint;
    now_ts bigint := extract(epoch from now())::bigint;
    dot_pos int;
BEGIN
    headers := current_setting('request.headers', true)::json;
    raw_ctx := headers->>'x-indiebase-internal-context';
    IF raw_ctx IS NULL OR length(raw_ctx) = 0 THEN
        RAISE insufficient_privilege USING MESSAGE = 'missing internal context';
    END IF;

    SELECT value INTO secret
    FROM public.gateway_config
    WHERE key = 'internal_context_secret';
    IF secret IS NULL OR length(secret) = 0 THEN
        RAISE insufficient_privilege USING MESSAGE = 'internal context secret not configured';
    END IF;

    dot_pos := position('.' in raw_ctx);
    IF dot_pos < 2 THEN
        RAISE insufficient_privilege USING MESSAGE = 'malformed internal context';
    END IF;

    payload_b64 := substr(raw_ctx, 1, dot_pos - 1);
    sig_hex := substr(raw_ctx, dot_pos + 1);

    payload_b64 := replace(replace(payload_b64, '-', '+'), '_', '/');
    WHILE length(payload_b64) % 4 <> 0 LOOP
        payload_b64 := payload_b64 || '=';
    END LOOP;

    BEGIN
        payload_bytes := decode(payload_b64, 'base64');
    EXCEPTION WHEN OTHERS THEN
        RAISE insufficient_privilege USING MESSAGE = 'invalid internal context payload encoding';
    END;

    expected_sig := encode(hmac(payload_bytes, convert_to(secret, 'UTF8'), 'sha256'), 'hex');
    IF expected_sig IS DISTINCT FROM lower(sig_hex) THEN
        RAISE insufficient_privilege USING MESSAGE = 'invalid internal context signature';
    END IF;

    payload_json := convert_from(payload_bytes, 'UTF8')::json;
    auth_mode := payload_json->>'auth_mode';
    user_id := payload_json->>'user_id';
    project_id := payload_json->>'project_id';
    project_role := payload_json->>'project_role';
    exp_ts := COALESCE((payload_json->>'exp')::bigint, 0);

    IF exp_ts > 0 AND exp_ts < now_ts THEN
        RAISE insufficient_privilege USING MESSAGE = 'internal context expired';
    END IF;

    IF auth_mode IS NULL OR length(auth_mode) = 0 THEN
        RAISE insufficient_privilege USING MESSAGE = 'missing auth_mode';
    END IF;

    role_name := CASE auth_mode
        WHEN 'anon' THEN 'anon'
        WHEN 'authenticated' THEN 'authenticated'
        WHEN 'project_operator' THEN 'project_operator'
        WHEN 'project_operator_readonly' THEN 'project_operator_readonly'
        WHEN 'service' THEN 'service'
        ELSE NULL
    END;

    IF role_name IS NULL THEN
        RAISE insufficient_privilege USING MESSAGE = 'unknown auth_mode';
    END IF;

    PERFORM set_config('app.auth_mode', auth_mode, true);
    PERFORM set_config('app.project_id', COALESCE(project_id, ''), true);
    PERFORM set_config('app.user_id', COALESCE(user_id, ''), true);
    PERFORM set_config('app.project_role', COALESCE(project_role, ''), true);
    PERFORM set_config('app.role', role_name, true);

    EXECUTE format('SET LOCAL ROLE %I', role_name);
END;
$$
"#;

pub const REVOKE_PRE_REQUEST: &str =
    "REVOKE ALL ON FUNCTION public.indiebase_pre_request() FROM PUBLIC";

pub const GRANT_PRE_REQUEST_AUTHENTICATOR: &str =
    "GRANT EXECUTE ON FUNCTION public.indiebase_pre_request() TO authenticator";

pub const GRANT_PRE_REQUEST_POSTGRES: &str =
    "GRANT EXECUTE ON FUNCTION public.indiebase_pre_request() TO postgres";

pub const CREATE_PRE_CONFIG_FN: &str = r#"
CREATE OR REPLACE FUNCTION public.indiebase_pre_config()
RETURNS void
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = public
AS $$
DECLARE
    schemas text := 'public';
BEGIN
    -- Before platform tables exist (first boot), keep bootstrap schema only.
    IF to_regclass('public.projects') IS NOT NULL THEN
        SELECT 'public' || COALESCE(
            (
                SELECT ',' || string_agg('proj_' || id, ',' ORDER BY id)
                FROM projects
                WHERE deleted_at IS NULL
            ),
            ''
        )
        INTO schemas;
    END IF;

    PERFORM set_config('pgrst.db_schemas', schemas, true);
END;
$$
"#;

pub const REVOKE_PRE_CONFIG: &str =
    "REVOKE ALL ON FUNCTION public.indiebase_pre_config() FROM PUBLIC";

pub const GRANT_PRE_CONFIG_AUTHENTICATOR: &str =
    "GRANT EXECUTE ON FUNCTION public.indiebase_pre_config() TO authenticator";

pub const GRANT_PRE_CONFIG_POSTGRES: &str =
    "GRANT EXECUTE ON FUNCTION public.indiebase_pre_config() TO postgres";

/// Hash input for development schema sync (order matters).
pub fn gateway_sql_hash_input() -> String {
    [
        CREATE_PGCRYPTO,
        CREATE_GATEWAY_CONFIG,
        CREATE_PRE_REQUEST_FN,
        REVOKE_PRE_REQUEST,
        GRANT_PRE_REQUEST_AUTHENTICATOR,
        GRANT_PRE_REQUEST_POSTGRES,
        CREATE_PRE_CONFIG_FN,
        REVOKE_PRE_CONFIG,
        GRANT_PRE_CONFIG_AUTHENTICATOR,
        GRANT_PRE_CONFIG_POSTGRES,
    ]
    .join("\n")
}

pub async fn apply_data_api_gateway_sql(pool: &sqlx::PgPool) -> Result<(), String> {
    for (label, sql) in [
        ("pgcrypto", CREATE_PGCRYPTO),
        ("gateway_config", CREATE_GATEWAY_CONFIG),
        ("indiebase_pre_request", CREATE_PRE_REQUEST_FN),
        ("revoke pre_request", REVOKE_PRE_REQUEST),
        (
            "grant pre_request authenticator",
            GRANT_PRE_REQUEST_AUTHENTICATOR,
        ),
        ("grant pre_request postgres", GRANT_PRE_REQUEST_POSTGRES),
        ("indiebase_pre_config", CREATE_PRE_CONFIG_FN),
        ("revoke pre_config", REVOKE_PRE_CONFIG),
        (
            "grant pre_config authenticator",
            GRANT_PRE_CONFIG_AUTHENTICATOR,
        ),
        ("grant pre_config postgres", GRANT_PRE_CONFIG_POSTGRES),
    ] {
        sqlx::query(sql)
            .execute(pool)
            .await
            .map_err(|err| format!("data API gateway SQL ({label}) failed: {err}"))?;
    }
    Ok(())
}

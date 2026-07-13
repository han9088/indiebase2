-- Production / non-development path (sqlx migrate).
-- Development uses SeaQuery synchronize from crates/api/src/db/schema.rs instead.
-- Keep this file aligned with schema.rs when cutting a production schema.

CREATE TABLE IF NOT EXISTS public.users (
    id CHAR(26) PRIMARY KEY,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX IF NOT EXISTS users_email_active_uidx
    ON public.users (email)
    WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS public.projects (
    id CHAR(26) PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS public.project_members (
    project_id CHAR(26) NOT NULL REFERENCES public.projects (id) ON DELETE CASCADE,
    user_id CHAR(26) NOT NULL REFERENCES public.users (id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK (role IN ('owner', 'admin', 'member')),
    deleted_at TIMESTAMPTZ,
    PRIMARY KEY (project_id, user_id)
);

CREATE TABLE IF NOT EXISTS public.api_keys (
    id CHAR(26) PRIMARY KEY,
    project_id CHAR(26) NOT NULL REFERENCES public.projects (id) ON DELETE CASCADE,
    key_type TEXT NOT NULL CHECK (key_type IN ('publishable', 'secret')),
    key_hash TEXT NOT NULL,
    key_prefix TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'disabled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS api_keys_project_id_idx ON public.api_keys (project_id);
CREATE INDEX IF NOT EXISTS api_keys_key_hash_idx ON public.api_keys (key_hash);

-- Shared tenant roles (grants applied per proj_{ulid} schema on create).
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'anon') THEN
        CREATE ROLE anon NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'authenticated') THEN
        CREATE ROLE authenticated NOLOGIN;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'service') THEN
        CREATE ROLE service NOLOGIN BYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'project_operator') THEN
        CREATE ROLE project_operator NOLOGIN BYPASSRLS;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'project_operator_readonly') THEN
        CREATE ROLE project_operator_readonly NOLOGIN;
    END IF;
END
$$;

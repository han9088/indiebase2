pub const DASHBOARD_SESSION_PREFIX: &str = "dashboard_session:";
pub const APP_USER_SESSION_PREFIX: &str = "app_user_session:";

/// Default Dashboard session TTL (24h).
pub const DEFAULT_SESSION_TTL_SECS: u64 = 86_400;

/// Authenticator JWT lifetime for PostgREST (seconds).
pub const POSTGREST_AUTHENTICATOR_JWT_TTL_SECS: u64 = 60;

/// Internal-Context payload lifetime (seconds).
pub const INTERNAL_CONTEXT_TTL_SECS: u64 = 60;

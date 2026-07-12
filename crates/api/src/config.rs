use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Active environment (`development` default), Vite-style.
    pub env: String,
    pub http_addr: String,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: String,
    pub postgrest_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    Missing { key: &'static str },
    Empty { key: &'static str },
    Invalid { key: &'static str, message: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { key } => write!(f, "missing required environment variable: {key}"),
            Self::Empty { key } => write!(f, "environment variable {key} must not be empty"),
            Self::Invalid { key, message } => {
                write!(f, "invalid environment variable {key}: {message}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let indiebase_env = resolve_indiebase_env();
        load_dotenv_for_env(&indiebase_env);
        Self::from_env_vars()
    }

    pub(crate) fn from_env_vars() -> Result<Self, ConfigError> {
        Ok(Self {
            env: resolve_indiebase_env(),
            http_addr: env::var("INDIEBASE_HTTP_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            postgres_host: required_env("POSTGRES_HOST")?,
            postgres_port: optional_port("POSTGRES_PORT", 5432)?,
            postgres_user: required_env("POSTGRES_USER")?,
            postgres_password: required_env("POSTGRES_PASSWORD")?,
            postgres_db: required_env("POSTGRES_DB")?,
            redis_host: required_env("REDIS_HOST")?,
            redis_port: optional_port("REDIS_PORT", 6379)?,
            redis_password: required_env("REDIS_PASSWORD")?,
            postgrest_url: required_env("POSTGREST_URL")?,
        })
    }

    /// Postgres connection URI for sqlx / drivers (password percent-encoded).
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.postgres_user,
            percent_encode(&self.postgres_password),
            self.postgres_host,
            self.postgres_port,
            self.postgres_db
        )
    }

    /// Redis connection URI (password percent-encoded).
    pub fn redis_url(&self) -> String {
        format!(
            "redis://:{}@{}:{}",
            percent_encode(&self.redis_password),
            self.redis_host,
            self.redis_port
        )
    }
}

/// Active environment — Vite-style (`development` default). See INDIEBASE_ENV.
pub(crate) fn resolve_indiebase_env() -> String {
    match env::var("INDIEBASE_ENV") {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                "development".to_string()
            } else {
                trimmed.to_string()
            }
        }
        Err(_) => "development".to_string(),
    }
}

/// Load env files like Vite:
/// `.env` < `.env.local` < `.env.[env]` < `.env.[env].local`
/// Process env already set wins (not overwritten).
///
/// Spec: https://cn.vite.dev/guide/env-and-mode
pub(crate) fn load_dotenv_for_env(indiebase_env: &str) {
    let mut merged: HashMap<String, String> = HashMap::new();

    for path in [
        ".env".to_string(),
        ".env.local".to_string(),
        format!(".env.{indiebase_env}"),
        format!(".env.{indiebase_env}.local"),
    ] {
        merge_dotenv_file(&mut merged, &path);
    }

    for (key, value) in merged {
        if env::var_os(&key).is_none() {
            unsafe {
                env::set_var(key, value);
            }
        }
    }
}

fn merge_dotenv_file(into: &mut HashMap<String, String>, path: &str) {
    let path = Path::new(path);
    if !path.exists() {
        return;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let value = strip_dotenv_quotes(value.trim());
        into.insert(key.to_string(), value.to_string());
    }
}

fn strip_dotenv_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn required_env(key: &'static str) -> Result<String, ConfigError> {
    match env::var(key) {
        Ok(value) if value.trim().is_empty() => Err(ConfigError::Empty { key }),
        Ok(value) => Ok(value),
        Err(_) => Err(ConfigError::Missing { key }),
    }
}

fn optional_port(key: &'static str, default: u16) -> Result<u16, ConfigError> {
    match env::var(key) {
        Err(_) => Ok(default),
        Ok(value) if value.trim().is_empty() => Ok(default),
        Ok(value) => value.trim().parse().map_err(|_| ConfigError::Invalid {
            key,
            message: format!("expected u16, got {value}"),
        }),
    }
}

fn percent_encode(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    const CLEAR_KEYS: &[&str] = &[
        "POSTGRES_HOST",
        "POSTGRES_PORT",
        "POSTGRES_USER",
        "POSTGRES_PASSWORD",
        "POSTGRES_DB",
        "REDIS_HOST",
        "REDIS_PORT",
        "REDIS_PASSWORD",
        "POSTGREST_URL",
        "INDIEBASE_HTTP_ADDR",
        "INDIEBASE_ENV",
        "INDIEBASE_MODE",
        "ONLY_IN_BASE",
        "ONLY_IN_ENV",
        "SHARED_KEY",
        "DATABASE_URL",
        "REDIS_URL",
    ];

    fn with_isolated_env(f: impl FnOnce()) {
        let _guard = ENV_LOCK.lock().expect("env test lock poisoned");
        for key in CLEAR_KEYS {
            unsafe {
                env::remove_var(key);
            }
        }
        f();
    }

    fn set_valid_connection_env() {
        unsafe {
            env::set_var("POSTGRES_HOST", "postgres.indiebase2.orb.local");
            env::set_var("POSTGRES_USER", "postgres");
            env::set_var("POSTGRES_PASSWORD", "dev@indiebase.com");
            env::set_var("POSTGRES_DB", "indiebase-dev");
            env::set_var("REDIS_HOST", "localhost");
            env::set_var("REDIS_PASSWORD", "dev@indiebase.com");
            env::set_var("POSTGREST_URL", "http://localhost:13000");
        }
    }

    #[test]
    fn fails_when_postgres_host_missing() {
        with_isolated_env(|| {
            assert_eq!(
                Config::from_env_vars(),
                Err(ConfigError::Missing {
                    key: "POSTGRES_HOST"
                })
            );
        });
    }

    #[test]
    fn fails_when_postgres_password_empty() {
        with_isolated_env(|| {
            set_valid_connection_env();
            unsafe {
                env::set_var("POSTGRES_PASSWORD", "   ");
            }
            assert_eq!(
                Config::from_env_vars(),
                Err(ConfigError::Empty {
                    key: "POSTGRES_PASSWORD"
                })
            );
        });
    }

    #[test]
    fn loads_postgres_fields_and_builds_database_url() {
        with_isolated_env(|| {
            set_valid_connection_env();

            let config = Config::from_env_vars().expect("config should load");
            assert_eq!(config.env, "development");
            assert_eq!(config.http_addr, "0.0.0.0:8080");
            assert_eq!(config.postgres_host, "postgres.indiebase2.orb.local");
            assert_eq!(config.postgres_port, 5432);
            assert_eq!(config.postgres_db, "indiebase-dev");
            assert_eq!(
                config.database_url(),
                "postgres://postgres:dev%40indiebase.com@postgres.indiebase2.orb.local:5432/indiebase-dev"
            );
            assert_eq!(
                config.redis_url(),
                "redis://:dev%40indiebase.com@localhost:6379"
            );
        });
    }

    #[test]
    fn defaults_env_to_development() {
        with_isolated_env(|| {
            assert_eq!(resolve_indiebase_env(), "development");
        });
    }

    #[test]
    fn reads_explicit_env() {
        with_isolated_env(|| {
            unsafe {
                env::set_var("INDIEBASE_ENV", "production");
            }
            assert_eq!(resolve_indiebase_env(), "production");
        });
    }

    #[test]
    fn env_file_overrides_base_env_file() {
        with_isolated_env(|| {
            let dir = env::temp_dir().join(format!(
                "indiebase-env-test-{}-{}",
                std::process::id(),
                "vite-priority"
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("temp dir");

            fs::write(
                dir.join(".env"),
                "SHARED_KEY=from-base\nONLY_IN_BASE=base\nPOSTGRES_HOST=from-base\n",
            )
            .unwrap();
            fs::write(
                dir.join(".env.development"),
                "SHARED_KEY=from-env\nONLY_IN_ENV=env\nPOSTGRES_HOST=from-env\n",
            )
            .unwrap();

            let previous = env::current_dir().unwrap();
            env::set_current_dir(&dir).unwrap();
            load_dotenv_for_env("development");
            env::set_current_dir(previous).unwrap();
            let _ = fs::remove_dir_all(&dir);

            assert_eq!(env::var("SHARED_KEY").unwrap(), "from-env");
            assert_eq!(env::var("ONLY_IN_BASE").unwrap(), "base");
            assert_eq!(env::var("ONLY_IN_ENV").unwrap(), "env");
            assert_eq!(env::var("POSTGRES_HOST").unwrap(), "from-env");
        });
    }

    #[test]
    fn process_env_wins_over_dotenv_files() {
        with_isolated_env(|| {
            let dir = env::temp_dir().join(format!(
                "indiebase-env-test-{}-{}",
                std::process::id(),
                "process-wins"
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("temp dir");

            fs::write(dir.join(".env"), "POSTGRES_HOST=from-file\n").unwrap();
            unsafe {
                env::set_var("POSTGRES_HOST", "from-process");
            }

            let previous = env::current_dir().unwrap();
            env::set_current_dir(&dir).unwrap();
            load_dotenv_for_env("development");
            env::set_current_dir(previous).unwrap();
            let _ = fs::remove_dir_all(&dir);

            assert_eq!(env::var("POSTGRES_HOST").unwrap(), "from-process");
        });
    }
}

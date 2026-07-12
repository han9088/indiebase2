use std::fs;
use std::path::Path;

use sqlx::PgPool;

use crate::config::Config;
use crate::error::ApiError;

/// Append `schema` to the schemas file (comma-separated) and rewrite PostgREST conf
/// so `NOTIFY pgrst, 'reload config'` picks up the new schema list.
///
/// Chosen approach (Phase 1 spike): file-based PostgREST config on a compose volume
/// + `pg_notify` reload. See `docs/notes/postgrest-schema-reload.md`.
pub async fn register_schema_and_reload(
    pool: &PgPool,
    config: &Config,
    schema: &str,
) -> Result<(), ApiError> {
    if let Err(err) = update_schemas_file(&config.postgrest_schemas_file, schema) {
        tracing::warn!(
            error = %err,
            schema,
            "failed to update PostgREST schemas file; schema created but may need manual reload"
        );
    } else if let Err(err) = rewrite_postgrest_config(config) {
        tracing::warn!(
            error = %err,
            "failed to rewrite PostgREST config; schemas file updated — restart postgrest if reload fails"
        );
    }

    sqlx::query("SELECT pg_notify('pgrst', 'reload config')")
        .execute(pool)
        .await?;
    sqlx::query("SELECT pg_notify('pgrst', 'reload schema')")
        .execute(pool)
        .await?;

    Ok(())
}

fn update_schemas_file(path: &str, schema: &str) -> Result<(), String> {
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let current = if path.exists() {
        fs::read_to_string(path).map_err(|e| e.to_string())?
    } else {
        "public".to_string()
    };

    let mut schemas: Vec<String> = current
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    if !schemas.iter().any(|s| s == schema) {
        schemas.push(schema.to_string());
    }

    let joined = schemas.join(",");
    fs::write(path, format!("{joined}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn rewrite_postgrest_config(config: &Config) -> Result<(), String> {
    let schemas = fs::read_to_string(&config.postgrest_schemas_file)
        .unwrap_or_else(|_| "public\n".to_string());
    let schemas = schemas.trim();

    let path = Path::new(&config.postgrest_config_path);
    if !path.exists() {
        return Err(format!(
            "PostgREST conf missing at {}; start compose so entrypoint writes db-uri",
            path.display()
        ));
    }

    let existing = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if extract_conf_value(&existing, "db-uri").is_none() {
        return Err(
            "PostgREST conf has no db-uri; refusing to rewrite (restart postgrest via compose)"
                .into(),
        );
    }

    // Surgically replace db-schemas only — never drop db-uri / other keys.
    let mut replaced = false;
    let mut lines: Vec<String> = existing
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                return line.to_string();
            }
            let Some((k, _)) = trimmed.split_once('=') else {
                return line.to_string();
            };
            if k.trim() != "db-schemas" {
                return line.to_string();
            }
            replaced = true;
            format!("db-schemas = \"{schemas}\"")
        })
        .collect();

    if !replaced {
        lines.push(format!("db-schemas = \"{schemas}\""));
    }
    if !lines.last().is_some_and(|l| l.is_empty()) {
        lines.push(String::new());
    }

    fs::write(path, lines.join("\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_conf_value(conf: &str, key: &str) -> Option<String> {
    for line in conf.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        if k.trim() != key {
            continue;
        }
        let v = v.trim().trim_matches('"').to_string();
        return Some(v);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn update_schemas_file_appends_once() {
        let dir = env::temp_dir().join(format!(
            "indiebase-pgrst-schemas-{}-{}",
            std::process::id(),
            "t"
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("db-schemas");
        fs::write(&path, "public\n").unwrap();

        update_schemas_file(path.to_str().unwrap(), "proj_abc").unwrap();
        update_schemas_file(path.to_str().unwrap(), "proj_abc").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "public,proj_abc");
        let _ = fs::remove_dir_all(&dir);
    }
}

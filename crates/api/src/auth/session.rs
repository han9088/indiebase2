use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::constants::session::{DASHBOARD_SESSION_PREFIX, PROJECT_SESSION_PREFIX};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSessionData {
    pub user_id: String,
    pub exp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSessionData {
    pub user_id: String,
    pub project_id: String,
    pub project_role: String,
    pub exp: u64,
}

pub fn mint_opaque_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn now_epoch_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn dashboard_session_key(token: &str) -> String {
    format!("{DASHBOARD_SESSION_PREFIX}{token}")
}

pub fn project_session_key(token: &str) -> String {
    format!("{PROJECT_SESSION_PREFIX}{token}")
}

pub async fn store_dashboard_session(
    state: &AppState,
    token: &str,
    user_id: &str,
) -> Result<DashboardSessionData, ApiError> {
    let ttl = state.config.session_ttl_secs;
    let data = DashboardSessionData {
        user_id: user_id.to_string(),
        exp: now_epoch_secs().saturating_add(ttl),
    };
    let payload = serde_json::to_string(&data)?;
    let key = dashboard_session_key(token);
    let mut redis = state.redis.clone();
    redis.set_ex::<_, _, ()>(key, payload, ttl).await?;
    Ok(data)
}

pub async fn load_dashboard_session(
    state: &AppState,
    token: &str,
) -> Result<Option<DashboardSessionData>, ApiError> {
    let key = dashboard_session_key(token);
    let mut redis = state.redis.clone();
    let value: Option<String> = redis.get(key).await?;
    match value {
        Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
        None => Ok(None),
    }
}

pub async fn delete_dashboard_session(state: &AppState, token: &str) -> Result<(), ApiError> {
    let key = dashboard_session_key(token);
    let mut redis = state.redis.clone();
    let _: () = redis.del(key).await?;
    Ok(())
}

pub async fn store_project_session(
    state: &AppState,
    token: &str,
    user_id: &str,
    project_id: &str,
    project_role: &str,
) -> Result<ProjectSessionData, ApiError> {
    let ttl = state.config.session_ttl_secs;
    let data = ProjectSessionData {
        user_id: user_id.to_string(),
        project_id: project_id.to_string(),
        project_role: project_role.to_string(),
        exp: now_epoch_secs().saturating_add(ttl),
    };
    let payload = serde_json::to_string(&data)?;
    let key = project_session_key(token);
    let mut redis = state.redis.clone();
    redis.set_ex::<_, _, ()>(key, payload, ttl).await?;
    Ok(data)
}

pub async fn load_project_session(
    state: &AppState,
    token: &str,
) -> Result<Option<ProjectSessionData>, ApiError> {
    let key = project_session_key(token);
    let mut redis = state.redis.clone();
    let value: Option<String> = redis.get(key).await?;
    match value {
        Some(raw) => Ok(Some(serde_json::from_str(&raw)?)),
        None => Ok(None),
    }
}

pub async fn delete_project_session(state: &AppState, token: &str) -> Result<(), ApiError> {
    let key = project_session_key(token);
    let mut redis = state.redis.clone();
    let _: () = redis.del(key).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_key_prefixes() {
        assert_eq!(dashboard_session_key("abc"), "dashboard_session:abc");
        assert_eq!(project_session_key("xyz"), "project_session:xyz");
    }

    #[test]
    fn mint_token_is_hex_64() {
        let token = mint_opaque_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

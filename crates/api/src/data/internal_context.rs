//! Signed `X-Indiebase-Internal-Context` for PostgREST db-pre-request.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use crate::constants::session::INTERNAL_CONTEXT_TTL_SECS;
use crate::error::ApiError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize)]
pub struct InternalContextPayload {
    pub auth_mode: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_role: Option<String>,
    pub exp: u64,
}

fn now_epoch_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Build `base64url(payload).hex(hmac-sha256(payload_bytes))`.
pub fn sign_internal_context(
    secret: &str,
    auth_mode: &str,
    project_id: &str,
    user_id: Option<&str>,
    project_role: Option<&str>,
) -> Result<String, ApiError> {
    let payload = InternalContextPayload {
        auth_mode: auth_mode.to_string(),
        project_id: project_id.to_string(),
        user_id: user_id.map(str::to_string),
        project_role: project_role.map(str::to_string),
        exp: now_epoch_secs().saturating_add(INTERNAL_CONTEXT_TTL_SECS),
    };
    let json = serde_json::to_vec(&payload)?;
    let payload_b64 = URL_SAFE_NO_PAD.encode(&json);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| ApiError::Internal(format!("hmac key error: {err}")))?;
    mac.update(&json);
    let sig = hex::encode(mac.finalize().into_bytes());

    Ok(format!("{payload_b64}.{sig}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_roundtrip_shape() {
        let token = sign_internal_context(
            "secret",
            "service",
            "01jcqz4sxf7k2m8n3p5r6t9vwx",
            None,
            None,
        )
        .unwrap();
        let (payload, sig) = token.split_once('.').unwrap();
        assert!(!payload.is_empty());
        assert_eq!(sig.len(), 64);
    }
}

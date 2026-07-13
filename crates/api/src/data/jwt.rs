//! Short-lived HS256 JWT for PostgREST authenticator role.

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;

use crate::constants::data_api::DB_ROLE_AUTHENTICATOR;
use crate::constants::session::POSTGREST_AUTHENTICATOR_JWT_TTL_SECS;
use crate::error::ApiError;

type HmacSha256 = Hmac<Sha256>;

#[derive(Serialize)]
struct JwtHeader {
    alg: &'static str,
    typ: &'static str,
}

#[derive(Serialize)]
struct JwtClaims {
    role: &'static str,
    exp: u64,
}

fn now_epoch_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn mint_authenticator_jwt(secret: &str) -> Result<String, ApiError> {
    let header = JwtHeader {
        alg: "HS256",
        typ: "JWT",
    };
    let claims = JwtClaims {
        role: DB_ROLE_AUTHENTICATOR,
        exp: now_epoch_secs().saturating_add(POSTGREST_AUTHENTICATOR_JWT_TTL_SECS),
    };

    let header_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
    let claims_b64 = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?);
    let signing_input = format!("{header_b64}.{claims_b64}");

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|err| ApiError::Internal(format!("jwt hmac key error: {err}")))?;
    mac.update(signing_input.as_bytes());
    let sig = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    Ok(format!("{signing_input}.{sig}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_has_three_parts() {
        let jwt = mint_authenticator_jwt("test-secret").unwrap();
        assert_eq!(jwt.split('.').count(), 3);
    }
}

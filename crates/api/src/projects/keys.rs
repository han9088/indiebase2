use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::constants::keys::{PUBLISHABLE_KEY_PREFIX, SECRET_KEY_PREFIX};

pub fn mint_publishable_key() -> String {
    mint_key(PUBLISHABLE_KEY_PREFIX)
}

pub fn mint_secret_key() -> String {
    mint_key(SECRET_KEY_PREFIX)
}

fn mint_key(prefix: &str) -> String {
    let mut bytes = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut bytes);
    format!("{prefix}{}", hex::encode(bytes))
}

pub fn hash_api_key(plaintext: &str) -> String {
    let digest = Sha256::digest(plaintext.as_bytes());
    hex::encode(digest)
}

pub fn key_prefix_for_display(plaintext: &str) -> String {
    let end = plaintext.len().min(12);
    format!("{}…", &plaintext[..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_and_hash_stable() {
        let pub_key = mint_publishable_key();
        assert!(pub_key.starts_with(PUBLISHABLE_KEY_PREFIX));
        let sec = mint_secret_key();
        assert!(sec.starts_with(SECRET_KEY_PREFIX));
        assert_eq!(hash_api_key("abc"), hash_api_key("abc"));
        assert_ne!(hash_api_key("abc"), hash_api_key("abd"));
    }
}

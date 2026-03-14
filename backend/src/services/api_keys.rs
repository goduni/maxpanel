use rand::TryRngCore;
use crate::services::crypto;
use crate::utils::hex_encode;

pub const MAX_KEYS_PER_BOT: i64 = 10;
const KEY_PREFIX_TAG: &str = "ak_";
const KEY_RANDOM_BYTES: usize = 32;
const PREFIX_HEX_LEN: usize = 8;

/// Generate a new API key. Returns (full_key, prefix).
/// Full key format: "ak_" + 64 hex chars (32 random bytes).
/// Prefix format: "ak_" + first 8 hex chars.
pub fn generate_api_key() -> (String, String) {
    let mut bytes = [0u8; KEY_RANDOM_BYTES];
    rand::rngs::OsRng.try_fill_bytes(&mut bytes).expect("OsRng failed");
    let hex = hex_encode(&bytes);
    let key = format!("{}{}", KEY_PREFIX_TAG, hex);
    let prefix = format!("{}{}", KEY_PREFIX_TAG, &hex[..PREFIX_HEX_LEN]);
    (key, prefix)
}

/// Hash an API key using HMAC-SHA256 (same pattern as refresh tokens).
pub fn hash_api_key(secret: &str, key: &str) -> String {
    crypto::hash_token(secret, key)
}

/// Verify an API key against a stored hash using constant-time comparison.
pub fn verify_api_key(secret: &str, key: &str, stored_hash: &str) -> bool {
    let computed = hash_api_key(secret, key);
    constant_time_eq(computed.as_bytes(), stored_hash.as_bytes())
}

/// Constant-time comparison. The length check is an acceptable side-channel here:
/// both inputs are always HMAC-SHA256 hex digests (64 chars). A length mismatch
/// means the stored hash is corrupted, not that the attacker learns useful info.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key_format() {
        let (key, prefix) = generate_api_key();
        assert!(key.starts_with("ak_"));
        assert_eq!(key.len(), 3 + 64); // ak_ + 64 hex chars
        assert!(prefix.starts_with("ak_"));
        assert_eq!(prefix.len(), 3 + PREFIX_HEX_LEN); // ak_ + 8 hex chars
        assert!(key.starts_with(&prefix));
    }

    #[test]
    fn test_generate_unique() {
        let (k1, _) = generate_api_key();
        let (k2, _) = generate_api_key();
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_hash_and_verify() {
        let secret = "test-secret-must-be-at-least-32-bytes-long";
        let (key, _) = generate_api_key();
        let hash = hash_api_key(secret, &key);
        assert!(verify_api_key(secret, &key, &hash));
    }

    #[test]
    fn test_verify_wrong_key() {
        let secret = "test-secret-must-be-at-least-32-bytes-long";
        let (key, _) = generate_api_key();
        let hash = hash_api_key(secret, &key);
        assert!(!verify_api_key(secret, "ak_wrong_key", &hash));
    }
}

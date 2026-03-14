use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, AeadCore,
};
use hkdf::Hkdf;
use sha2::Sha256;
use uuid::Uuid;

pub fn derive_bot_key(master_key: &[u8; 32], bot_id: Uuid, key_version: i32) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(Some(b"maxpanel-bot-token-v1"), master_key);
    let mut okm = [0u8; 32];
    // info = bot_id bytes (16 bytes) + key_version (4 bytes, big-endian)
    let mut info = Vec::with_capacity(20);
    info.extend_from_slice(bot_id.as_bytes());
    info.extend_from_slice(&key_version.to_be_bytes());
    hkdf.expand(&info, &mut okm)
        .expect("32 bytes is a valid HKDF output length");
    okm
}

pub fn encrypt_token(master_key: &[u8; 32], bot_id: Uuid, key_version: i32, plaintext: &str) -> (Vec<u8>, Vec<u8>) {
    let key = derive_bot_key(master_key, bot_id, key_version);
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key length");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .expect("encryption should not fail");
    (ciphertext, nonce.to_vec())
}

pub fn decrypt_token(master_key: &[u8; 32], bot_id: Uuid, key_version: i32, ciphertext: &[u8], nonce: &[u8]) -> Result<String, anyhow::Error> {
    let key = derive_bot_key(master_key, bot_id, key_version);
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    if nonce.len() != 12 {
        return Err(anyhow::anyhow!("Invalid nonce length: expected 12, got {}", nonce.len()));
    }
    let nonce = aes_gcm::Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
    Ok(String::from_utf8(plaintext)?)
}

pub fn hash_token(secret: &str, token: &str) -> String {
    use hmac::{Hmac, Mac, digest::KeyInit};
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(token.as_bytes());
    let result = mac.finalize();
    crate::utils::hex_encode(result.into_bytes().as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_bot_key_is_deterministic() {
        let master = [0xABu8; 32];
        let bot_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let k1 = derive_bot_key(&master, bot_id, 1);
        let k2 = derive_bot_key(&master, bot_id, 1);
        assert_eq!(k1, k2);
    }

    #[test]
    fn derive_bot_key_differs_by_version() {
        let master = [0xABu8; 32];
        let bot_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let k1 = derive_bot_key(&master, bot_id, 1);
        let k2 = derive_bot_key(&master, bot_id, 2);
        assert_ne!(k1, k2);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let master = [0xCDu8; 32];
        let bot_id = Uuid::new_v4();
        let plaintext = "secret-token-value";

        let (ciphertext, nonce) = encrypt_token(&master, bot_id, 1, plaintext);
        let decrypted = decrypt_token(&master, bot_id, 1, &ciphertext, &nonce).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn decrypt_with_wrong_version_fails() {
        let master = [0xCDu8; 32];
        let bot_id = Uuid::new_v4();
        let (ciphertext, nonce) = encrypt_token(&master, bot_id, 1, "test");
        let result = decrypt_token(&master, bot_id, 2, &ciphertext, &nonce);
        assert!(result.is_err());
    }

    #[test]
    fn decrypt_with_invalid_nonce_length_fails() {
        let master = [0xCDu8; 32];
        let bot_id = Uuid::new_v4();
        let (ciphertext, _) = encrypt_token(&master, bot_id, 1, "test");
        let result = decrypt_token(&master, bot_id, 1, &ciphertext, &[0u8; 8]);
        assert!(result.is_err());
    }

    #[test]
    fn hash_token_deterministic() {
        let h1 = hash_token("secret", "token");
        let h2 = hash_token("secret", "token");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_token_differs_by_input() {
        let h1 = hash_token("secret", "token1");
        let h2 = hash_token("secret", "token2");
        assert_ne!(h1, h2);
    }
}

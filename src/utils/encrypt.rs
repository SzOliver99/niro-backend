use chacha20poly1305::{
    AeadCore, ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
pub type HmacSecret = Vec<u8>;

pub fn hash_value(secret: &[u8], value: &str) -> Vec<u8> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(secret).unwrap();
    mac.update(value.as_bytes());
    mac.finalize().into_bytes().to_vec()
}

pub fn encrypt_value(key: &Key, plaintext: &str) -> (Vec<u8>, Vec<u8>) {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng); // 96-bit
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).unwrap();
    (ciphertext, nonce.to_vec())
}

pub fn decrypt_value(key: &Key, ciphertext: &[u8], nonce: &[u8]) -> Option<String> {
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(nonce);
    match cipher.decrypt(nonce, ciphertext) {
        Ok(plaintext) => String::from_utf8(plaintext).ok(),
        Err(_) => None, // decryption failed
    }
}

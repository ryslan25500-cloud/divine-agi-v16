//! Cryptography Module V15 for Divine AGI
//!
//! SHA-256/SHA-3, AES-GCM, secp256k1

use sha2::{Sha256, Digest};
use sha3::Sha3_256;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use hkdf::Hkdf;
use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivineSignature {
    pub algorithm: String,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationKeys {
    pub rot0_key: Vec<u8>,
    pub rot90_key: Vec<u8>,
    pub rot180_key: Vec<u8>,
    pub rot270_key: Vec<u8>,
}

impl RotationKeys {
    pub fn generate() -> Self {
        Self {
            rot0_key: rand::thread_rng().gen::<[u8; 32]>().to_vec(),
            rot90_key: rand::thread_rng().gen::<[u8; 32]>().to_vec(),
            rot180_key: rand::thread_rng().gen::<[u8; 32]>().to_vec(),
            rot270_key: rand::thread_rng().gen::<[u8; 32]>().to_vec(),
        }
    }

    pub fn key_for_angle(&self, angle: u16) -> &[u8] {
        match angle {
            0 => &self.rot0_key,
            90 => &self.rot90_key,
            180 => &self.rot180_key,
            270 => &self.rot270_key,
            _ => &self.rot0_key,
        }
    }
}

pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn hash_sha3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn derive_key(master: &[u8], info: &[u8]) -> [u8; 32] {
    let hkdf = Hkdf::<Sha256>::new(None, master);
    let mut key = [0u8; 32];
    hkdf.expand(info, &mut key).expect("HKDF expand failed");
    key
}

pub fn encrypt_aes_gcm(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {:?}", e))?;
    
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt_aes_gcm(key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < 12 {
        return Err("Ciphertext too short".into());
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    
    cipher.decrypt(nonce, &ciphertext[12..])
        .map_err(|e| format!("Decryption failed: {:?}", e))
}

pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    use secp256k1::{Secp256k1, SecretKey, PublicKey};
    
    let secp = Secp256k1::new();
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    (secret_key.secret_bytes().to_vec(), public_key.serialize().to_vec())
}

pub fn sign_message(secret_key: &[u8], message: &[u8]) -> Result<Vec<u8>, String> {
    use secp256k1::{Secp256k1, SecretKey, Message};
    
    let secp = Secp256k1::new();
    let sk = SecretKey::from_slice(secret_key)
        .map_err(|e| format!("Invalid secret key: {:?}", e))?;
    
    let hash = hash_sha256(message);
    let msg = Message::from_digest_slice(&hash)
        .map_err(|e| format!("Invalid message: {:?}", e))?;
    
    let sig = secp.sign_ecdsa(&msg, &sk);
    Ok(sig.serialize_compact().to_vec())
}

pub fn verify_signature(public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
    use secp256k1::{Secp256k1, PublicKey, Message, ecdsa::Signature};
    
    let secp = Secp256k1::new();
    
    let pk = match PublicKey::from_slice(public_key) {
        Ok(pk) => pk,
        Err(_) => return false,
    };
    
    let sig = match Signature::from_compact(signature) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    
    let hash = hash_sha256(message);
    let msg = match Message::from_digest_slice(&hash) {
        Ok(msg) => msg,
        Err(_) => return false,
    };
    
    secp.verify_ecdsa(&msg, &sig, &pk).is_ok()
}

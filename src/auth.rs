//! Authentication Module V15.1 for Divine Wallet
//!
//! Features:
//! - Password hashing (SHA-256 + salt)
//! - JWT-like session tokens
//! - Wallet registration/login

use sha2::{Sha256, Digest};
use rand::Rng;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use chrono::Utc;

const TOKEN_VALIDITY_HOURS: i64 = 24 * 7; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub salt: String,
    pub wallet_address: String,
    pub rsm_balance: f64,
    pub founder_pool_rsm: f64,
    pub is_founder: bool,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
    pub token: String,
    pub wallet_address: String,
    pub username: String,
    pub created_at: i64,
    pub expires_at: i64,
}

impl SessionToken {
    pub fn is_valid(&self) -> bool {
        Utc::now().timestamp() < self.expires_at
    }
}

pub struct AuthManager {
    sessions: HashMap<String, SessionToken>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Generate salt for password hashing
    pub fn generate_salt() -> String {
        let salt: [u8; 16] = rand::thread_rng().gen();
        hex::encode(salt)
    }

    /// Hash password with salt
    pub fn hash_password(password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt.as_bytes());
        hasher.update(b"DIVINE_AGI_V15_SECRET");
        hex::encode(hasher.finalize())
    }

    /// Verify password
    pub fn verify_password(password: &str, salt: &str, hash: &str) -> bool {
        Self::hash_password(password, salt) == hash
    }

    /// Generate wallet address from username
    pub fn generate_wallet_address(username: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(username.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        hasher.update(rand::random::<[u8; 8]>());
        format!("rsm_{}", &hex::encode(hasher.finalize())[..32])
    }

    /// Generate session token
    pub fn generate_token(&mut self, wallet_address: &str, username: &str) -> SessionToken {
        let mut hasher = Sha256::new();
        hasher.update(wallet_address.as_bytes());
        hasher.update(Utc::now().timestamp_nanos_opt().unwrap_or(0).to_le_bytes());
        hasher.update(rand::random::<[u8; 32]>());
        
        let token = hex::encode(hasher.finalize());
        let now = Utc::now().timestamp();
        
        let session = SessionToken {
            token: token.clone(),
            wallet_address: wallet_address.to_string(),
            username: username.to_string(),
            created_at: now,
            expires_at: now + (TOKEN_VALIDITY_HOURS * 3600),
        };

        self.sessions.insert(token.clone(), session.clone());
        session
    }

    /// Validate token and get wallet address
    pub fn validate_token(&self, token: &str) -> Option<&SessionToken> {
        self.sessions.get(token).filter(|s| s.is_valid())
    }

    /// Logout (invalidate token)
    pub fn logout(&mut self, token: &str) -> bool {
        self.sessions.remove(token).is_some()
    }

    /// Clean expired sessions
    pub fn cleanup_expired(&mut self) {
        let now = Utc::now().timestamp();
        self.sessions.retain(|_, s| s.expires_at > now);
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub is_founder: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub wallet_address: Option<String>,
    pub username: Option<String>,
    pub rsm_balance: Option<f64>,
    pub founder_pool_rsm: Option<f64>,
    pub is_founder: Option<bool>,
    pub expires_at: Option<i64>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub username: String,
    pub wallet_address: String,
    pub rsm_balance: f64,
    pub founder_pool_rsm: f64,
    pub is_founder: bool,
    pub created_at: i64,
    pub last_login: Option<i64>,
    pub total_value_usd: f64,
}

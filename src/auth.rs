use jsonwebtoken::{decode, encode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i32,
    pub username: String,
    pub role: String,
    pub exp: usize,
}

pub fn generate_token(user_id: i32, username: &str, role: &str) -> Result<String, String> {
    let expiration = SystemTime::now()
        .checked_add(Duration::from_hours(24))
        .ok_or("Failed to set expiration time")?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as usize;

    let claims = Claims {
        user_id,
        username: username.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let secret = "your-secret-key"; // 实际应用中应从环境变量读取
    let encoding_key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), &claims, &encoding_key)
        .map_err(|e| e.to_string())
}

pub fn verify_token(token: &str) -> Result<Claims, String> {
    let secret = "your-secret-key"; // 实际应用中应从环境变量读取
    let decoding_key = DecodingKey::from_secret(secret.as_bytes());
    decode::<Claims>(token, &decoding_key, &Validation::default())
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
}

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST)
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash)
        .map_err(|e| e.to_string())
}

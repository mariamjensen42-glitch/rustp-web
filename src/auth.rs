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
    // 验证密码强度
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }
    
    let mut has_uppercase = false;
    let mut has_lowercase = false;
    let mut has_number = false;
    let mut has_special = false;
    
    for c in password.chars() {
        if c.is_uppercase() {
            has_uppercase = true;
        } else if c.is_lowercase() {
            has_lowercase = true;
        } else if c.is_numeric() {
            has_number = true;
        } else if "@$!%*?&" .contains(c) {
            has_special = true;
        }
    }
    
    if !has_uppercase || !has_lowercase || !has_number || !has_special {
        return Err("Password must contain at least one uppercase letter, one lowercase letter, one number, and one special character".to_string());
    }
    
    hash(password, DEFAULT_COST)
        .map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash)
        .map_err(|e| e.to_string())
}

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::models::JwtClaims;

const TOKEN_EXPIRY_SECS: u64 = 7 * 24 * 3600;

// ══════════════════════════════════════════════════════════════════
//  PASSWORD HASHING  (Argon2id)
// ══════════════════════════════════════════════════════════════════

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!("Gagal hash password: {}", e))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow!("Hash tidak valid: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ══════════════════════════════════════════════════════════════════
//  JWT  (HS256)
// ══════════════════════════════════════════════════════════════════

pub fn sign_jwt(
    user_id:    i64,
    email:      &str,
    role:       &str,
    jwt_secret: &str,
) -> Result<(String, u64)> {
    let now = Utc::now().timestamp() as usize;
    let exp = now + TOKEN_EXPIRY_SECS as usize;

    let claims = JwtClaims {
        sub:   user_id.to_string(),
        email: email.to_string(),
        role:  role.to_string(),
        exp,
        iat:   now,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok((token, TOKEN_EXPIRY_SECS))
}

pub fn verify_jwt(token: &str, jwt_secret: &str) -> Result<JwtClaims> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )?;

    Ok(data.claims)
}

pub fn extract_bearer(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ").map(str::trim)
}

// ══════════════════════════════════════════════════════════════════
//  API KEY GENERATION
//  Pakai OsRng dari argon2::password_hash::rand_core (sudah diimport)
//  — tidak perlu rand::thread_rng() yang berubah API di rand 0.9
// ══════════════════════════════════════════════════════════════════

pub fn generate_api_key() -> Result<(String, String, String)> {
    use argon2::password_hash::rand_core::RngCore;
    let mut random_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut random_bytes);
    let hex: String = random_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let full_key = format!("rag_{}", hex);
    let prefix   = full_key[..12].to_string(); // "rag_" + 8 chars
    let key_hash = hash_password(&full_key)?;
    Ok((full_key, prefix, key_hash))
}
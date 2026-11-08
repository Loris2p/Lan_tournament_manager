use std::env;
use crate::auth::claims::{Claims, UserRole};
use crate::errors::Errors;
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::bson::oid::ObjectId;
pub const DUMMY_HASH: &str = "$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj4085449q1K"; //  hash factice pré-calculé (coût 12 identique aux mots de passes)
pub fn hash_password(password: &str) -> Result<String, Errors> {
    if password.chars().count() < 8 {
        return Err(Errors::Error400BadRequest(
            "Le mot de passe doit contenir au moins 8 caractères.".to_string(),
        ));
    }
    if password.len() > 72 {
        return Err(Errors::Error400BadRequest(
            "Le mot de passe ne peut pas dépasser 72 octets (limite Bcrypt).".to_string(),
        ));
    }
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Errors> {
    Ok(bcrypt::verify(password, hash)?)
}
pub fn create_jwt(
    sub: ObjectId,
    username: String,
    role: UserRole,
    token_version: u32,
    exp: usize,
    secret: &str,
) -> Result<String, Errors> {
    let claims = Claims {
        sub: sub.to_hex(),
        username,
        role,
        exp,
        token_version,
    };
    Ok(encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?)
}
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, Errors> {
    let res = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?
    .claims;
    Ok(res)
}
pub fn build_auth_cookie(token: String) -> Cookie<'static> {
    let is_secure =  env::var("APP_ENV").map(|v| v=="production").unwrap_or(false)||env::var("CC_DEPLOYMENT_ID").is_ok();
    Cookie::build(("lan_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .secure(is_secure)
        .build()
}

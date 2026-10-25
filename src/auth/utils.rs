use crate::auth::claims::{Claims, UserRole};
use crate::errors::Errors;
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use mongodb::bson::oid::ObjectId;
pub fn hash_password(password: &str) -> Result<String, Errors> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}
pub fn verify_password(password: &str, hash: &str) -> Result<bool, Errors> {
    Ok(bcrypt::verify(password, hash)?)
}
pub fn create_jwt(
    sub: ObjectId,
    username: String,
    role: UserRole,
    exp: usize,
    secret: &str,
) -> Result<String, Errors> {
    let claims = Claims {
        sub: sub.to_hex(),
        username,
        role,
        exp,
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
    Cookie::build(("lan_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .build()
}

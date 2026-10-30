use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum UserRole {
    Admin,
    Captain { team_id: String },
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, //ObjectId en hexadecimal de l'admin ou de l'equipe
    pub username: String,
    pub role: UserRole,
    pub exp: usize, // timestamp de l'expiration
    #[serde(default)]
    pub token_version: u32,
}

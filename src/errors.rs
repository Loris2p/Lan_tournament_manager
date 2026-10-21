use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub enum Errors {
    Error400BadRequest(String),
    Error401Unauthorized(String),
    Error403Forbidden(String),
    Error404NotFound(String),
    Error409Conflict(String),
    Error500InternalError(String),
}
impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Errors::Error400BadRequest(s) => (StatusCode::BAD_REQUEST, s),
            Errors::Error401Unauthorized(s) => (StatusCode::UNAUTHORIZED, s),
            Errors::Error403Forbidden(s) => (StatusCode::FORBIDDEN, s),
            Errors::Error404NotFound(s) => (StatusCode::NOT_FOUND, s),
            Errors::Error409Conflict(s) => (StatusCode::CONFLICT, s),
            Errors::Error500InternalError(s) => (StatusCode::INTERNAL_SERVER_ERROR, s),
        };
        (status, Json(json!({"error":message}))).into_response()
    }
}
// Erreurs liées à la connexion discord
impl From<reqwest::Error> for Errors {
    fn from(err: reqwest::Error) -> Self {
        Errors::Error500InternalError(err.to_string())
    }
}

//Erreurs des cookies JWT
impl From<jsonwebtoken::errors::Error> for Errors {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Errors::Error401Unauthorized(err.to_string())
    }
}

//Erreurs de la DB mango
impl From<mongodb::error::Error> for Errors {
    fn from(err: mongodb::error::Error) -> Self {
        if err.to_string().contains("E11000") {
            return Errors::Error409Conflict(err.to_string());
        }
        Errors::Error500InternalError(err.to_string())
    }
}

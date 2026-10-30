use std::sync::Arc;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use crate::auth::{verify_jwt, UserRole};
use crate::models::AppState;

pub async fn serve_admin_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let is_admin = jar
        .get("lan_session")
        .and_then(|c| verify_jwt(c.value(), &state.jwt_secret).ok())
        .map(|claims| matches!(claims.role, UserRole::Admin))
        .unwrap_or(false);

    if is_admin {
        Html(include_str!("../../templates/admin.html")).into_response()
    } else {
        Redirect::to("/login.html").into_response()
    }
}

pub async fn serve_captain_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let is_captain = jar
        .get("lan_session")
        .and_then(|c| verify_jwt(c.value(), &state.jwt_secret).ok())
        .map(|claims| matches!(claims.role, UserRole::Captain { .. }))
        .unwrap_or(false);

    if is_captain {
        Html(include_str!("../../templates/captain.html")).into_response()
    } else {
        Redirect::to("/login.html").into_response()
    }
}

use std::sync::Arc;
use axum::extract::State;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use mongodb::bson::oid::ObjectId;
use crate::auth::{verify_jwt, UserRole};
use crate::models::AppState;

pub async fn serve_admin_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let is_admin = match jar.get("lan_session") {
        Some(c) => match verify_jwt(c.value(), &state.jwt_secret) {
            Ok(claims) if matches!(claims.role, UserRole::Admin) => {
                match ObjectId::parse_str(&claims.sub) {
                    Ok(oid) => crate::db::verify_admin_session(&state.db, oid, claims.token_version)
                        .await
                        .unwrap_or(false),
                    Err(_) => false,
                }
            }
            _ => false,
        },
        None => false,
    };

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
    let is_captain = match jar.get("lan_session") {
        Some(c) => match verify_jwt(c.value(), &state.jwt_secret) {
            Ok(claims) if matches!(claims.role, UserRole::Captain { .. }) => {
                match ObjectId::parse_str(&claims.sub) {
                    Ok(oid) => crate::db::verify_team_session(&state.db, oid, claims.token_version)
                        .await
                        .unwrap_or(false),
                    Err(_) => false,
                }
            }
            _ => false,
        },
        None => false,
    };

    if is_captain {
        Html(include_str!("../../templates/captain.html")).into_response()
    } else {
        Redirect::to("/login.html").into_response()
    }
}

use std::net::IpAddr;
use crate::auth::{
    build_auth_cookie, create_jwt, hash_password, verify_jwt, verify_password,
    AuthenticatedUser, Claims, UserRole,
};
use crate::db::{
    create_initial_admin, find_admin_by_username, find_team_by_captain_username,
    increment_admin_token_version, increment_team_token_version, is_setup_required,
};
use crate::errors::Errors;
use crate::models::{AppState, CreateAdminRequest, LoginRequest};
use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn setup_status(State(state): State<Arc<AppState>>) -> Result<Json<Value>, Errors> {
    let need_setup = is_setup_required(&state.db).await?;
    Ok(Json(json!({ "needs_setup": need_setup })))
}

pub async fn setup_first_admin(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(req): Json<CreateAdminRequest>,
) -> Result<(CookieJar, Json<Value>), Errors> {
    let db = &state.db;
    let username = &req.username;
    if !is_setup_required(db).await? {
        return Err(Errors::Error403Forbidden(
            "L'initialisation a déjà été effectuée".to_string(),
        ));
    }

    if find_team_by_captain_username(db, username).await.is_ok() {
        return Err(Errors::Error409Conflict(
            "Un capitaine a déjà ce nom".to_string(),
        ));
    }

    let h_pswd = hash_password(&req.password)?;
    let admin = create_initial_admin(db, username, &h_pswd).await?;
    let admin_id = admin
        .id
        .ok_or_else(|| Errors::Error500InternalError("ID manquant".to_string()))?;

    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;
    let token = create_jwt(
        admin_id,
        admin.username,
        UserRole::Admin,
        admin.token_version,
        exp,
        &state.jwt_secret,
    )?;
    let jar = jar.add(build_auth_cookie(token));

    Ok((jar, Json(json!({ "redirect": "/admin.html" }))))
}

pub(crate) fn extract_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').last())
        .map(|s| s.trim().to_string())
        .filter(|s| s.parse::<IpAddr>().is_ok())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> Result<(CookieJar, Json<Value>), Errors> {
    let client_ip = extract_client_ip(&headers);
    let username = req.username.trim();
    let key_rate = format!("{}:{}", client_ip, username);
    if !state.rate_limiter.check(&key_rate) {
        return Err(Errors::Error429TooManyRequests(
            "Trop de tentatives de connexion. Veuillez patienter une minute.".to_string(),
        ));
    }

    let db = &state.db;
    let username = req.username.trim();
    let password = &req.password;
    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;

    let (target_hash, auth_info) = if let Ok(admin) = find_admin_by_username(db, username).await {
        let info = admin
            .id
            .map(|id| (id, admin.username, UserRole::Admin, admin.token_version));
        (Some(admin.password_hash), info)
    } else if let Ok(team) = find_team_by_captain_username(db, username).await {
        let info = team.id.map(|id| {
            (
                id,
                team.captain_username,
                UserRole::Captain {
                    team_id: id.to_hex(),
                },
                team.token_version,
            )
        });
        (Some(team.captain_password_hash), info)
    } else {
        (None, None)
    };

    let hash_to_verify = target_hash.as_deref().unwrap_or(crate::auth::DUMMY_HASH);
    let is_valid = verify_password(password, hash_to_verify).unwrap_or(false);

    if is_valid {
        if let Some((user_id, user_name, role, token_version)) = auth_info {
            let (role_str, redirect) = match role {
                UserRole::Admin => ("ADMIN", "/admin.html"),
                UserRole::Captain { .. } => ("CAPTAIN", "/captain.html"),
            };
            let token = create_jwt(
                user_id,
                user_name,
                role,
                token_version,
                exp,
                &state.jwt_secret,
            )?;
            let jar = jar.add(build_auth_cookie(token));
            return Ok((
                jar,
                Json(json!({ "role": role_str, "redirect": redirect })),
            ));
        }
    }

    Err(Errors::Error401Unauthorized(
        "Identifiant ou mot de passe incorrect".to_string(),
    ))
}

pub async fn get_me(user: AuthenticatedUser) -> Json<Claims> {
    Json(user.claims)
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> (CookieJar, Json<Value>) {
    if let Some(cookie) = jar.get("lan_session") {
        if let Ok(claims) = verify_jwt(cookie.value(), &state.jwt_secret) {
            if let Ok(oid) = ObjectId::parse_str(&claims.sub) {
                match claims.role {
                    UserRole::Admin => {
                        let _ = increment_admin_token_version(&state.db, oid).await;
                    }
                    UserRole::Captain { .. } => {
                        let _ = increment_team_token_version(&state.db, oid).await;
                    }
                }
            }
        }
    }
    let cookie = Cookie::build("lan_session").path("/").build();
    let new_jar = jar.remove(cookie);
    (
        new_jar,
        Json(json!({ "message": "Déconnecté avec succès" })),
    )
}

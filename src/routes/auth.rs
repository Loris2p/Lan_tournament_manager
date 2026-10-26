use crate::auth::{
    AuthenticatedUser, Claims, UserRole, build_auth_cookie, create_jwt, hash_password,
    verify_password,
};
use crate::db::{
    count_admins, create_admin, find_admin_by_username, find_team_by_captain_username,
};
use crate::errors::Errors;
use crate::models::{AppState, CreateAdminRequest, LoginRequest};
use axum::Json;
use axum::extract::State;
use axum_extra::extract::CookieJar;
use axum_extra::extract::cookie::Cookie;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn setup_status(State(state): State<Arc<AppState>>) -> Result<Json<Value>, Errors> {
    let nb_admins = count_admins(&state.db).await?;
    let need_setup = nb_admins == 0;
    Ok(Json(json!({ "needs_setup": need_setup })))
}

pub async fn setup_first_admin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateAdminRequest>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>), Errors> {
    let db = &state.db;
    let username = &req.username;
    let nb_admins = count_admins(db).await?;
    if nb_admins != 0 {
        return Err(Errors::Error403Forbidden(
            "Un administrateur existe déjà".to_string(),
        ));
    }

    if find_team_by_captain_username(db, username).await.is_ok() {
        return Err(Errors::Error409Conflict(
            "Un capitaine a déjà ce nom".to_string(),
        ));
    }

    let h_pswd = hash_password(&req.password)?;
    let admin = create_admin(db, username, &h_pswd).await?;
    let admin_id = admin
        .id
        .ok_or_else(|| Errors::Error500InternalError("ID manquant".to_string()))?;

    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;
    let token = create_jwt(
        admin_id,
        admin.username,
        UserRole::Admin,
        exp,
        &state.jwt_secret,
    )?;
    let jar = jar.add(build_auth_cookie(token));

    Ok((jar, Json(json!({ "redirect": "/admin.html" }))))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<Value>), Errors> {
    let db = &state.db;
    let username = &req.username;
    let password = &req.password;
    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;
    match find_admin_by_username(db, username).await {
        Ok(admin) => {
            if verify_password(password, &admin.password_hash)? {
                let Some(admin_id) = admin.id else {
                    return Err(Errors::Error500InternalError("ID manquant".to_string()));
                };
                let token = create_jwt(
                    admin_id,
                    admin.username,
                    UserRole::Admin,
                    exp,
                    &state.jwt_secret,
                )?;
                let jar = jar.add(build_auth_cookie(token));
                return Ok((
                    jar,
                    Json(json!({ "role":"ADMIN", "redirect": "/admin.html" })),
                ));
            }
        }
        _ => {
            if let Ok(team) = find_team_by_captain_username(db, username).await {
                let pass_hash = team.captain_password_hash;
                if verify_password(password, &pass_hash)? {
                    let Some(team_id) = team.id else {
                        return Err(Errors::Error500InternalError("ID manquant".to_string()));
                    };
                    let token = create_jwt(
                        team_id,
                        team.captain_username,
                        UserRole::Captain {
                            team_id: team_id.to_hex(),
                        },
                        exp,
                        &state.jwt_secret,
                    )?;
                    let jar = jar.add(build_auth_cookie(token));
                    return Ok((
                        jar,
                        Json(json!({ "role":"CAPTAIN", "redirect": "/captain.html" })),
                    ));
                }
            }
        }
    };
    Err(Errors::Error401Unauthorized(
        "Identifiant ou mot de passe incorrect".to_string(),
    ))
}

pub async fn get_me(user: AuthenticatedUser) -> Json<Claims> {
    Json(user.claims)
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Json<Value>) {
    let new_jar = jar.remove(Cookie::from("lan_session"));
    (
        new_jar,
        Json(json!({ "message": "Déconnecté avec succès"
        })),
    )
}

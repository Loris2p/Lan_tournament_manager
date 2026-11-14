use crate::auth::{
    build_auth_cookie, create_jwt, hash_password, verify_jwt, verify_password, UserRole,
    DUMMY_HASH,
};
use crate::db;
use crate::errors::Errors;
use crate::models::AppState;
use crate::routes::auth::extract_client_ip;
use crate::routes::pages::common::HtmlTemplate;
use askama::Template;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::{CookieJar, Form};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub struct LoginPageQuery {
    pub error: Option<String>,
    pub registered: Option<String>,
}

#[derive(Clone)]
pub struct LoginGameOption {
    pub id: String,
    pub name: String,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub is_first_setup: bool,
    pub error: Option<String>,
    pub is_registered: bool,
    pub games: Vec<LoginGameOption>,
}

impl LoginTemplate {
    pub fn error_message(&self) -> Option<&'static str> {
        match self.error.as_deref() {
            Some("invalid_credentials") => Some("Identifiant ou mot de passe incorrect."),
            Some("account_pending") => {
                Some("Votre compte est en attente de validation par un administrateur.")
            }
            Some("rate_limited") => {
                Some("Trop de tentatives. Veuillez patienter une minute.")
            }
            Some("passwords_dont_match") => Some("Les mots de passe saisis ne correspondent pas."),
            Some("username_taken") => Some("Ce pseudo d'administrateur est déjà utilisé."),
            Some("captain_name_taken") => Some("Ce pseudo de capitaine est déjà utilisé."),
            Some("team_name_taken") => Some("Ce nom d'équipe est déjà utilisé."),
            Some("invalid_register") => {
                Some("Veuillez remplir tous les champs (mot de passe d'au moins 8 caractères).")
            }
            Some("setup_forbidden") => Some("L'initialisation a déjà été effectuée."),
            Some("server_error") => Some("Une erreur interne est survenue. Réessayez."),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SetupForm {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

pub async fn serve_login_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(query): Query<LoginPageQuery>,
) -> Response {
    if let Some(c) = jar.get("lan_session") {
        if let Ok(claims) = verify_jwt(c.value(), &state.jwt_secret) {
            return match claims.role {
                UserRole::Admin => Redirect::to("/admin").into_response(),
                UserRole::Captain { .. } => Redirect::to("/captain").into_response(),
            }
        }
    }

    let is_first_setup = db::is_setup_required(&state.db).await.unwrap_or(false);

    let is_registered = query.registered.is_some();
    let games = db::list_games(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|g| g.id.map(|id| LoginGameOption { id: id.to_hex(), name: g.name }))
        .collect();

    HtmlTemplate(LoginTemplate {
        is_first_setup,
        error: query.error,
        is_registered,
        games,
    })
    .into_response()
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub captain_username: String,
    pub password: String,
    pub password_confirm: String,
    #[serde(default)]
    pub game_id: String,
}

pub async fn handle_register_form(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Form(form): Form<RegisterForm>,
) -> Response {
    let client_ip = extract_client_ip(&headers);
    let key_rate = format!("register:{}", client_ip);
    if !state.rate_limiter.check(&key_rate) {
        return Redirect::to("/login?error=rate_limited").into_response();
    }

    let team_name = form.name.trim();
    let captain_username = form.captain_username.trim().to_lowercase();

    if team_name.is_empty() || captain_username.is_empty() || form.password.len() < 8 {
        return Redirect::to("/login?error=invalid_register").into_response();
    }

    if form.password != form.password_confirm {
        return Redirect::to("/login?error=passwords_dont_match").into_response();
    }

    let db = &state.db;

    let games = db::list_games(db).await.unwrap_or_default();
    let game_oid = if form.game_id.trim().is_empty() {
        if !games.is_empty() {
            return Redirect::to("/login?error=invalid_register").into_response();
        }
        None
    } else {
        match ObjectId::parse_str(form.game_id.trim()) {
            Ok(oid) if db::find_game_by_id(db, oid).await.is_ok() => Some(oid),
            _ => return Redirect::to("/login?error=invalid_register").into_response(),
        }
    };

    if db::find_admin_by_username(db, &captain_username).await.is_ok()
        || db::find_team_by_captain_username(db, &captain_username).await.is_ok()
    {
        return Redirect::to("/login?error=captain_name_taken").into_response();
    }

    let h_pwd = match hash_password(&form.password) {
        Ok(h) => h,
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    match db::register_team(db, team_name, &captain_username, &h_pwd, game_oid).await {
        Ok(_) => Redirect::to("/login?registered=1").into_response(),
        Err(Errors::Error409Conflict(_)) => {
            Redirect::to("/login?error=team_name_taken").into_response()
        }
        Err(_) => Redirect::to("/login?error=server_error").into_response(),
    }
}

pub async fn handle_login_form(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let client_ip = extract_client_ip(&headers);
    let username = form.username.trim().to_lowercase();
    let key_rate = format!("{}:{}", client_ip, username);
    if !state.rate_limiter.check(&key_rate) {
        return Redirect::to("/login?error=rate_limited").into_response();
    }

    let db = &state.db;
    let password = &form.password;
    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;

    let (target_hash, auth_info, is_verified) = if let Ok(admin) = db::find_admin_by_username(db, &username).await
    {
        let info = admin
            .id
            .map(|id| (id, admin.username, UserRole::Admin, admin.token_version));
        (Some(admin.password_hash), info, true)
    } else if let Ok(team) = db::find_team_by_captain_username(db, &username).await {
        let verified = team.is_verified;
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
        (Some(team.captain_password_hash), info, verified)
    } else {
        (None, None, false)
    };

    let hash_to_verify = target_hash.as_deref().unwrap_or(DUMMY_HASH);
    let is_valid = verify_password(password, hash_to_verify).unwrap_or(false);

    if is_valid {
        if !is_verified {
            return Redirect::to("/login?error=account_pending").into_response();
        }
        if let Some((user_id, user_name, role, token_version)) = auth_info {
            let target_url = match role {
                UserRole::Admin => "/admin",
                UserRole::Captain { .. } => "/captain",
            };
            if let Ok(token) = create_jwt(
                user_id,
                user_name,
                role,
                token_version,
                exp,
                &state.jwt_secret,
            ) {
                let new_jar = jar.add(build_auth_cookie(token));
                return (new_jar, Redirect::to(target_url)).into_response();
            }
        }
    }

    Redirect::to("/login?error=invalid_credentials").into_response()
}

pub async fn handle_setup_form(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<SetupForm>,
) -> Response {
    let db = &state.db;
    if !db::is_setup_required(db).await.unwrap_or(false) {
        return Redirect::to("/login?error=setup_forbidden").into_response();
    }

    if form.password != form.password_confirm {
        return Redirect::to("/login?error=passwords_dont_match").into_response();
    }

    let username = form.username.trim().to_lowercase();
    if db::find_team_by_captain_username(db, &username)
        .await
        .is_ok()
    {
        return Redirect::to("/login?error=username_taken").into_response();
    }

    let h_pswd = match hash_password(&form.password) {
        Ok(h) => h,
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    let admin = match db::create_initial_admin(db, &username, &h_pswd).await {
        Ok(a) => a,
        Err(Errors::Error403Forbidden(_)) => {
            return Redirect::to("/login?error=setup_forbidden").into_response();
        }
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    let admin_id = match admin.id {
        Some(id) => id,
        None => return Redirect::to("/login?error=server_error").into_response(),
    };

    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;
    let token = match create_jwt(
        admin_id,
        admin.username,
        UserRole::Admin,
        admin.token_version,
        exp,
        &state.jwt_secret,
    ) {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    let new_jar = jar.add(build_auth_cookie(token));
    (new_jar, Redirect::to("/admin")).into_response()
}

pub async fn handle_logout(State(state): State<Arc<AppState>>, jar: CookieJar) -> Response {
    if let Some(cookie) = jar.get("lan_session") {
        if let Ok(claims) = verify_jwt(cookie.value(), &state.jwt_secret) {
            if let Ok(oid) = ObjectId::parse_str(&claims.sub) {
                match claims.role {
                    UserRole::Admin => {
                        let _ = db::increment_admin_token_version(&state.db, oid).await;
                    }
                    UserRole::Captain { .. } => {
                        let _ = db::increment_team_token_version(&state.db, oid).await;
                    }
                }
            }
        }
    }
    let cookie = axum_extra::extract::cookie::Cookie::build("lan_session")
        .path("/")
        .build();
    let new_jar = jar.remove(cookie);
    (new_jar, Redirect::to("/login")).into_response()
}

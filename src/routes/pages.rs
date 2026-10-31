use std::sync::Arc;
use askama::Template;
use axum::extract::{Form, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use crate::auth::{verify_jwt, AdminUser, CaptainUser, UserRole};
use crate::models::AppState;

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur lors du rendu du template : {err}"),
            )
                .into_response(),
        }
    }
}

pub struct CaptainMatchView {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub instructions: String,
}

#[derive(Deserialize, Default)]
pub struct CaptainQuery {
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct SubmissionForm {
    pub match_id: String,
    pub placement: u8,
    #[serde(default)]
    pub secondary_stat: u32,
    pub proof_base64: String,
}

#[derive(Template)]
#[template(path = "captain.html")]
pub struct CaptainTemplate {
    pub team_name: String,
    pub total_points: u32,
    pub matches: Vec<CaptainMatchView>,
    pub status: Option<String>,
}

impl CaptainTemplate {
    pub fn is_success(&self) -> bool {
        self.status.as_deref() == Some("success")
    }

    pub fn error_message(&self) -> Option<&'static str> {
        match self.status.as_deref() {
            Some("invalid_match") => Some("Match sélectionné introuvable."),
            Some("match_closed") => Some("Ce match a déjà été clôturé par les arbitres."),
            Some("not_assigned") => Some("Votre équipe n'est pas inscrite à ce match."),
            Some("invalid_image") => Some("La preuve fournie doit être une image valide."),
            Some("invalid_score") => Some("Le score de placement ou la statistique secondaire est invalide."),
            Some("upload_failed") => Some("Échec de l'envoi de la capture d'écran sur le stockage."),
            Some("db_error") => Some("Erreur lors de l'enregistrement en base de données."),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub struct NavAuth {
    pub is_logged_in: bool,
    pub target_url: String,
    pub label: String,
}

impl NavAuth {
    pub fn from_jar(jar: &CookieJar, secret: &str) -> Self {
        if let Some(c) = jar.get("lan_session") {
            if let Ok(claims) = verify_jwt(c.value(), secret) {
                return match claims.role {
                    UserRole::Admin => Self {
                        is_logged_in: true,
                        target_url: "/admin".to_string(),
                        label: "Espace Arbitre".to_string(),
                    },
                    UserRole::Captain { .. } => Self {
                        is_logged_in: true,
                        target_url: "/captain".to_string(),
                        label: "Espace Équipe".to_string(),
                    },
                };
            }
        }
        Self {
            is_logged_in: false,
            target_url: "/login".to_string(),
            label: "Connexion".to_string(),
        }
    }
}

pub struct HomeGameView {
    pub name: String,
    pub format: String,
    pub cover_image: String,
    pub rules: String,
    pub placement_text: String,
    pub secondary_text: String,
    pub max_placement: u8,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub games: Vec<HomeGameView>,
    pub auth: NavAuth,
}

pub async fn serve_home_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let games_db = crate::db::list_games(&state.db).await.unwrap_or_default();
    let auth = NavAuth::from_jar(&jar, &state.jwt_secret);

    let games_view: Vec<HomeGameView> = games_db
        .into_iter()
        .map(|g| {
            let format = match g.format {
                crate::models::GameFormat::Duel => "DUEL 1V1".to_string(),
                crate::models::GameFormat::Brawl => "BRAWL // MULTI-ÉQUIPES".to_string(),
            };
            let placement_text = if !g.placement_points.is_empty() {
                g.placement_points
                    .iter()
                    .enumerate()
                    .take(4)
                    .map(|(i, pts)| format!("{}e: {}pts", i + 1, pts))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "Non configuré".to_string()
            };
            let secondary_text = if let Some(label) = g.secondary_stat_label {
                format!(
                    "{} ({} pt/unité, max {})",
                    label, g.points_per_secondary_stat, g.max_secondary_stat
                )
            } else {
                "Aucune statistique secondaire".to_string()
            };
            let rules = if !g.rules.trim().is_empty() {
                g.rules
            } else {
                "Consignes de match standard. Respecter le fair-play et les directives des arbitres.".to_string()
            };

            HomeGameView {
                name: g.name,
                format,
                cover_image: g.cover_image,
                rules,
                placement_text,
                secondary_text,
                max_placement: g.max_placement,
            }
        })
        .collect();

    HtmlTemplate(IndexTemplate {
        games: games_view,
        auth,
    })
    .into_response()
}

#[derive(Template)]
#[template(path = "leaderboard.html")]
pub struct LeaderboardTemplate {
    pub teams: Vec<crate::models::PublicTeam>,
    pub live_matches: Vec<crate::models::Match>,
    pub auth: NavAuth,
}

impl LeaderboardTemplate {
    pub fn first(&self) -> Option<&crate::models::PublicTeam> {
        self.teams.get(0)
    }
    pub fn second(&self) -> Option<&crate::models::PublicTeam> {
        self.teams.get(1)
    }
    pub fn third(&self) -> Option<&crate::models::PublicTeam> {
        self.teams.get(2)
    }
    pub fn remaining_teams(&self) -> &[crate::models::PublicTeam] {
        if self.teams.len() > 3 {
            &self.teams[3..]
        } else {
            &[]
        }
    }
}

pub async fn serve_leaderboard_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let teams = crate::db::get_leaderboard(&state.db).await.unwrap_or_default();
    let live_matches = crate::db::get_open_matches(&state.db).await.unwrap_or_default();
    let auth = NavAuth::from_jar(&jar, &state.jwt_secret);

    HtmlTemplate(LeaderboardTemplate {
        teams,
        live_matches,
        auth,
    })
    .into_response()
}

#[derive(Deserialize, Default)]
pub struct LoginPageQuery {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub is_first_setup: bool,
    pub error: Option<String>,
}

impl LoginTemplate {
    pub fn error_message(&self) -> Option<&'static str> {
        match self.error.as_deref() {
            Some("invalid_credentials") => Some("Identifiant ou mot de passe incorrect."),
            Some("rate_limited") => Some("Trop de tentatives de connexion. Veuillez patienter une minute."),
            Some("passwords_dont_match") => Some("Les mots de passe saisis ne correspondent pas."),
            Some("username_taken") => Some("Ce pseudo d'administrateur est déjà utilisé."),
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
            match claims.role {
                UserRole::Admin => return Redirect::to("/admin").into_response(),
                UserRole::Captain { .. } => return Redirect::to("/captain").into_response(),
            }
        }
    }

    let is_first_setup = crate::db::count_admins(&state.db).await.unwrap_or(1) == 0;

    HtmlTemplate(LoginTemplate {
        is_first_setup,
        error: query.error,
    })
    .into_response()
}

pub async fn handle_login_form(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Response {
    let client_ip = crate::routes::auth::extract_client_ip(&headers);
    if !state.rate_limiter.check(&client_ip) {
        return Redirect::to("/login?error=rate_limited").into_response();
    }

    let db = &state.db;
    let username = form.username.trim();
    let password = &form.password;
    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;

    if let Ok(admin) = crate::db::find_admin_by_username(db, username).await {
        if crate::auth::verify_password(password, &admin.password_hash).unwrap_or(false) {
            if let Some(admin_id) = admin.id {
                if let Ok(token) = crate::auth::create_jwt(
                    admin_id,
                    admin.username,
                    UserRole::Admin,
                    admin.token_version,
                    exp,
                    &state.jwt_secret,
                ) {
                    let new_jar = jar.add(crate::auth::build_auth_cookie(token));
                    return (new_jar, Redirect::to("/admin")).into_response();
                }
            }
        }
    } else if let Ok(team) = crate::db::find_team_by_captain_username(db, username).await {
        if crate::auth::verify_password(password, &team.captain_password_hash).unwrap_or(false) {
            if let Some(team_id) = team.id {
                if let Ok(token) = crate::auth::create_jwt(
                    team_id,
                    team.captain_username,
                    UserRole::Captain {
                        team_id: team_id.to_hex(),
                    },
                    team.token_version,
                    exp,
                    &state.jwt_secret,
                ) {
                    let new_jar = jar.add(crate::auth::build_auth_cookie(token));
                    return (new_jar, Redirect::to("/captain")).into_response();
                }
            }
        }
    }

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Redirect::to("/login?error=invalid_credentials").into_response()
}

pub async fn handle_setup_form(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Form(form): Form<SetupForm>,
) -> Response {
    let db = &state.db;
    let nb_admins = crate::db::count_admins(db).await.unwrap_or(1);
    if nb_admins != 0 {
        return Redirect::to("/login?error=setup_forbidden").into_response();
    }

    if form.password != form.password_confirm {
        return Redirect::to("/login?error=passwords_dont_match").into_response();
    }

    let username = form.username.trim();
    if crate::db::find_team_by_captain_username(db, username).await.is_ok() {
        return Redirect::to("/login?error=username_taken").into_response();
    }

    let h_pswd = match crate::auth::hash_password(&form.password) {
        Ok(h) => h,
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    let admin = match crate::db::create_admin(db, username, &h_pswd).await {
        Ok(a) => a,
        Err(_) => return Redirect::to("/login?error=server_error").into_response(),
    };

    let admin_id = match admin.id {
        Some(id) => id,
        None => return Redirect::to("/login?error=server_error").into_response(),
    };

    let exp = (chrono::Utc::now().timestamp() + 24 * 3600) as usize;
    let token = match crate::auth::create_jwt(
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

    let new_jar = jar.add(crate::auth::build_auth_cookie(token));
    (new_jar, Redirect::to("/admin")).into_response()
}

pub async fn handle_logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    if let Some(cookie) = jar.get("lan_session") {
        if let Ok(claims) = verify_jwt(cookie.value(), &state.jwt_secret) {
            if let Ok(oid) = ObjectId::parse_str(&claims.sub) {
                match claims.role {
                    UserRole::Admin => {
                        let _ = crate::db::increment_admin_token_version(&state.db, oid).await;
                    }
                    UserRole::Captain { .. } => {
                        let _ = crate::db::increment_team_token_version(&state.db, oid).await;
                    }
                }
            }
        }
    }
    let cookie = axum_extra::extract::cookie::Cookie::build("lan_session").path("/").build();
    let new_jar = jar.remove(cookie);
    (new_jar, Redirect::to("/login")).into_response()
}

pub struct AdminSubmissionView {
    pub id: String,
    pub team_name: String,
    pub placement: u8,
    pub secondary_stat: u32,
    pub calculated_points: u32,
    pub proof_url: String,
}

#[allow(dead_code)]
pub struct AdminGameView {
    pub id: String,
    pub name: String,
    pub format: String,
    pub rules: String,
    pub points_per_secondary_stat: u32,
}

pub struct AdminMatchView {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub is_open: bool,
    pub instructions: String,
    pub assigned_teams_count: usize,
}

pub struct AdminUserView {
    pub id: String,
    pub username: String,
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    pub admin_username: String,
    pub submissions: Vec<AdminSubmissionView>,
    pub games: Vec<AdminGameView>,
    pub matches: Vec<AdminMatchView>,
    pub teams: Vec<crate::models::PublicTeam>,
    pub admins: Vec<AdminUserView>,
}

pub async fn serve_admin_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let (admin_oid, admin_username) = match jar.get("lan_session") {
        Some(c) => match verify_jwt(c.value(), &state.jwt_secret) {
            Ok(claims) if matches!(claims.role, UserRole::Admin) => {
                match ObjectId::parse_str(&claims.sub) {
                    Ok(oid) => {
                        let valid = crate::db::verify_admin_session(&state.db, oid, claims.token_version)
                            .await
                            .unwrap_or(false);
                        if valid {
                            (Some(oid), claims.username)
                        } else {
                            (None, String::new())
                        }
                    }
                    Err(_) => (None, String::new()),
                }
            }
            _ => (None, String::new()),
        },
        None => (None, String::new()),
    };

    if admin_oid.is_none() {
        return Redirect::to("/login").into_response();
    }

    let submissions_db = crate::db::get_pending(&state.db).await.unwrap_or_default();
    let games_db = crate::db::list_games(&state.db).await.unwrap_or_default();
    let matches_db = crate::db::get_all_matches(&state.db).await.unwrap_or_default();
    let teams_db = crate::db::get_leaderboard(&state.db).await.unwrap_or_default();
    let admins_db = crate::db::list_admins(&state.db).await.unwrap_or_default();

    let submissions_view: Vec<AdminSubmissionView> = submissions_db
        .into_iter()
        .map(|s| AdminSubmissionView {
            id: s.id.map(|id| id.to_hex()).unwrap_or_default(),
            team_name: s.team_name,
            placement: s.score.placement,
            secondary_stat: s.score.secondary_stat,
            calculated_points: s.calculated_points,
            proof_url: s.proof_base64.unwrap_or_default(),
        })
        .collect();

    let games_view: Vec<AdminGameView> = games_db
        .into_iter()
        .map(|g| AdminGameView {
            id: g.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: g.name,
            format: match g.format {
                crate::models::GameFormat::Duel => "DUEL".to_string(),
                crate::models::GameFormat::Brawl => "BRAWL".to_string(),
            },
            rules: g.rules,
            points_per_secondary_stat: g.points_per_secondary_stat,
        })
        .collect();

    let matches_view: Vec<AdminMatchView> = matches_db
        .into_iter()
        .map(|m| AdminMatchView {
            id: m.id.map(|id| id.to_hex()).unwrap_or_default(),
            title: m.title,
            game_name: m.game_name,
            is_open: m.is_open,
            instructions: m.instructions,
            assigned_teams_count: m.assigned_teams.len(),
        })
        .collect();

    let admins_view: Vec<AdminUserView> = admins_db
        .into_iter()
        .map(|a| AdminUserView {
            id: a.id.map(|id| id.to_hex()).unwrap_or_default(),
            username: a.username,
        })
        .collect();

    let template = AdminTemplate {
        admin_username,
        submissions: submissions_view,
        games: games_view,
        matches: matches_view,
        teams: teams_db,
        admins: admins_view,
    };

    HtmlTemplate(template).into_response()
}

pub async fn serve_captain_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Query(query): Query<CaptainQuery>,
) -> Response {
    let team_oid_opt = match jar.get("lan_session") {
        Some(c) => match verify_jwt(c.value(), &state.jwt_secret) {
            Ok(claims) if matches!(claims.role, UserRole::Captain { .. }) => {
                match ObjectId::parse_str(&claims.sub) {
                    Ok(oid) => {
                        let valid = crate::db::verify_team_session(&state.db, oid, claims.token_version)
                            .await
                            .unwrap_or(false);
                        if valid {
                            Some(oid)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            }
            _ => None,
        },
        None => None,
    };

    let team_oid = match team_oid_opt {
        Some(oid) => oid,
        None => return Redirect::to("/login.html").into_response(),
    };

    let team = match crate::db::find_team_by_id(&state.db, team_oid).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login.html").into_response(),
    };

    let matches_db = crate::db::get_open_matches_for_team(&state.db, team_oid)
        .await
        .unwrap_or_default();

    let matches_view: Vec<CaptainMatchView> = matches_db
        .into_iter()
        .map(|m| CaptainMatchView {
            id: m.id.map(|id| id.to_hex()).unwrap_or_default(),
            title: m.title,
            game_name: m.game_name,
            instructions: m.instructions,
        })
        .collect();

    let template = CaptainTemplate {
        team_name: team.name,
        total_points: team.total_points,
        matches: matches_view,
        status: query.status,
    };

    HtmlTemplate(template).into_response()
}

pub async fn submit_captain_score(
    State(state): State<Arc<AppState>>,
    captain: CaptainUser,
    Form(form): Form<SubmissionForm>,
) -> Response {
    let match_oid = match ObjectId::parse_str(&form.match_id) {
        Ok(oid) => oid,
        Err(_) => return Redirect::to("/captain?status=invalid_match").into_response(),
    };

    let team = match crate::db::find_team_by_id(&state.db, captain.team_id).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login.html").into_response(),
    };

    let m = match crate::db::find_match_by_id(&state.db, match_oid).await {
        Ok(m) => m,
        Err(_) => return Redirect::to("/captain?status=invalid_match").into_response(),
    };

    let game = match crate::db::find_game_by_id(&state.db, m.game_id).await {
        Ok(g) => g,
        Err(_) => return Redirect::to("/captain?status=invalid_match").into_response(),
    };

    if !m.is_open {
        return Redirect::to("/captain?status=match_closed").into_response();
    }

    if !m.assigned_teams.contains(&captain.team_id) {
        return Redirect::to("/captain?status=not_assigned").into_response();
    }

    if !form.proof_base64.starts_with("data:image/") {
        return Redirect::to("/captain?status=invalid_image").into_response();
    }

    let score = crate::models::SubmissionScore {
        placement: form.placement,
        secondary_stat: form.secondary_stat,
    };

    if game.validate_score(&score).is_err() {
        return Redirect::to("/captain?status=invalid_score").into_response();
    }

    let proof_url = match state.s3.upload_base_image(&form.proof_base64, "submissions").await {
        Ok(url) => url,
        Err(_) => return Redirect::to("/captain?status=upload_failed").into_response(),
    };

    let calculated_points = game.calculate_points(&score);

    if crate::db::insert_submission(
        &state.db,
        match_oid,
        captain.team_id,
        team.name,
        score,
        Some(proof_url),
        calculated_points,
        crate::models::SubmissionStatus::Pending,
    )
    .await
    .is_err()
    {
        return Redirect::to("/captain?status=db_error").into_response();
    }

    let _ = state.ws_tx.send("NEW_SUBMISSION".to_string());

    Redirect::to("/captain?status=success").into_response()
}

pub async fn admin_approve_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    if crate::db::approve_or_reject_submission(
        &state.db,
        id,
        crate::models::SubmissionStatus::Approved,
    )
    .await
    .is_ok()
    {
        let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=arbitrage").into_response()
}

pub async fn admin_reject_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = crate::db::approve_or_reject_submission(
        &state.db,
        id,
        crate::models::SubmissionStatus::Rejected,
    )
    .await;
    Redirect::to("/admin?tab=arbitrage").into_response()
}

pub async fn admin_toggle_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = crate::db::toggle_match_status(&state.db, id).await;
    Redirect::to("/admin?tab=matches").into_response()
}

pub async fn admin_delete_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = crate::db::delete_game(&state.db, id).await;
    Redirect::to("/admin?tab=games").into_response()
}

pub async fn admin_delete_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = crate::db::delete_team(&state.db, id).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=teams").into_response()
}

pub async fn admin_delete_admin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = crate::db::delete_admin(&state.db, id).await;
    Redirect::to("/admin?tab=admins").into_response()
}

#[derive(Deserialize)]
pub struct AdjustPointsForm {
    pub adjustment: i32,
}

pub async fn admin_adjust_points(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Form(form): Form<AdjustPointsForm>,
) -> Response {
    let _ = crate::db::adjust_team_points(&state.db, id, form.adjustment).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=teams").into_response()
}

#[derive(Deserialize)]
pub struct CreateGameForm {
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub cover_image: String,
    pub placement_points: String,
    #[serde(default)]
    pub default_placement_points: u32,
    #[serde(default = "default_two")]
    pub max_placement: u8,
    pub secondary_stat_label: Option<String>,
    #[serde(default)]
    pub points_per_secondary_stat: u32,
    #[serde(default)]
    pub max_secondary_stat: u32,
    #[serde(default)]
    pub rules: String,
}

fn default_two() -> u8 {
    2
}

pub async fn admin_create_game_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateGameForm>,
) -> Response {
    let format = match form.format.as_str() {
        "duel" => crate::models::GameFormat::Duel,
        _ => crate::models::GameFormat::Brawl,
    };
    let placement_points: Vec<u32> = form
        .placement_points
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();
    let sec_label = form
        .secondary_stat_label
        .filter(|s| !s.trim().is_empty());

    let _ = crate::db::create_game(
        &state.db,
        &form.name,
        &form.cover_image,
        format,
        placement_points,
        form.default_placement_points,
        sec_label,
        form.points_per_secondary_stat,
        form.max_placement,
        form.max_secondary_stat,
        &form.rules,
    )
    .await;

    Redirect::to("/admin?tab=games").into_response()
}

#[derive(Deserialize)]
pub struct CreateTeamForm {
    pub name: String,
    pub captain_username: String,
    pub captain_password: String,
}

pub async fn admin_create_team_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateTeamForm>,
) -> Response {
    if let Ok(hash) = crate::auth::hash_password(&form.captain_password) {
        let _ = crate::db::create_team(
            &state.db,
            &form.name,
            &form.captain_username,
            &hash,
        )
        .await;
        let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=teams").into_response()
}

#[derive(Deserialize)]
pub struct CreateAdminForm {
    pub username: String,
    pub password: String,
}

pub async fn admin_create_admin_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateAdminForm>,
) -> Response {
    if let Ok(hash) = crate::auth::hash_password(&form.password) {
        let _ = crate::db::create_admin(&state.db, &form.username, &hash).await;
    }
    Redirect::to("/admin?tab=admins").into_response()
}

#[derive(Deserialize)]
pub struct CreateMatchForm {
    pub game_id: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub match_teams: Vec<String>,
}

pub async fn admin_create_match_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateMatchForm>,
) -> Response {
    if let Ok(game_oid) = ObjectId::parse_str(&form.game_id) {
        if let Ok(game) = crate::db::find_game_by_id(&state.db, game_oid).await {
            let teams_oids: Vec<ObjectId> = form
                .match_teams
                .iter()
                .filter_map(|t| ObjectId::parse_str(t).ok())
                .collect();
            let _ = crate::db::create_match(
                &state.db,
                &game.name,
                game_oid,
                &game.name,
                teams_oids,
                &form.instructions,
                true,
            )
            .await;
            let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
        }
    }
    Redirect::to("/admin?tab=matches").into_response()
}

#[derive(Deserialize)]
pub struct UpdateInstructionsForm {
    pub instructions: String,
}

pub async fn admin_update_instructions_form(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Form(form): Form<UpdateInstructionsForm>,
) -> Response {
    let _ = crate::db::update_match_instructions(&state.db, id, &form.instructions).await;
    let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
    Redirect::to("/admin?tab=matches").into_response()
}

#[derive(Deserialize)]
pub struct ResetTournamentForm {
    pub scope: String,
}

pub async fn admin_reset_tournament_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<ResetTournamentForm>,
) -> Response {
    let scope = match form.scope.as_str() {
        "ScoresOnly" => crate::models::ResetScope::ScoresOnly,
        "UsersAndTeams" => crate::models::ResetScope::UsersAndTeams,
        _ => crate::models::ResetScope::FullTournament,
    };
    let _ = crate::db::reset_tournament(&state.db, scope).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=danger").into_response()
}

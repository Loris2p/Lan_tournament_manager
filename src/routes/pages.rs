use std::sync::Arc;
use askama::Template;
use axum::extract::{Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::CookieJar;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use crate::auth::{verify_jwt, CaptainUser, UserRole};
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

use crate::auth::{verify_jwt, CaptainUser, UserRole};
use crate::db;
use crate::models::{
    AppState, Game, Match, PlayerSubmission, PlayerSubmissionInput, SubmissionScore,
    SubmissionStatus,
};
use crate::routes::pages::common::{
    get_tournament_status_view, HtmlTemplate, TournamentStatusView,
};
use askama::Template;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::{CookieJar, Form};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CaptainMatchView {
    pub id: String,
    pub match_data: Match,
    pub game: Game,
}

impl CaptainMatchView {
    pub fn new(m: Match, game: Game) -> Self {
        Self {
            id: m.id.map(|id| id.to_hex()).unwrap_or_default(),
            match_data: m,
            game,
        }
    }
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
    #[serde(default)]
    pub proof_base64: Option<String>,
    #[serde(default)]
    pub player_submissions_json: Option<String>,
}

#[derive(Template)]
#[template(path = "captain.html")]
pub struct CaptainTemplate {
    pub team_name: String,
    pub total_points: u32,
    pub matches: Vec<CaptainMatchView>,
    pub status: Option<String>,
    pub tournament_status: TournamentStatusView,
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
            Some("invalid_score") => {
                Some("Le score de placement ou la statistique secondaire est invalide.")
            }
            Some("upload_failed") => {
                Some("Échec de l'envoi de la capture d'écran sur le stockage.")
            }
            Some("db_error") => Some("Erreur lors de l'enregistrement en base de données."),
            _ => None,
        }
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
                        let valid = db::verify_team_session(&state.db, oid, claims.token_version)
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
        None => {
            let cookie = axum_extra::extract::cookie::Cookie::build("lan_session")
                .path("/")
                .build();
            let new_jar = jar.remove(cookie);
            return (new_jar, Redirect::to("/login.html")).into_response();
        }
    };
    let team = match db::find_team_by_id(&state.db, team_oid).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login.html").into_response(),
    };

    let matches_db = db::get_open_matches_for_team(&state.db, team_oid)
        .await
        .unwrap_or_default();
    let games = db::list_games(&state.db).await.unwrap_or_default();
    let games_map: HashMap<ObjectId, Game> = games
        .into_iter()
        .filter_map(|g| g.id.map(|id| (id, g)))
        .collect();

    let matches_view: Vec<CaptainMatchView> = matches_db
        .into_iter()
        .filter_map(|m| {
            let game = games_map.get(&m.game_id)?.clone();
            Some(CaptainMatchView::new(m, game))
        })
        .collect();

    let tournament_status = get_tournament_status_view(&state.db).await;

    let template = CaptainTemplate {
        team_name: team.name,
        total_points: team.total_points,
        matches: matches_view,
        status: query.status,
        tournament_status,
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

    let team = match db::find_team_by_id(&state.db, captain.team_id).await {
        Ok(t) => t,
        Err(_) => return Redirect::to("/login.html").into_response(),
    };

    let m = match db::find_match_by_id(&state.db, match_oid).await {
        Ok(m) => m,
        Err(_) => return Redirect::to("/captain?status=invalid_match").into_response(),
    };

    let game = match db::find_game_by_id(&state.db, m.game_id).await {
        Ok(g) => g,
        Err(_) => return Redirect::to("/captain?status=invalid_match").into_response(),
    };

    if !m.is_open {
        return Redirect::to("/captain?status=match_closed").into_response();
    }

    if !m.assigned_teams.contains(&captain.team_id) {
        return Redirect::to("/captain?status=not_assigned").into_response();
    }

    let parsed_player_inputs: Option<Vec<PlayerSubmissionInput>> = form
        .player_submissions_json
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| serde_json::from_str(s).ok());

    let mut secondary_stat = form.secondary_stat;
    let (proof_url, player_submissions) = if let Some(player_inputs) = parsed_player_inputs {
        if !player_inputs.is_empty() {
            let mut uploaded = Vec::with_capacity(player_inputs.len());
            for p in &player_inputs {
                if !p.proof_base64.starts_with("data:image/") {
                    return Redirect::to("/captain?status=invalid_image").into_response();
                }
                let url = match state.s3.upload_base_image(&p.proof_base64, "submissions").await {
                    Ok(u) => u,
                    Err(err) => {
                        eprintln!("problème avec l'envoi de la preuve joueur : {err}");
                        return Redirect::to("/captain?status=upload_failed").into_response();
                    }
                };
                uploaded.push(PlayerSubmission {
                    player_number: p.player_number,
                    score: p.score,
                    proof_url: url,
                });
            }
            if secondary_stat == 0 {
                secondary_stat = uploaded.iter().map(|p| p.score).sum();
            }
            let first_url = uploaded.first().map(|p| p.proof_url.clone());
            (first_url, Some(uploaded))
        } else {
            let single_proof = form.proof_base64.as_deref().unwrap_or("");
            if !single_proof.starts_with("data:image/") {
                return Redirect::to("/captain?status=invalid_image").into_response();
            }
            let url = match state.s3.upload_base_image(single_proof, "submissions").await {
                Ok(u) => u,
                Err(err) => {
                    eprintln!("problème avec l'envoi de la preuve : {err}");
                    return Redirect::to("/captain?status=upload_failed").into_response();
                }
            };
            (Some(url), None)
        }
    } else {
        let single_proof = form.proof_base64.as_deref().unwrap_or("");
        if !single_proof.starts_with("data:image/") {
            return Redirect::to("/captain?status=invalid_image").into_response();
        }
        let url = match state.s3.upload_base_image(single_proof, "submissions").await {
            Ok(u) => u,
            Err(err) => {
                eprintln!("problème avec l'envoi de la preuve : {err}");
                return Redirect::to("/captain?status=upload_failed").into_response();
            }
        };
        (Some(url), None)
    };

    let score = SubmissionScore {
        placement: form.placement,
        secondary_stat,
    };

    if game.validate_score(&score).is_err() {
        return Redirect::to("/captain?status=invalid_score").into_response();
    }
    if db::has_submission_for_match(&state.db, match_oid, captain.team_id)
        .await
        .unwrap_or(false)
    {
        return Redirect::to("/captain?status=already_submitted").into_response();
    }

    let calculated_points = game.calculate_points(&score);

    if db::insert_submission(
        &state.db,
        match_oid,
        captain.team_id,
        team.name,
        score,
        proof_url,
        calculated_points,
        SubmissionStatus::Pending,
        player_submissions,
    )
    .await
    .is_err()
    {
        return Redirect::to("/captain?status=db_error").into_response();
    }

    let _ = state.ws_tx.send("NEW_SUBMISSION".to_string());

    Redirect::to("/captain?status=success").into_response()
}

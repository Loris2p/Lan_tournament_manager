use crate::auth::CaptainUser;
use crate::db::{
    find_game_by_id, find_match_by_id, find_team_by_id, get_open_matches_for_team,
    insert_submission,
};
use crate::errors::Errors;
use crate::errors::Errors::{Error400BadRequest, Error403Forbidden};
use crate::models::{AppState, CreateSubmissionRequest, Match, Submission, SubmissionStatus};
use axum::Json;
use axum::extract::State;

use std::sync::Arc;

pub async fn list_open_matches(
    State(state): State<Arc<AppState>>,
    user: CaptainUser,
) -> Result<Json<Vec<Match>>, Errors> {
    let db = &state.db;
    let res = get_open_matches_for_team(db, user.team_id).await?;
    Ok(Json(res))
}

pub async fn create_submission(
    State(state): State<Arc<AppState>>,
    user: CaptainUser,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<Json<Submission>, Errors> {
    let db = &state.db;

    let team = find_team_by_id(db, user.team_id).await?;
    let m = find_match_by_id(db, req.match_id).await?;
    let game = find_game_by_id(db, m.game_id).await?;
    if !m.is_open {
        return Err(Error400BadRequest("Le match est fermé".to_string()));
    }
    if !m.assigned_teams.contains(&user.team_id) {
        return Err(Error403Forbidden(
            "Votre équipe n'est pas inscrite à ce match".to_string(),
        ));
    }
    if !req.proof_base64.starts_with("data:image/") {
        return Err(Error400BadRequest(
            "Le preuve doit être un image".to_string(),
        ));
    }
    game.validate_score(&req.score)?;
    let calculated_points = game.calculate_points(&req.score);

    let res = insert_submission(
        db,
        req.match_id,
        user.team_id,
        team.name,
        req.score,
        Some(req.proof_base64),
        calculated_points,
        SubmissionStatus::Pending,
    )
    .await?;
    let _ = state.ws_tx.send("NEW_SUBMISSION".to_string());
    Ok(Json(res))
}

use crate::auth::AdminUser;
use crate::db;
use crate::db::find_team_by_id;
use crate::errors::Errors;
use crate::models::{AdminBatchScoresRequest, AppState, Submission, SubmissionStatus};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn list_pending_submissions(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Vec<Submission>>, Errors> {
    let res = db::get_pending(&state.db).await?;
    Ok(Json(res))
}

pub async fn approve_submission(
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    let submission = db::find_submission_by_id(&state.db, submission_id).await?;
    if submission.status != SubmissionStatus::Pending {
        return Err(Errors::Error400BadRequest(
            "Cette soumission a déjà été traitée".to_string(),
        ));
    }
    db::approve_or_reject_submission(&state.db, submission_id, SubmissionStatus::Approved).await?;
    db::add_points_to_team(&state.db, submission.team_id, submission.calculated_points).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}
pub async fn reject_submission(
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::approve_or_reject_submission(&state.db, submission_id, SubmissionStatus::Rejected).await?;
    Ok(Json(json!({"success":true})))
}

pub async fn submit_match_scores(
    State(state): State<Arc<AppState>>,
    Path(match_id): Path<ObjectId>,
    _admin: AdminUser,
    Json(req): Json<AdminBatchScoresRequest>,
) -> Result<Json<Value>, Errors> {
    let m = db::find_match_by_id(&state.db, match_id).await?;
    let game = db::find_game_by_id(&state.db, m.game_id).await?;
    for e in &req.results {
        game.validate_score(&e.score)?;
        db::find_team_by_id(&state.db, e.team_id).await?;
    }
    for e in req.results {
        let pts = game.calculate_points(&e.score);
        let team_name = find_team_by_id(&state.db, e.team_id).await?.name;
        db::insert_submission(
            &state.db,
            match_id,
            e.team_id,
            team_name,
            e.score,
            None,
            pts,
            SubmissionStatus::Approved,
        )
        .await?;
        db::add_points_to_team(&state.db, e.team_id, pts).await?;
    }
    if req.close_match && m.is_open {
        db::toggle_match_status(&state.db, match_id).await?;
    }
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}

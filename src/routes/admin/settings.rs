use crate::auth::AdminUser;
use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, TournamentSettings, UpdateTournamentSettingsRequest};
use axum::Json;
use axum::extract::State;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn get_settings(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<TournamentSettings>, Errors> {
    let res_db = db::get_tournament_settings(&state.db).await?;
    Ok(Json(res_db))
}
pub async fn set_settings(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<UpdateTournamentSettingsRequest>,
) -> Result<Json<Value>, Errors> {
    let recalculate = req.global_placement_points.is_some();
    db::update_tournament_settings(&state.db, &req).await?;
    if recalculate {
        db::recalculate_tournament_scores(&state.db).await?;
        let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    }
    let _ = state.ws_tx.send("TOURNAMENT_SCHEDULE_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}


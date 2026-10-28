use crate::auth::AdminUser;
use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, ResetTournamentRequest};
use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn reset_tournament(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<ResetTournamentRequest>,
) -> Result<Json<Value>, Errors> {
    db::reset_tournament(&state.db, req.scope).await?;
    let _ =&state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}

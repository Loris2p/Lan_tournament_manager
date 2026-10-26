use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, Game, PublicTeam};
use axum::Json;
use axum::extract::State;
use std::sync::Arc;

pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PublicTeam>>, Errors> {
    Ok(Json(db::get_leaderboard(&state.db).await?))
}
pub async fn list_games(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Game>>, Errors> {
    Ok(Json(db::list_games(&state.db).await?))
}

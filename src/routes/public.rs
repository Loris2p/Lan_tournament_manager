use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, GameResponse, PublicTeam};
use axum::Json;
use axum::extract::State;
use mongodb::bson::doc;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PublicTeam>>, Errors> {
    Ok(Json(db::get_leaderboard(&state.db).await?))
}
pub async fn list_games(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<GameResponse>>, Errors> {
    let games = db::list_games(&state.db).await?;
    let response: Vec<GameResponse> = games.into_iter().map(GameResponse::from).collect();
    Ok(Json(response))
}
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<Value>, Errors> {
    let _ = &state.db.run_command(doc! { "ping": 1 }).await?;
    Ok(Json(json!({"status":"ok", "db":"connected"})))
}

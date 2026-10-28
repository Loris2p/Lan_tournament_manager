use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, Game, Match, PublicTeam};
use axum::extract::State;
use axum::{ Json};
use mongodb::bson::doc;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn get_leaderboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PublicTeam>>, Errors> {
    Ok(Json(db::get_leaderboard(&state.db).await?))
}
pub async fn list_games(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Game>>, Errors> {
    Ok(Json(db::list_games(&state.db).await?))
}
pub async fn health_check(State(state): State<Arc<AppState>>) -> Result<Json<Value>, Errors> {
    let _ = &state.db.run_command(doc! { "ping": 1 }).await?;
    Ok(Json(json!({"status":"ok", "db":"connected"})))
}
pub async fn list_open_matches(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Match>>, Errors> {
    Ok(Json(db::get_open_matches(&state.db).await?))
}

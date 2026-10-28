use crate::auth::AdminUser;
use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, CreateMatchRequest, UpdateMatchInstructionsRequest};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn create_match(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<CreateMatchRequest>,
) -> Result<Json<Value>, Errors> {
    let game_name = db::find_game_by_id(&state.db, req.game_id).await?.name;
    let res = db::create_match(
        &state.db,
        &req.title,
        req.game_id,
        &game_name,
        req.assigned_teams,
        &req.instructions,
        req.is_open,
    )
    .await?;
    Ok(Json(json!(res)))
}

pub async fn update_match_instructions(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Json(req): Json<UpdateMatchInstructionsRequest>,
) -> Result<Json<Value>, Errors> {
    db::update_match_instructions(&state.db, id, &req.instructions).await?;
    Ok(Json(json!({"success":true})))
}
pub async fn toggle_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::toggle_match_status(&state.db, id).await?;
    Ok(Json(json!({"success":true})))
}

use crate::auth::AdminUser;
use crate::db;
use crate::errors::Errors;
use crate::models::{AppState, CreateGameRequest};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn create_game(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateGameRequest>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::create_game(
        &state.db,
        &req.name,
        &req.cover_image,
        req.format,
        req.placement_points,
        req.default_placement_points,
        req.secondary_stat_label,
        req.points_per_secondary_stat,
        req.max_placement,
        req.max_secondary_stat,
        &req.rules,
    )
    .await?;
    Ok(Json(json!({"success":true})))
}
pub async fn delete_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::delete_game(&state.db, id).await?;
    Ok(Json(json!({"success":true})))
}

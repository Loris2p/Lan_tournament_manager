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
    _admin: AdminUser,
    Json(req): Json<CreateGameRequest>,
) -> Result<Json<Value>, Errors> {
    let cover_image = match &req.cover_image.starts_with("data:image/") {
        true => {
            &state
                .s3
                .upload_base_image(&req.cover_image, "games")
                .await?
        }
        false => &req.cover_image,
    };
    db::create_game(
        &state.db,
        &req.name,
        &cover_image,
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

use std::sync::Arc;
use crate::auth::AdminUser;
use crate::errors::Errors;
use crate::models::{AppState, CreateTeamRequest};
use crate::{auth, db};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<Value>, Errors> {
    if db::find_team_by_captain_username(&state.db, &req.captain_username)
        .await
        .is_ok()
        || db::find_admin_by_username(&state.db, &req.captain_username)
            .await
            .is_ok()
    {
        return Err(Errors::Error409Conflict(
            "Nom déjà utilisé par un capitaine ou un Admin".to_string(),
        ));
    };
    let h_pwd = auth::hash_password(&req.captain_password)?;
    db::create_team(&state.db, &req.name, &req.captain_username, &h_pwd).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}
pub async fn delete_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {

    db::delete_team(&state.db,id ).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());

    Ok(Json(json!({"success":true})))
}
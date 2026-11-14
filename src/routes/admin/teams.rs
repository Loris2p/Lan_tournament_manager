use crate::auth::AdminUser;
use crate::errors::Errors;
use crate::models::{AppState, CreateTeamRequest, UpdateTeamPointsRequest};
use crate::{auth, db};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn create_team(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<CreateTeamRequest>,
) -> Result<Json<Value>, Errors> {
    let captain_username = req.captain_username.trim().to_lowercase();
    if db::find_team_by_captain_username(&state.db, &captain_username)
        .await
        .is_ok()
        || db::find_admin_by_username(&state.db, &captain_username)
            .await
            .is_ok()
    {
        return Err(Errors::Error409Conflict(
            "Nom déjà utilisé par un capitaine ou un Admin".to_string(),
        ));
    };
    let h_pwd = auth::hash_password(&req.captain_password)?;
    db::create_team(&state.db, &req.name, &captain_username, &h_pwd, req.game_id).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({"success":true})))
}
pub async fn delete_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::delete_team(&state.db, id).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());

    Ok(Json(json!({"success":true})))
}
pub async fn update_teams_points(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Json(req): Json<UpdateTeamPointsRequest>,
) -> Result<Json<Value>, Errors> {
    let new_points = db::adjust_team_points(&state.db, id, req.adjustment).await?;
    let _ = &state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({ "success": true, "points": new_points })))
}

pub async fn approve_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    db::approve_team(&state.db, id).await?;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Ok(Json(json!({ "success": true })))
}

pub async fn get_pending_teams(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    let teams = db::get_pending_teams(&state.db).await?;
    let games = db::list_games(&state.db).await.unwrap_or_default();
    let games_map: std::collections::HashMap<ObjectId, String> = games
        .into_iter()
        .filter_map(|g| g.id.map(|id| (id, g.name)))
        .collect();
    let views: Vec<crate::models::PublicTeam> = teams
        .into_iter()
        .filter_map(|t| {
            t.id.map(|id| crate::models::PublicTeam {
                id: id.to_hex(),
                name: t.name,
                captain_username: t.captain_username,
                total_points: t.total_points,
                game_id: t.game_id.map(|gid| gid.to_hex()),
                game_name: t.game_id.and_then(|gid| games_map.get(&gid).cloned()),
            })
        })
        .collect();
    Ok(Json(json!(views)))
}

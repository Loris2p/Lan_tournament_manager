use crate::auth::{AdminUser, hash_password};
use crate::db;
use crate::db::{find_admin_by_username, find_team_by_captain_username};
use crate::errors::Errors;
use crate::models::{AppState, CreateAdminRequest};
use axum::Json;
use axum::extract::{Path, State};
use mongodb::bson::oid::ObjectId;
use serde_json::{Value, json};
use std::sync::Arc;

pub async fn list_admins(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
) -> Result<Json<Vec<Value>>, Errors> {
    let db = &state.db;
    let res_db = db::list_admins(db).await?;
    let res: Vec<Value> = res_db
        .into_iter()
        .map(|a| json!({"id":a.id, "username":a.username}))
        .collect();
    Ok(Json(res))
}
pub async fn create_admin(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Json(req): Json<CreateAdminRequest>,
) -> Result<Json<Value>, Errors> {
    let db = &state.db;
    let username = req.username;
    let password = req.password;
    if find_team_by_captain_username(db, &username).await.is_ok()
        || find_admin_by_username(db, &username).await.is_ok()
    {
        return Err(Errors::Error409Conflict(
            "Nom déjà utilisé par un capitaine ou un Admin".to_string(),
        ));
    };
    let h_pwd = hash_password(&password)?;
    let res = db::create_admin(db, &username, &h_pwd).await?;
    Ok(Json(json!({"id": res.id, "username":res.username})))
}

pub async fn delete_admin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Result<Json<Value>, Errors> {
    let db = &state.db;
    if db::count_admins(db).await? <= 1 {
        return Err(Errors::Error400BadRequest(
            "Impossible de supprimer le dernier Admin".to_string(),
        ));
    }
    db::delete_admin(db, id).await?;
    Ok(Json(json!({"success":true})))
}

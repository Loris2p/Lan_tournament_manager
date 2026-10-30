use crate::errors::Errors;
use crate::models::Admin;
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
pub async fn count_admins(db: &Database) -> Result<u64, Errors> {
    let count = db
        .collection::<Admin>("admins")
        .count_documents(doc! {})
        .await?;
    Ok(count)
}
pub async fn find_admin_by_username(
    db: &Database,
    username: &str,
) -> Result<Admin, Errors> {
    let admin = db
        .collection::<Admin>("admins")
        .find_one(doc! {"username":username})
        .await?;
    match admin {
        Some(s) => Ok(s),
        None => Err(Errors::Error404NotFound("Admin introuvable".to_string())),
    }

}
pub async fn list_admins(db: &Database) -> Result<Vec<Admin>, Errors> {
    let admins = db
        .collection::<Admin>("admins")
        .find(doc! {})
        .await?
        .try_collect()
        .await?; // try_collect() : pour récup le résultat sous forme de vec et non 1 par 1
    Ok(admins)
}
pub async fn create_admin(
    db: &Database,
    username: &str,
    password_hash: &str,
) -> Result<Admin, Errors> {
    let mut new_admin = Admin {
        id: None,
        username: username.to_string(),
        password_hash: password_hash.to_string(),
        token_version: 0,
    };
    let res = db
        .collection::<Admin>("admins")
        .insert_one(&new_admin)
        .await?;
    new_admin.id = res.inserted_id.as_object_id();
    Ok(new_admin)
}
pub async fn delete_admin(db: &Database, admin_id: ObjectId) -> Result<(), Errors> {
    if count_admins(db).await? <= 1 {
        return Err(Errors::Error400BadRequest(
            "Impossible de supprimer le dernier administrateur !".to_string(),
        ));
    }
    let res = db
        .collection::<Admin>("admins")
        .delete_one(doc! {"_id":admin_id})
        .await?;
    if res.deleted_count == 0 {
        return Err(Errors::Error404NotFound("Admin introuvable".to_string()));
    }
    Ok(())
}

pub async fn increment_admin_token_version(db: &Database, admin_id: ObjectId) -> Result<(), Errors> {
    db.collection::<Admin>("admins")
        .update_one(doc! {"_id": admin_id}, doc! {"$inc": {"token_version": 1}})
        .await?;
    Ok(())
}

pub async fn verify_admin_session(db: &Database, admin_id: ObjectId, token_version: u32) -> Result<bool, Errors> {
    let count = db
        .collection::<Admin>("admins")
        .count_documents(doc! {"_id": admin_id, "token_version": token_version})
        .await?;
    Ok(count > 0)
}

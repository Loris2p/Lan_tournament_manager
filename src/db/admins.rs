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
        .count_documents(doc! {}, None)
        .await?;
    Ok(count)
}

pub async fn is_setup_required(db: &Database) -> Result<bool, Errors> {
    if count_admins(db).await? > 0 {
        return Ok(false);
    }
    let has_lock = db
        .collection::<mongodb::bson::Document>("system_config")
        .find_one(doc! { "_id": "setup_done" }, None)
        .await?
        .is_some();
    Ok(!has_lock)
}

pub async fn create_initial_admin(
    db: &Database,
    username: &str,
    password_hash: &str,
) -> Result<Admin, Errors> {
    if !is_setup_required(db).await? {
        return Err(Errors::Error403Forbidden(
            "L'initialisation a déjà été effectuée".to_string(),
        ));
    }

    let lock_res = db
        .collection::<mongodb::bson::Document>("system_config")
        .insert_one(doc! { "_id": "setup_done" }, None)
        .await;

    if lock_res.is_err() {
        return Err(Errors::Error403Forbidden(
            "L'initialisation a déjà été effectuée ou est en cours".to_string(),
        ));
    }

    match create_admin(db, username, password_hash).await {
        Ok(admin) => Ok(admin),
        Err(e) => {
            let _ = db
                .collection::<mongodb::bson::Document>("system_config")
                .delete_one(doc! { "_id": "setup_done" }, None)
                .await;
            Err(e)
        }
    }
}
pub async fn find_admin_by_username(
    db: &Database,
    username: &str,
) -> Result<Admin, Errors> {
    let username_norm = username.trim().to_lowercase();
    let admin = db
        .collection::<Admin>("admins")
        .find_one(doc! {"username": username_norm}, None)
        .await?;
    match admin {
        Some(s) => Ok(s),
        None => Err(Errors::Error404NotFound("Admin introuvable".to_string())),
    }

}
pub async fn list_admins(db: &Database) -> Result<Vec<Admin>, Errors> {
    let admins = db
        .collection::<Admin>("admins")
        .find(doc! {}, None)
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
        username: username.trim().to_lowercase(),
        password_hash: password_hash.to_string(),
        token_version: 0,
    };
    let res = db
        .collection::<Admin>("admins")
        .insert_one(&new_admin, None)
        .await?;
    new_admin.id = res.inserted_id.as_object_id();

    let update_opts = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let _ = db
        .collection::<mongodb::bson::Document>("system_config")
        .update_one(
            doc! { "_id": "admin_count" },
            doc! { "$inc": { "count": 1 } },
            update_opts,
        )
        .await;

    Ok(new_admin)
}
pub async fn delete_admin(db: &Database, admin_id: ObjectId) -> Result<(), Errors> {
    let count = count_admins(db).await?;
    if count <= 1 {
        return Err(Errors::Error400BadRequest(
            "Impossible de supprimer le dernier administrateur !".to_string(),
        ));
    }

    // Décrément atomique avec condition count > 1
    let dec_res = db
        .collection::<mongodb::bson::Document>("system_config")
        .find_one_and_update(
            doc! { "_id": "admin_count", "count": { "$gt": 1 } },
            doc! { "$inc": { "count": -1 } },
            None,
        )
        .await?;

    if dec_res.is_none() {
        return Err(Errors::Error400BadRequest(
            "Impossible de supprimer le dernier administrateur !".to_string(),
        ));
    }

    let res = db
        .collection::<Admin>("admins")
        .delete_one(doc! {"_id":admin_id}, None)
        .await?;

    if res.deleted_count == 0 {
        // Restaurer le compteur si aucun document n'a été supprimé
        let _ = db
            .collection::<mongodb::bson::Document>("system_config")
            .update_one(
                doc! { "_id": "admin_count" },
                doc! { "$inc": { "count": 1 } },
                None,
            )
            .await;
        return Err(Errors::Error404NotFound("Admin introuvable".to_string()));
    }
    Ok(())
}

pub async fn increment_admin_token_version(db: &Database, admin_id: ObjectId) -> Result<(), Errors> {
    db.collection::<Admin>("admins")
        .update_one(doc! {"_id": admin_id}, doc! {"$inc": {"token_version": 1}}, None)
        .await?;
    Ok(())
}

pub async fn verify_admin_session(db: &Database, admin_id: ObjectId, token_version: u32) -> Result<bool, Errors> {
    let count = db
        .collection::<Admin>("admins")
        .count_documents(doc! {"_id": admin_id, "token_version": token_version as i64}, None)
        .await?;
    Ok(count > 0)
}

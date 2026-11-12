use crate::errors::Errors;
use crate::models::*;
use mongodb::{
    Database, IndexModel,
    bson::{doc},
    options::IndexOptions,
};

pub async fn init_indexes(db: &Database) -> Result<(), Errors> {
    let unique_option = IndexOptions::builder().unique(true).build();

    let admin_index = IndexModel::builder()
        .keys(doc! {"username":1})
        .options(unique_option.clone())
        .build();
    db.collection::<Admin>("admins")
        .create_index(admin_index, None)
        .await?;

    let submissions = IndexModel::builder()
        .keys(doc! {"match_id":1, "team_id":1})
        .options(unique_option.clone())
        .build();
    db.collection::<Submission>("submissions")
        .create_index(submissions, None)
        .await?;

    let teams_captain = IndexModel::builder()
        .keys(doc! { "captain_username": 1 })
        .options(unique_option.clone())
        .build();
    let teams_name = IndexModel::builder()
        .keys(doc! { "name": 1 })
        .options(unique_option)
        .build();

    db.collection::<Team>("teams")
        .create_indexes([teams_captain, teams_name], None)
        .await?;

    let count = crate::db::count_admins(db).await.unwrap_or(0);
    let update_options = mongodb::options::UpdateOptions::builder().upsert(true).build();
    let _ = db
        .collection::<mongodb::bson::Document>("system_config")
        .update_one(
            doc! { "_id": "admin_count" },
            doc! { "$set": { "count": count as i64 } },
            update_options,
        )
        .await;

    if count > 0 {
        let _ = db
            .collection::<mongodb::bson::Document>("system_config")
            .insert_one(doc! { "_id": "setup_done" }, None)
            .await;
    }

    Ok(())
}

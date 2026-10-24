use crate::errors::Errors;
use crate::models::*;
use futures::TryStreamExt; // Indispensable pour .try_collect() sur les curseurs !
use mongodb::{
    Database, IndexModel,
    bson::{doc, oid::ObjectId},
    options::IndexOptions,
};

pub async fn init_indexes(db: &Database) -> Result<(), Errors> {
    let unique_option = IndexOptions::builder().unique(true).build();
    //
    let admin_index = IndexModel::builder()
        .keys(doc! {"username":1})
        .options(unique_option.clone())
        .build();
    db.collection::<Admin>("admins")
        .create_index(admin_index)
        .await?;
    //
    let submissions = IndexModel::builder()
        .keys(doc! {"match_id":1, "team_id":1})
        .options(unique_option.clone())
        .build();
    db.collection::<Submission>("submissions")
        .create_index(submissions)
        .await?;
    //
    let teams_captain = IndexModel::builder()
        .keys(doc! { "captain_username": 1 })
        .options(unique_option.clone())
        .build();
    let teams_name = IndexModel::builder()
        .keys(doc! { "name": 1 })
        .options(unique_option)
        .build();

    db.collection::<Team>("teams")
        .create_indexes([teams_captain, teams_name])
        .await?;

    Ok(())
}

use crate::errors::Errors;
use crate::models::{Match, Submission, SubmissionScore, SubmissionStatus};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
use std::ops::Sub;
use serde_json::Value::Null;

pub async fn get_open_matches(db: &Database) -> Result<Vec<Match>, Errors> {
    let matches = db
        .collection::<Match>("matches")
        .find(doc! {"is_open":true})
        .await?
        .try_collect()
        .await?;
    Ok(matches)
}
pub async fn get_all_matches(db: &Database) -> Result<Vec<Match>, Errors> {
    let matches = db
        .collection::<Match>("matches")
        .find(doc! {})
        .await?
        .try_collect()
        .await?;
    Ok(matches)
}
pub async fn find_match_by_id(db: &Database, match_id: ObjectId) -> Result<Match, Errors> {
    let found_match = db
        .collection::<Match>("matches")
        .find_one(doc! {"_id":match_id})
        .await?;
    match found_match {
        Some(m) => Ok(m),
        None => Err(Errors::Error404NotFound("Match introuvable".to_string())),
    }
}
pub async fn create_match(
    db: &Database,
    title: &str,
    game_id: ObjectId,
    game_name: &str,
    is_open: bool,
) -> Result<Match, Errors> {
    let mut m = Match {
        id: None,
        title: title.to_string(),
        game_id,
        game_name: game_name.to_string(),
        is_open,
    };
    let res = db.collection::<Match>("matches").insert_one(&m).await?;
    m.id = res.inserted_id.as_object_id();
    Ok(m)
}

pub async fn toggle_match_status(db: &Database, match_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Match>("matches")
        .update_one(
            doc! { "_id": match_id },
            vec![doc! {"$set": {"is_open": {"$not": ["$is_open"]}}}],
        )
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Match introuvable".to_string()));
    }
    Ok(())
}

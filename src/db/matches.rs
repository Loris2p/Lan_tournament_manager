use crate::errors::Errors;
use crate::models::{Match, Submission};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};


pub async fn get_open_matches(db: &Database) -> Result<Vec<Match>, Errors> {
    let matches = db
        .collection::<Match>("matches")
        .find(doc! {"is_open":true}, None)
        .await?
        .try_collect()
        .await?;
    Ok(matches)
}
pub async fn get_all_matches(db: &Database) -> Result<Vec<Match>, Errors> {
    let matches = db
        .collection::<Match>("matches")
        .find(doc! {}, None)
        .await?
        .try_collect()
        .await?;
    Ok(matches)
}
pub async fn find_match_by_id(db: &Database, match_id: ObjectId) -> Result<Match, Errors> {
    let found_match = db
        .collection::<Match>("matches")
        .find_one(doc! {"_id":match_id}, None)
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
    assigned_teams: Vec<ObjectId>,
    instructions: &str,
    is_open: bool,
) -> Result<Match, Errors> {
    let mut m = Match {
        id: None,
        title: title.to_string(),
        game_id,
        game_name: game_name.to_string(),
        is_open,
        instructions: instructions.to_string(),
        assigned_teams,
    };
    let res = db.collection::<Match>("matches").insert_one(&m, None).await?;
    m.id = res.inserted_id.as_object_id();
    Ok(m)
}

pub async fn toggle_match_status(db: &Database, match_id: ObjectId) -> Result<(), Errors> {
    let m = find_match_by_id(db, match_id).await?;
    let new_status = !m.is_open;
    let res = db
        .collection::<Match>("matches")
        .update_one(
            doc! { "_id": match_id },
            doc! { "$set": { "is_open": new_status } },
            None,
        )
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Match introuvable".to_string()));
    }
    Ok(())
}

pub async fn delete_match(db: &Database, match_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Match>("matches")
        .delete_one(doc! { "_id": match_id }, None)
        .await?;
    if res.deleted_count == 0 {
        return Err(Errors::Error404NotFound("Match introuvable".to_string()));
    }
    Ok(())
}
pub async fn get_open_matches_for_team(
    db: &Database,
    team_id: ObjectId,
) -> Result<Vec<Match>, Errors> {
    let submitted_match_ids = db
        .collection::<Submission>("submissions")
        .distinct("match_id", doc! { "team_id": team_id }, None)
        .await?;

    let res = db
        .collection::<Match>("matches")
        .find(
            doc! {
                "is_open": true,
                "assigned_teams": team_id,
                "_id": { "$nin": submitted_match_ids }
            },
            None,
        )
        .await?
        .try_collect()
        .await?;
    Ok(res)
}
pub async fn update_match_instructions(
    db: &Database,
    match_id: ObjectId,
    instructions: &str,
) -> Result<(), Errors> {
    let res = db
        .collection::<Match>("matches")
        .update_one(
            doc! {"_id":match_id},
            doc! {"$set":{"instructions":instructions}},
            None,
        )
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Match introuvable".to_string()));
    }
    Ok(())
}

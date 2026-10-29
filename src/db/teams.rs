use crate::errors::Errors;
use crate::models::{ PublicTeam, Team};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};
pub async fn find_team_by_captain_username(
    db: &Database,
    captain_username: &str,
) -> Result<Team, Errors> {
    let team = db
        .collection::<Team>("teams")
        .find_one(doc! {"captain_username":captain_username})
        .await?;
    match team {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound(
            "Équipe du captaine introuvable".to_string(),
        )),
    }
}
pub async fn find_team_by_id(db: &Database, team_id: ObjectId) -> Result<Team, Errors> {
    let team = db
        .collection::<Team>("teams")
        .find_one(doc! {"_id":team_id})
        .await?;
    match team {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound("Équipe introuvable".to_string())),
    }
}

pub async fn get_leaderboard(db: &Database) -> Result<Vec<PublicTeam>, Errors> {
    let sorted_teams: Vec<Team> = db
        .collection::<Team>("teams")
        .find(doc! {})
        .sort(doc! {"total_points":-1})
        .await?
        .try_collect()
        .await?;
    let mut leaderboard = Vec::new();
    for t in sorted_teams {
        let id = match t.id {
            Some(oid) => oid.to_hex(),
            None => {
                return Err(Errors::Error500InternalError(
                    "Équipe sans ID en base".to_string(),
                ));
            }
        };
        leaderboard.push(PublicTeam {
            id,
            name: t.name,
            captain_username: t.captain_username,
            total_points: t.total_points,
        });
    }
    Ok(leaderboard)
}

pub async fn create_team(
    db: &Database,
    name: &str,
    captain_username: &str,
    captain_password_hash: &str,
) -> Result<Team, Errors> {
    let mut equipe = Team {
        id: None,
        name: name.to_string(),
        captain_username: captain_username.to_string(),
        captain_password_hash: captain_password_hash.to_string(),
        total_points: 0,
    };
    let res = db.collection::<Team>("teams").insert_one(&equipe).await?;
    equipe.id = res.inserted_id.as_object_id();

    Ok(equipe)
}
pub async fn delete_team(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .delete_one(doc! {"_id":team_id})
        .await?;
    if res.deleted_count == 0 {
        return Err(Errors::Error404NotFound("Team introuvable".to_string()));
    }
    Ok(())
}
pub async fn add_points_to_team(
    db: &Database,
    team_id: ObjectId,
    points: u32,
) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .update_one(doc! {"_id":team_id}, doc! {"$inc":{"total_points":points}})
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Équipe introuvable".to_string()));
    }
    Ok(())
}
pub async fn adjust_team_points(
    db: &Database,
    team_id: ObjectId,
    adjustment: i32,
) -> Result<u32, Errors> {
    let team = find_team_by_id(db, team_id).await?;
    let new_points = if adjustment < 0 {
        team.total_points.saturating_sub(adjustment.unsigned_abs())
    } else {
        team.total_points.saturating_add(adjustment as u32)
    };
    set_points_to_team(db, team_id, new_points).await?;
    Ok(new_points)
}

pub async fn set_points_to_team(
    db: &Database,
    team_id: ObjectId,
    points: u32,
) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .update_one(doc! {"_id":team_id}, doc! {"$set":{"total_points":points}})
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Équipe introuvable".to_string()));
    }
    Ok(())
}

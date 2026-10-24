use crate::errors::Errors;
use crate::models::{ Game, GameFormat};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};

pub async fn list_games(db: &Database) -> Result<Vec<Game>, Errors> {
    let games = db
        .collection::<Game>("games")
        .find(doc! {})
        .await?
        .try_collect()
        .await?;
    Ok(games)
}
pub async fn find_game_by_id(db: &Database, game_id: ObjectId) -> Result<Game, Errors> {
    let game = db
        .collection::<Game>("games")
        .find_one(doc! {"_id":game_id})
        .await?;
    match game {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound("Jeu introuvable".to_string())),
    }
}
pub async fn create_game(
    db: &Database,
    name: &str,
    cover_image: &str,
    format: GameFormat,
    placement_points: Vec<u32>,
    default_placement_points: u32,
    secondary_stat_label: Option<String>,
    points_per_secondary_stat: u32,
    max_placement: u8,
    max_secondary_stat :u32
) -> Result<Game, Errors> {
    let mut new_game = Game {
        id: None,
        name: name.to_string(),
        cover_image: cover_image.to_string(),
        format,
        placement_points,
        default_placement_points,
        secondary_stat_label,
        points_per_secondary_stat,
        max_placement,
        max_secondary_stat,
    };
    let res = db.collection::<Game>("games").insert_one(&new_game).await?;
    new_game.id = res.inserted_id.as_object_id();
    Ok(new_game)
}
pub async fn delete_game(db : &Database, game_id : ObjectId) -> Result<(), Errors>{
    let res = db
        .collection::<Game>("games")
        .delete_one(doc! {"_id":game_id})
        .await?;
    if res.deleted_count == 0 {
        return Err(Errors::Error404NotFound("Jeu introuvable".to_string()));
    }
    Ok(())
}

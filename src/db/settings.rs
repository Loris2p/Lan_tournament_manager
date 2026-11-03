use crate::errors::Errors;
use crate::models::TournamentSettings;
use mongodb::Database;
use mongodb::bson::{doc};


pub async fn get_tournament_settings(db: &Database) -> Result<TournamentSettings, Errors> {
    match db
        .collection::<TournamentSettings>("tournament_settings")
        .find_one(doc! {})
        .await?
    {
        Some(s) => return Ok(s),
        None => {
            let mut tournament = TournamentSettings::default();
            let res = db
                .collection::<TournamentSettings>("tournament_settings")
                .insert_one(&tournament)
                .await?;
            tournament.id = res.inserted_id.as_object_id();
            Ok(tournament)
        }
    }
}

pub async fn update_tournament_settings(db: &Database, points: Vec<u32>) -> Result<(), Errors> {
    get_tournament_settings(db).await?;

    let bson_points = mongodb::bson::to_bson(&points)
        .map_err(|e| Errors::Error500InternalError(e.to_string()))?;

    let res = db
        .collection::<TournamentSettings>("tournament_settings")
        .update_one(
            doc! {},
            doc! { "$set": { "global_placement_points": bson_points } },
        )
        .await?;

    if res.matched_count == 0 {
        return Err(Errors::Error500InternalError(
            "Impossible de mettre à jour la liste des points".to_string(),
        ));
    }
    Ok(())
}

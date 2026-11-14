use crate::errors::Errors;
use crate::models::{TournamentSettings, UpdateTournamentSettingsRequest};
use mongodb::Database;
use mongodb::bson::{doc, to_bson};


pub async fn get_tournament_settings(db: &Database) -> Result<TournamentSettings, Errors> {
    match db
        .collection::<TournamentSettings>("tournament_settings")
        .find_one(doc! {}, None)
        .await?
    {
        Some(s) => Ok(s),
        None => {
            let mut tournament = TournamentSettings::default();
            let res = db
                .collection::<TournamentSettings>("tournament_settings")
                .insert_one(&tournament, None)
                .await?;
            tournament.id = res.inserted_id.as_object_id();
            Ok(tournament)
        }
    }
}

pub async fn update_tournament_settings(
    db: &Database,
    req: &UpdateTournamentSettingsRequest,
) -> Result<(), Errors> {
    get_tournament_settings(db).await?;

    let mut set_doc = doc! {};
    if let Some(ref points) = req.global_placement_points {
        let bson_points = to_bson(points)
            .map_err(|e| Errors::Error500InternalError(e.to_string()))?;
        set_doc.insert("global_placement_points", bson_points);
    }
    if let Some(ref start) = req.start_at {
        set_doc.insert("start_at", start);
    }
    if let Some(ref end) = req.end_at {
        set_doc.insert("end_at", end);
    }
    if let Some(paused) = req.is_paused {
        set_doc.insert("is_paused", paused);
    }
    if let Some(enable_general) = req.enable_general_leaderboard {
        set_doc.insert("enable_general_leaderboard", enable_general);
    }

    if set_doc.is_empty() {
        return Ok(());
    }

    let res = db
        .collection::<TournamentSettings>("tournament_settings")
        .update_one(
            doc! {},
            doc! { "$set": set_doc },
            None,
        )
        .await?;

    if res.matched_count == 0 {
        return Err(Errors::Error500InternalError(
            "Impossible de mettre à jour les paramètres".to_string(),
        ));
    }
    Ok(())
}

use crate::errors::Errors;
use crate::models::{Match, ResetScope, Submission, Team};

use mongodb::{
    Database,
    bson::{doc},
};


pub async fn reset_tournament(db: &Database, scope: ResetScope) -> Result<(), Errors> {
    // Suppression de toutes les soumissions
    db.collection::<Submission>("submissions")
        .delete_many(doc! {})
        .await?;
    if scope == ResetScope::ScoresOnly {
        //suppression des scores des équipes (reset à 0)
        db.collection::<Team>("teams")
            .update_many(doc! {}, doc! {"$set":{"total_points":0}})
            .await?;
    } else {
        //Suppression de toutes les équipes
        db.collection::<Team>("teams").delete_many(doc! {}).await?;
        if scope != ResetScope::UsersAndTeams {
            db.collection::<Match>("matches")
                .delete_many(doc! {})
                .await?;
        }
    }
    Ok(())
}

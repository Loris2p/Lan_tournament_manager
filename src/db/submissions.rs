use crate::errors::Errors;
use crate::models::{Submission, SubmissionScore, SubmissionStatus};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};

pub async fn insert_submission(
    db: &Database,
    match_id: ObjectId,
    team_id: ObjectId,
    team_name: String,
    score: SubmissionScore,
    proof_base64: Option<String>,
    calculated_points: u32,
    status: SubmissionStatus,
) -> Result<Submission, Errors> {
    let mut s = Submission {
        id: None,
        match_id,
        team_id,
        team_name: team_name.to_string(),
        score,
        proof_base64,
        calculated_points,
        status,
    };
    let res = db
        .collection::<Submission>("submissions")
        .insert_one(&s)
        .await?;
    s.id = res.inserted_id.as_object_id();
    Ok(s)
}

pub async fn get_pending(db: &Database) -> Result<Vec<Submission>, Errors> {
    let res = db
        .collection::<Submission>("submissions")
        .find(doc! {"status":SubmissionStatus::Pending})
        .await?
        .try_collect()
        .await?;
    Ok(res)
}
pub async fn approve_or_reject_submission(db: &Database, submission_id: ObjectId, new_status: SubmissionStatus) -> Result<(), Errors> {
    let res = db
        .collection::<Submission>("submissions")
        .update_one(
            doc! { "_id": submission_id },
            doc! {"$set": {"status": new_status}},
        )
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Soumission introuvable".to_string()));
    }
    Ok(())
}
pub async fn find_submission_by_id(db: &Database, submission_id: ObjectId)-> Result<Submission, Errors>{
    let found_submission = db
        .collection::<Submission>("submissions")
        .find_one(doc! {"_id":submission_id})
        .await?;
    match found_submission {
        Some(m) => Ok(m),
        None => Err(Errors::Error404NotFound("Submission introuvable".to_string())),
    }
}
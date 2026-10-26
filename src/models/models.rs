use crate::errors::Errors;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub struct AppState {
    pub db: mongodb::Database,
    pub jwt_secret: String,
    pub ws_tx: tokio::sync::broadcast::Sender<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameFormat {
    Duel,
    Brawl,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionScore {
    pub placement: u8,
    pub secondary_stat: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub cover_image: String,
    pub rules:String,
    pub format: GameFormat,
    pub placement_points: Vec<u32>,
    pub default_placement_points: u32,
    pub secondary_stat_label: Option<String>,
    pub points_per_secondary_stat: u32,
    pub max_placement: u8,
    pub max_secondary_stat: u32,
}
impl Game {
    pub fn validate_score(&self, score: &SubmissionScore) -> Result<(), Errors> {
        if score.placement < 1 || score.placement > self.max_placement {
            return Err(Errors::Error400BadRequest(
                "Score de placement illegal !".to_string(),
            ));
        }
        if score.secondary_stat > self.max_secondary_stat {
            return Err(Errors::Error400BadRequest(
                "Valeur de la stat secondaire illégale !".to_string(),
            ));
        }
        Ok(())
    }
    pub fn calculate_points(&self, score: &SubmissionScore) -> u32 {
        let pts_rang = match self.placement_points.get(usize::from(score.placement - 1)) {
            None => self.default_placement_points,
            Some(&n) => n,
        };
        let pts_secondary = score
            .secondary_stat
            .saturating_mul(self.points_per_secondary_stat);
        pts_rang + pts_secondary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Admin {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct PublicTeam {
    pub id: String,
    pub name: String,
    pub captain_username: String,
    pub total_points: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub captain_username: String,
    pub captain_password_hash: String,
    pub total_points: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub game_id: ObjectId,
    pub game_name: String,
    pub is_open: bool,
    pub assigned_teams : Vec<ObjectId>,
    pub instructions : String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubmissionStatus {
    Pending,
    Approved,
    Rejected,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub match_id: ObjectId,
    pub team_id: ObjectId,
    pub team_name: String,
    pub score: SubmissionScore,
    pub proof_base64: Option<String>,
    pub calculated_points: u32,
    pub status: SubmissionStatus,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetScope {
    ScoresOnly, // remet les total_points de toutes les équipes à 0 et supprime toutes les submissions (comptes gardés)
    UsersAndTeams, // ScoresOnly + supprime toutes les équipes et les capitaines
    FullTournament, // UsersAndTeams + matches
}
impl From<SubmissionStatus> for mongodb::bson::Bson {
    fn from(status: SubmissionStatus) -> Self {
        match status {
            SubmissionStatus::Pending => mongodb::bson::Bson::String("Pending".to_string()),
            SubmissionStatus::Approved => mongodb::bson::Bson::String("Approved".to_string()),
            SubmissionStatus::Rejected => mongodb::bson::Bson::String("Rejected".to_string()),
        }
    }
}

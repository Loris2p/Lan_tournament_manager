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
        if score.secondary_stat <= self.max_secondary_stat {
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

use chrono::SecondsFormat;
use futures::stream::ForEach;
use serde::{Deserialize, Serialize};
use mongodb::bson::oid::ObjectId;

pub struct AppState{
    pub db : mongodb::Database,
    pub jwt_secret : String,
    pub ws_tx : tokio::sync::broadcast::Sender<String>
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum GameFormat {
    Duel,    Brawl
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubmissionScore{
    pub placement : u8,
    pub secondary_stat : u32
}

#[derive(Debug, Clone, Serialize, Serialize)]
pub struct Game{
    #[serde(rename="_id", skip_serializing_if="Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub format : GameFormat,
    pub placement_points : Vec<u32>,
    pub default_placement_points:u32,
    pub secondary_stat_label:Option<String>,
    pub points_per_secondary_stat: u32,
    pub max_placement: u8,
    pub max_secondary_stat: u32,
}
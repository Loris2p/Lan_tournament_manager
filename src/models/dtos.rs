use crate::models::*;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
}
#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    pub captain_username: String,
    pub captain_password: String,
}

#[derive(Deserialize)]
pub struct CreateGameRequest {
    pub name: String,
    pub format: GameFormat,
    pub cover_image: String,
    pub rules : String,
    pub placement_points: Vec<u32>,
    pub default_placement_points: u32,
    pub secondary_stat_label: Option<String>,
    pub points_per_secondary_stat: u32,
    pub max_placement: u8,
    pub max_secondary_stat: u32,
}
#[derive(Deserialize)]
pub struct CreateMatchRequest {
    pub title: String,
    pub game_id: ObjectId,
    pub assigned_teams: Vec<ObjectId>,
    pub instructions: String,
    pub is_open: bool,
}

#[derive(Deserialize)]
pub struct CreateSubmissionRequest {
    pub match_id: ObjectId,
    pub score: SubmissionScore,
    pub proof_base64: String,
}
#[derive(Deserialize)]
pub struct AdminTeamScoreEntry {
    pub team_id: ObjectId,
    pub score: SubmissionScore,
}
#[derive(Deserialize)]
pub struct AdminBatchScoresRequest {
    pub results: Vec<AdminTeamScoreEntry>,
    pub close_match: bool,
}
#[derive(Deserialize)]
pub struct ResetTournamentRequest {
    pub scope: ResetScope,
}
#[derive(Deserialize)]
pub struct UpdateMatchInstructionsRequest {
    pub instructions: String,
}
#[derive(Deserialize)]
pub struct UpdateTeamPointsRequest {
    #[serde(alias = "points")]
    pub adjustment: i32,
}
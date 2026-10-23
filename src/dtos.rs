use crate::models::*;
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
    pub game_id: String,
}

#[derive(Deserialize)]
pub struct CreateSubmissionRequest {
    pub match_id: String,
    pub score: SubmissionScore,
    pub proof_base64: String,
}
#[derive(Deserialize)]
pub struct AdminTeamScoreEntry {
    pub team_id: String,
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

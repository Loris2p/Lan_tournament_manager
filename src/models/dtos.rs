use crate::models::*;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct MatchResponse {
    pub id: String,
    pub title: String,
    pub game_id: String,
    pub game_name: String,
    pub is_open: bool,
    pub assigned_teams: Vec<String>,
    pub instructions: String,
}

impl From<Match> for MatchResponse {
    fn from(m: Match) -> Self {
        Self {
            id: m.id.map(|id| id.to_hex()).unwrap_or_default(),
            title: m.title,
            game_id: m.game_id.to_hex(),
            game_name: m.game_name,
            is_open: m.is_open,
            assigned_teams: m.assigned_teams.into_iter().map(|id| id.to_hex()).collect(),
            instructions: m.instructions,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameResponse {
    pub id: String,
    pub name: String,
    pub cover_image: String,
    pub rules: String,
    pub format: GameFormat,
    pub placement_points: Vec<u32>,
    pub default_placement_points: u32,
    pub secondary_stat_label: Option<String>,
    pub points_per_secondary_stat: u32,
    pub max_placement: u8,
    pub max_secondary_stat: u32,
}

impl From<Game> for GameResponse {
    fn from(g: Game) -> Self {
        Self {
            id: g.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: g.name,
            cover_image: g.cover_image,
            rules: g.rules,
            format: g.format,
            placement_points: g.placement_points,
            default_placement_points: g.default_placement_points,
            secondary_stat_label: g.secondary_stat_label,
            points_per_secondary_stat: g.points_per_secondary_stat,
            max_placement: g.max_placement,
            max_secondary_stat: g.max_secondary_stat,
        }
    }
}
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
#[derive(Deserialize)]
pub struct UpdateTournamentSettingsRequest{
     pub global_placement_points:Vec<u32>
}
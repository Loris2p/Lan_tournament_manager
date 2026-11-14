use crate::db;
use crate::models::{AppState, GameLeaderboardView, Match, PublicTeam};
use crate::routes::pages::common::{wants_arena, HtmlTemplate, NavAuth};
use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

#[derive(Template)]
#[template(path = "leaderboard.html")]
pub struct LeaderboardTemplate {
    pub teams: Vec<PublicTeam>,
    pub live_matches: Vec<Match>,
    pub game_leaderboards: Vec<GameLeaderboardView>,
    pub auth: NavAuth,
    pub arena: bool,
    pub enable_general_leaderboard: bool,
}

impl LeaderboardTemplate {
    pub fn first(&self) -> Option<&PublicTeam> {
        self.teams.get(0)
    }
    pub fn second(&self) -> Option<&PublicTeam> {
        self.teams.get(1)
    }
    pub fn third(&self) -> Option<&PublicTeam> {
        self.teams.get(2)
    }
    pub fn remaining_teams(&self) -> &[PublicTeam] {
        self.teams.get(3..).unwrap_or(&[])
    }
}

pub async fn serve_leaderboard_page(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Response {
    let settings = db::get_tournament_settings(&state.db)
        .await
        .unwrap_or_default();
    let enable_general_leaderboard = settings.enable_general_leaderboard;
    let teams = if enable_general_leaderboard {
        db::get_leaderboard(&state.db).await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let live_matches = db::get_open_matches(&state.db).await.unwrap_or_default();
    let game_leaderboards = db::get_game_leaderboards(&state.db)
        .await
        .unwrap_or_default();
    let auth = NavAuth::from_jar(&jar, &state.jwt_secret);

    HtmlTemplate(LeaderboardTemplate {
        teams,
        live_matches,
        game_leaderboards,
        auth,
        arena: wants_arena(&jar),
        enable_general_leaderboard,
    })
    .into_response()
}

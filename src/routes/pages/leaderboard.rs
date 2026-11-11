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
    let teams = db::get_leaderboard(&state.db).await.unwrap_or_default();
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
    })
    .into_response()
}

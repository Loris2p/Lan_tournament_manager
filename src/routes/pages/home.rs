use crate::db;
use crate::models::{AppState, GameFormat};
use crate::routes::pages::common::{
    get_tournament_status_view, wants_arena, HtmlTemplate, NavAuth, TournamentStatusView,
};
use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use std::sync::Arc;

pub struct HomeGameView {
    pub name: String,
    pub format: String,
    pub cover_image: String,
    pub rules: String,
    pub placement_text: String,
    pub secondary_text: String,
    pub max_placement: u8,
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub games: Vec<HomeGameView>,
    pub auth: NavAuth,
    pub arena: bool,
    pub status: TournamentStatusView,
}

pub async fn serve_home_page(State(state): State<Arc<AppState>>, jar: CookieJar) -> Response {
    let games_db = db::list_games(&state.db).await.unwrap_or_default();
    let auth = NavAuth::from_jar(&jar, &state.jwt_secret);

    let games_view: Vec<HomeGameView> = games_db
        .into_iter()
        .map(|g| {
            let format = match g.format {
                GameFormat::Duel => "DUEL 1V1".to_string(),
                GameFormat::Brawl => "BRAWL // MULTI-ÉQUIPES".to_string(),
            };
            let placement_text = if !g.placement_points.is_empty() {
                g.placement_points
                    .iter()
                    .enumerate()
                    .take(4)
                    .map(|(i, pts)| format!("{}e: {}pts", i + 1, pts))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "Non configuré".to_string()
            };
            let secondary_text = if let Some(label) = g.secondary_stat_label {
                format!(
                    "{} ({} pt/unité, max {})",
                    label, g.points_per_secondary_stat, g.max_secondary_stat
                )
            } else {
                "Aucune statistique secondaire".to_string()
            };
            let rules = if !g.rules.trim().is_empty() {
                g.rules
            } else {
                "Consignes de match standard. Respecter le fair-play et les directives des arbitres.".to_string()
            };

            HomeGameView {
                name: g.name,
                format,
                cover_image: g.cover_image,
                rules,
                placement_text,
                secondary_text,
                max_placement: g.max_placement,
            }
        })
        .collect();

    let status_view = get_tournament_status_view(&state.db).await;

    HtmlTemplate(IndexTemplate {
        games: games_view,
        auth,
        arena: wants_arena(&jar),
        status: status_view,
    })
    .into_response()
}

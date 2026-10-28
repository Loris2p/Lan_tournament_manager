use crate::models::AppState;
use crate::routes::captain;
use crate::routes::public;
use crate::routes::{admin, auth};
use crate::ws;
use axum::Router;
use axum::routing::{delete, get, patch, post};
use std::sync::Arc;

pub fn create_router() -> Router<Arc<AppState>> {
    Router::new()
        .nest("/api/admin", create_admin_nest_router())
        .nest("/api/captain", create_captain_nest_routes())
        .nest("/api/auth", create_auth_nest_routes())
        .nest("/api", create_public_routes())
        .route("/ws", get(ws::ws_handler))
}

pub fn create_admin_nest_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/admins", get(admin::list_admins).post(admin::create_admin))
        .route("/admins/{id}", delete(admin::delete_admin))
        .route("/games", post(admin::create_game))
        .route("/games/{id}", delete(admin::delete_game))
        .route("/teams", post(admin::create_team))
        .route("/teams/{id}", delete(admin::delete_team))
        .route("/matches", post(admin::create_match))
        .route(
            "/matches/{id}/instructions",
            patch(admin::update_match_instructions),
        )
        .route("/matches/{id}/toggle", post(admin::toggle_match))
        .route("/matches/{id}/scores", post(admin::submit_match_scores))
        .route("/submissions", get(admin::list_pending_submissions))
        .route("/submissions/{id}/approve", post(admin::approve_submission))
        .route("/submissions/{id}/reject", post(admin::reject_submission))
        .route("/reset", post(admin::reset_tournament))
}
pub fn create_captain_nest_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/matches", get(captain::list_open_matches))
        .route("/submissions", post(captain::create_submission))
}
pub fn create_auth_nest_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/setup-status", get(auth::setup_status))
        .route("/setup", post(auth::setup_first_admin))
        .route("/login", post(auth::login))
        .route("/me", get(auth::get_me))
        .route("/logout", post(auth::logout))
}
pub fn create_public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/leaderboard", get(public::get_leaderboard))
        .route("/games", get(public::list_games))
        .route("/health", get(public::health_check))
}

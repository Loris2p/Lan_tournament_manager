use crate::auth::{hash_password, verify_jwt, AdminUser, UserRole};
use crate::db;
use crate::models::{
    AppState, GameFormat, PlayerSubmission, PublicTeam, ResetScope, SubmissionStatus,
};
use crate::routes::pages::common::HtmlTemplate;
use askama::Template;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum_extra::extract::{CookieJar, Form};
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub struct AdminSubmissionView {
    pub id: String,
    pub team_name: String,
    pub placement: u8,
    pub secondary_stat: u32,
    pub calculated_points: u32,
    pub proof_url: String,
    pub player_submissions: Vec<PlayerSubmission>,
}

pub struct AdminSubmissionGroup {
    pub match_title: String,
    pub game_name: String,
    pub submissions: Vec<AdminSubmissionView>,
}

pub struct AdminMatchGroup {
    pub game_name: String,
    pub matches: Vec<AdminMatchView>,
}

#[allow(dead_code)]
pub struct AdminGameView {
    pub id: String,
    pub name: String,
    pub format: String,
    pub rules: String,
    pub points_per_secondary_stat: u32,
}

pub struct AdminMatchView {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub is_open: bool,
    pub instructions: String,
    pub assigned_teams_count: usize,
}

pub struct AdminUserView {
    pub id: String,
    pub username: String,
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminTemplate {
    pub admin_username: String,
    pub submissions: Vec<AdminSubmissionView>,
    pub submission_groups: Vec<AdminSubmissionGroup>,
    pub games: Vec<AdminGameView>,
    pub matches: Vec<AdminMatchView>,
    pub match_groups: Vec<AdminMatchGroup>,
    pub teams: Vec<PublicTeam>,
    pub pending_teams: Vec<PublicTeam>,
    pub admins: Vec<AdminUserView>,
}

pub async fn serve_admin_page(State(state): State<Arc<AppState>>, jar: CookieJar) -> Response {
    let (admin_oid, admin_username) = match jar.get("lan_session") {
        Some(c) => match verify_jwt(c.value(), &state.jwt_secret) {
            Ok(claims) if matches!(claims.role, UserRole::Admin) => {
                match ObjectId::parse_str(&claims.sub) {
                    Ok(oid) => {
                        let valid = db::verify_admin_session(&state.db, oid, claims.token_version)
                            .await
                            .unwrap_or(false);
                        if valid {
                            (Some(oid), claims.username)
                        } else {
                            (None, String::new())
                        }
                    }
                    Err(_) => (None, String::new()),
                }
            }
            _ => (None, String::new()),
        },
        None => (None, String::new()),
    };

    if admin_oid.is_none() {
        let cookie = axum_extra::extract::cookie::Cookie::build("lan_session")
            .path("/")
            .build();
        let new_jar = jar.remove(cookie);
        return (new_jar, Redirect::to("/login")).into_response();
    }

    let submissions_db = db::get_pending(&state.db).await.unwrap_or_default();
    let games_db = db::list_games(&state.db).await.unwrap_or_default();
    let matches_db = db::get_all_matches(&state.db).await.unwrap_or_default();
    let teams_db = db::get_leaderboard(&state.db).await.unwrap_or_default();
    let admins_db = db::list_admins(&state.db).await.unwrap_or_default();
    let pending_teams_db = db::get_pending_teams(&state.db).await.unwrap_or_default();

    let match_meta: HashMap<ObjectId, (String, String)> = matches_db
        .iter()
        .filter_map(|m| m.id.map(|id| (id, (m.title.clone(), m.game_name.clone()))))
        .collect();

    let mut submission_groups: Vec<AdminSubmissionGroup> = Vec::new();
    let mut sub_group_idx: HashMap<ObjectId, usize> = HashMap::new();
    let mut submissions_view: Vec<AdminSubmissionView> = Vec::with_capacity(submissions_db.len());

    for s in submissions_db {
        let match_id = s.match_id;
        let sub_id = s.id.map(|id| id.to_hex()).unwrap_or_default();
        let proof_url = s.proof_base64.unwrap_or_default();
        let player_submissions = s.player_submissions.unwrap_or_default();

        submissions_view.push(AdminSubmissionView {
            id: sub_id.clone(),
            team_name: s.team_name.clone(),
            placement: s.score.placement,
            secondary_stat: s.score.secondary_stat,
            calculated_points: s.calculated_points,
            proof_url: proof_url.clone(),
            player_submissions: player_submissions.clone(),
        });

        let sub_for_group = AdminSubmissionView {
            id: sub_id,
            team_name: s.team_name,
            placement: s.score.placement,
            secondary_stat: s.score.secondary_stat,
            calculated_points: s.calculated_points,
            proof_url,
            player_submissions,
        };

        if let Some(&idx) = sub_group_idx.get(&match_id) {
            submission_groups[idx].submissions.push(sub_for_group);
        } else {
            let (match_title, game_name) = match_meta
                .get(&match_id)
                .cloned()
                .unwrap_or_else(|| ("Match supprimé".to_string(), "Inconnu".to_string()));
            let idx = submission_groups.len();
            sub_group_idx.insert(match_id, idx);
            submission_groups.push(AdminSubmissionGroup {
                match_title,
                game_name,
                submissions: vec![sub_for_group],
            });
        }
    }

    let games_view: Vec<AdminGameView> = games_db
        .into_iter()
        .map(|g| AdminGameView {
            id: g.id.map(|id| id.to_hex()).unwrap_or_default(),
            name: g.name,
            format: match g.format {
                GameFormat::Duel => "DUEL".to_string(),
                GameFormat::Brawl => "BRAWL".to_string(),
            },
            rules: g.rules,
            points_per_secondary_stat: g.points_per_secondary_stat,
        })
        .collect();

    let mut matches_view: Vec<AdminMatchView> = Vec::with_capacity(matches_db.len());
    let mut match_groups: Vec<AdminMatchGroup> = Vec::new();
    let mut match_group_idx: HashMap<String, usize> = HashMap::new();

    for m in matches_db {
        let id = m.id.map(|id| id.to_hex()).unwrap_or_default();
        let assigned_teams_count = m.assigned_teams.len();
        matches_view.push(AdminMatchView {
            id: id.clone(),
            title: m.title.clone(),
            game_name: m.game_name.clone(),
            is_open: m.is_open,
            instructions: m.instructions.clone(),
            assigned_teams_count,
        });

        let match_for_group = AdminMatchView {
            id,
            title: m.title,
            game_name: m.game_name.clone(),
            is_open: m.is_open,
            instructions: m.instructions,
            assigned_teams_count,
        };

        if let Some(&idx) = match_group_idx.get(&m.game_name) {
            match_groups[idx].matches.push(match_for_group);
        } else {
            let idx = match_groups.len();
            match_group_idx.insert(m.game_name.clone(), idx);
            match_groups.push(AdminMatchGroup {
                game_name: m.game_name,
                matches: vec![match_for_group],
            });
        }
    }

    let admins_view: Vec<AdminUserView> = admins_db
        .into_iter()
        .map(|a| AdminUserView {
            id: a.id.map(|id| id.to_hex()).unwrap_or_default(),
            username: a.username,
        })
        .collect();

    let pending_teams_view: Vec<PublicTeam> = pending_teams_db
        .into_iter()
        .filter_map(|t| {
            t.id.map(|id| PublicTeam {
                id: id.to_hex(),
                name: t.name,
                captain_username: t.captain_username,
                total_points: t.total_points,
            })
        })
        .collect();

    let template = AdminTemplate {
        admin_username,
        submissions: submissions_view,
        submission_groups,
        games: games_view,
        matches: matches_view,
        match_groups,
        teams: teams_db,
        pending_teams: pending_teams_view,
        admins: admins_view,
    };

    HtmlTemplate(template).into_response()
}

#[derive(Deserialize)]
pub struct EditSubmissionForm {
    pub placement: u8,
    #[serde(default)]
    pub secondary_stat: u32,
}

pub async fn admin_edit_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Form(form): Form<EditSubmissionForm>,
) -> Response {
    let submission = match db::find_submission_by_id(&state.db, id).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/admin?tab=arbitrage").into_response(),
    };
    if submission.status != SubmissionStatus::Pending {
        return Redirect::to("/admin?tab=arbitrage").into_response();
    }
    let m = match db::find_match_by_id(&state.db, submission.match_id).await {
        Ok(m) => m,
        Err(_) => return Redirect::to("/admin?tab=arbitrage").into_response(),
    };
    let game = match db::find_game_by_id(&state.db, m.game_id).await {
        Ok(g) => g,
        Err(_) => return Redirect::to("/admin?tab=arbitrage").into_response(),
    };

    let score = crate::models::SubmissionScore {
        placement: form.placement,
        secondary_stat: form.secondary_stat,
    };
    if game.validate_score(&score).is_err() {
        return Redirect::to("/admin?tab=arbitrage").into_response();
    }

    let calculated_points = game.calculate_points(&score);
    let _ = db::update_submission_score(&state.db, id, score, calculated_points).await;
    Redirect::to("/admin?tab=arbitrage").into_response()
}

pub async fn admin_approve_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let submission = match db::find_submission_by_id(&state.db, id).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/admin?tab=arbitrage").into_response(),
    };
    if submission.status != SubmissionStatus::Pending {
        return Redirect::to("/admin?tab=arbitrage").into_response();
    }
    if db::approve_or_reject_submission(&state.db, id, SubmissionStatus::Approved)
        .await
        .is_ok()
    {
        let _ = db::recalculate_tournament_scores(&state.db).await;
        let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=arbitrage").into_response()
}

pub async fn admin_reject_submission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let submission = match db::find_submission_by_id(&state.db, id).await {
        Ok(s) => s,
        Err(_) => return Redirect::to("/admin?tab=arbitrage").into_response(),
    };
    if submission.status != SubmissionStatus::Pending {
        return Redirect::to("/admin?tab=arbitrage").into_response();
    }
    let _ = db::approve_or_reject_submission(&state.db, id, SubmissionStatus::Rejected).await;
    Redirect::to("/admin?tab=arbitrage").into_response()
}

pub async fn admin_toggle_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    if let Ok(_) = db::toggle_match_status(&state.db, id).await {
        let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=matches").into_response()
}

pub async fn admin_delete_match(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    if let Ok(_) = db::delete_match(&state.db, id).await {
        let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=matches").into_response()
}

pub async fn admin_delete_game(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = db::delete_game(&state.db, id).await;
    Redirect::to("/admin?tab=games").into_response()
}

pub async fn admin_approve_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = db::approve_team(&state.db, id).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=teams").into_response()
}

pub async fn admin_delete_team(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
) -> Response {
    let _ = db::delete_team(&state.db, id).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=teams").into_response()
}

pub async fn admin_delete_admin(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    admin: AdminUser,
) -> Response {
    if admin.id == id {
        return Redirect::to("/admin?tab=admins&error=self_delete").into_response();
    }
    if db::delete_admin(&state.db, id).await.is_err() {
        return Redirect::to("/admin?tab=admins&error=cannot_delete_admin").into_response();
    }
    Redirect::to("/admin?tab=admins").into_response()
}

#[derive(Deserialize)]
pub struct AdjustPointsForm {
    pub adjustment: i32,
}

pub async fn admin_adjust_points(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Form(form): Form<AdjustPointsForm>,
) -> Response {
    let _ = db::adjust_team_points(&state.db, id, form.adjustment).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=teams").into_response()
}

#[derive(Deserialize)]
pub struct CreateGameForm {
    pub name: String,
    pub format: String,
    #[serde(default)]
    pub cover_image: String,
    pub placement_points: String,
    #[serde(default)]
    pub default_placement_points: u32,
    #[serde(default = "default_two")]
    pub max_placement: u8,
    pub secondary_stat_label: Option<String>,
    #[serde(default)]
    pub points_per_secondary_stat: u32,
    #[serde(default)]
    pub max_secondary_stat: u32,
    #[serde(default)]
    pub rules: String,
    #[serde(default)]
    pub team_player_count: Option<u8>,
}

fn default_two() -> u8 {
    2
}

pub async fn admin_create_game_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateGameForm>,
) -> Response {
    let format = match form.format.as_str() {
        "duel" => GameFormat::Duel,
        _ => GameFormat::Brawl,
    };
    let placement_points: Vec<u32> = form
        .placement_points
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();
    let sec_label = form.secondary_stat_label.filter(|s| !s.trim().is_empty());
    let team_player_count = form.team_player_count.filter(|&c| c > 1);

    let _ = db::create_game(
        &state.db,
        &form.name,
        &form.cover_image,
        format,
        placement_points,
        form.default_placement_points,
        sec_label,
        form.points_per_secondary_stat,
        form.max_placement,
        form.max_secondary_stat,
        &form.rules,
        team_player_count,
    )
    .await;

    Redirect::to("/admin?tab=games").into_response()
}

#[derive(Deserialize)]
pub struct CreateTeamForm {
    pub name: String,
    pub captain_username: String,
    pub captain_password: String,
}

pub async fn admin_create_team_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateTeamForm>,
) -> Response {
    if let Ok(hash) = hash_password(&form.captain_password) {
        let _ = db::create_team(&state.db, &form.name, &form.captain_username, &hash).await;
        let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    }
    Redirect::to("/admin?tab=teams").into_response()
}

#[derive(Deserialize)]
pub struct CreateAdminForm {
    pub username: String,
    pub password: String,
}

pub async fn admin_create_admin_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateAdminForm>,
) -> Response {
    if let Ok(hash) = hash_password(&form.password) {
        let _ = db::create_admin(&state.db, &form.username, &hash).await;
    }
    Redirect::to("/admin?tab=admins").into_response()
}

#[derive(Deserialize)]
pub struct CreateMatchForm {
    pub game_id: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub match_teams: Vec<String>,
}

pub async fn admin_create_match_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<CreateMatchForm>,
) -> Response {
    if let Ok(game_oid) = ObjectId::parse_str(&form.game_id) {
        if let Ok(game) = db::find_game_by_id(&state.db, game_oid).await {
            let teams_oids: Vec<ObjectId> = form
                .match_teams
                .iter()
                .filter_map(|t| ObjectId::parse_str(t).ok())
                .collect();
            let _ = db::create_match(
                &state.db,
                &game.name,
                game_oid,
                &game.name,
                teams_oids,
                &form.instructions,
                true,
            )
            .await;
            let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
        }
    }
    Redirect::to("/admin?tab=matches").into_response()
}

#[derive(Deserialize)]
pub struct UpdateInstructionsForm {
    pub instructions: String,
}

pub async fn admin_update_instructions_form(
    State(state): State<Arc<AppState>>,
    Path(id): Path<ObjectId>,
    _admin: AdminUser,
    Form(form): Form<UpdateInstructionsForm>,
) -> Response {
    let _ = db::update_match_instructions(&state.db, id, &form.instructions).await;
    let _ = state.ws_tx.send("MATCH_UPDATE".to_string());
    Redirect::to("/admin?tab=matches").into_response()
}

#[derive(Deserialize)]
pub struct ResetTournamentForm {
    pub scope: String,
}

pub async fn admin_reset_tournament_form(
    State(state): State<Arc<AppState>>,
    _admin: AdminUser,
    Form(form): Form<ResetTournamentForm>,
) -> Response {
    let scope = match form.scope.as_str() {
        "ScoresOnly" => ResetScope::ScoresOnly,
        "UsersAndTeams" => ResetScope::UsersAndTeams,
        _ => ResetScope::FullTournament,
    };
    let _ = db::reset_tournament(&state.db, &state.s3, scope).await;
    let _ = state.ws_tx.send("LEADERBOARD_UPDATE".to_string());
    Redirect::to("/admin?tab=danger").into_response()
}

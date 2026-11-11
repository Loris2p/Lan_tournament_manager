use crate::auth::CaptainUser;
use crate::db::{
    find_game_by_id, find_match_by_id, find_team_by_id, get_open_matches_for_team,
    insert_submission,
};
use crate::errors::Errors;
use crate::errors::Errors::{Error400BadRequest, Error403Forbidden};
use crate::models::{
    AppState, CreateSubmissionRequest, MatchResponse, PlayerSubmission, Submission,
    SubmissionStatus,
};
use axum::Json;
use axum::extract::State;

use crate::db;
use std::sync::Arc;

pub async fn list_open_matches(
    State(state): State<Arc<AppState>>,
    user: CaptainUser,
) -> Result<Json<Vec<MatchResponse>>, Errors> {
    let db = &state.db;
    let res = get_open_matches_for_team(db, user.team_id).await?;
    let response: Vec<MatchResponse> = res.into_iter().map(MatchResponse::from).collect();
    Ok(Json(response))
}

pub async fn create_submission(
    State(state): State<Arc<AppState>>,
    user: CaptainUser,
    Json(req): Json<CreateSubmissionRequest>,
) -> Result<Json<Submission>, Errors> {
    let db = &state.db;
    let team = find_team_by_id(db, user.team_id).await?;
    let m = find_match_by_id(db, req.match_id).await?;
    let game = find_game_by_id(db, m.game_id).await?;
    if !m.is_open {
        return Err(Error400BadRequest("Le match est fermé".to_string()));
    }
    if !m.assigned_teams.contains(&user.team_id) {
        return Err(Error403Forbidden(
            "Votre équipe n'est pas inscrite à ce match".to_string(),
        ));
    }
    let mut score = req.score;
    let (proof_url, player_submissions) = if let Some(player_inputs) = req.player_submissions {
        if !player_inputs.is_empty() {
            let mut uploaded_players = Vec::with_capacity(player_inputs.len());
            for p in &player_inputs {
                if !p.proof_base64.starts_with("data:image/") {
                    return Err(Error400BadRequest(
                        format!("La preuve du joueur {} doit être une image valide", p.player_number)
                    ));
                }
                let url = state.s3.upload_base_image(&p.proof_base64, "submissions").await?;
                uploaded_players.push(PlayerSubmission {
                    player_number: p.player_number,
                    score: p.score,
                    proof_url: url,
                });
            }
            if score.secondary_stat == 0 {
                score.secondary_stat = uploaded_players.iter().map(|p| p.score).sum();
            }
            let first_url = uploaded_players.first().map(|p| p.proof_url.clone());
            (first_url, Some(uploaded_players))
        } else {
            let single_proof = req.proof_base64.as_deref().unwrap_or("");
            if !single_proof.starts_with("data:image/") {
                return Err(Error400BadRequest("La preuve doit être une image".to_string()));
            }
            let url = state.s3.upload_base_image(single_proof, "submissions").await?;
            (Some(url), None)
        }
    } else {
        let single_proof = req.proof_base64.as_deref().unwrap_or("");
        if !single_proof.starts_with("data:image/") {
            return Err(Error400BadRequest("La preuve doit être une image".to_string()));
        }
        let url = state.s3.upload_base_image(single_proof, "submissions").await?;
        (Some(url), None)
    };

    game.validate_score(&score)?;
    let calculated_points = game.calculate_points(&score);
    if db::has_submission_for_match(db, req.match_id, user.team_id).await? {
        return Err(Errors::Error409Conflict(
            "Votre équipe a déjà soumis un score pour ce match".to_string(),
        ));
    };

    let res = insert_submission(
        db,
        req.match_id,
        user.team_id,
        team.name,
        score,
        proof_url,
        calculated_points,
        SubmissionStatus::Pending,
        player_submissions,
    )
    .await?;
    let _ = state.ws_tx.send("NEW_SUBMISSION".to_string());
    Ok(Json(res))
}

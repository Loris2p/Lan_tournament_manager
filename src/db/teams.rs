use std::collections::HashMap;

use crate::db;
use crate::db::get_tournament_settings;
use crate::errors::Errors;
use crate::models::{GameLeaderboardEntry, GameLeaderboardView, PublicTeam, Submission, Team};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
};

pub async fn find_team_by_captain_username(
    db: &Database,
    captain_username: &str,
) -> Result<Team, Errors> {
    let team = db
        .collection::<Team>("teams")
        .find_one(doc! {"captain_username":captain_username})
        .await?;
    match team {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound(
            "Équipe du captaine introuvable".to_string(),
        )),
    }
}
pub async fn find_team_by_id(db: &Database, team_id: ObjectId) -> Result<Team, Errors> {
    let team = db
        .collection::<Team>("teams")
        .find_one(doc! {"_id":team_id})
        .await?;
    match team {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound("Équipe introuvable".to_string())),
    }
}

pub async fn get_leaderboard(db: &Database) -> Result<Vec<PublicTeam>, Errors> {
    let sorted_teams: Vec<Team> = db
        .collection::<Team>("teams")
        .find(doc! {})
        .sort(doc! {"total_points":-1})
        .await?
        .try_collect()
        .await?;
    let mut leaderboard = Vec::new();
    for t in sorted_teams {
        let id = match t.id {
            Some(oid) => oid.to_hex(),
            None => {
                return Err(Errors::Error500InternalError(
                    "Équipe sans ID en base".to_string(),
                ));
            }
        };
        leaderboard.push(PublicTeam {
            id,
            name: t.name,
            captain_username: t.captain_username,
            total_points: t.total_points,
        });
    }
    Ok(leaderboard)
}

pub async fn create_team(
    db: &Database,
    name: &str,
    captain_username: &str,
    captain_password_hash: &str,
) -> Result<Team, Errors> {
    let mut equipe = Team {
        id: None,
        name: name.to_string(),
        captain_username: captain_username.to_string(),
        captain_password_hash: captain_password_hash.to_string(),
        total_points: 0,
        token_version: 0,
    };
    let res = db.collection::<Team>("teams").insert_one(&equipe).await?;
    equipe.id = res.inserted_id.as_object_id();

    Ok(equipe)
}
pub async fn delete_team(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .delete_one(doc! {"_id":team_id})
        .await?;
    if res.deleted_count == 0 {
        return Err(Errors::Error404NotFound("Team introuvable".to_string()));
    }
    Ok(())
}

pub async fn adjust_team_points(
    db: &Database,
    team_id: ObjectId,
    adjustment: i32,
) -> Result<u32, Errors> {
    let team = find_team_by_id(db, team_id).await?;
    let new_points = if adjustment < 0 {
        team.total_points.saturating_sub(adjustment.unsigned_abs())
    } else {
        team.total_points.saturating_add(adjustment as u32)
    };
    set_points_to_team(db, team_id, new_points).await?;
    Ok(new_points)
}

pub async fn set_points_to_team(
    db: &Database,
    team_id: ObjectId,
    points: u32,
) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .update_one(doc! {"_id":team_id}, doc! {"$set":{"total_points":points}})
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Équipe introuvable".to_string()));
    }
    Ok(())
}

pub async fn increment_team_token_version(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    db.collection::<Team>("teams")
        .update_one(doc! {"_id": team_id}, doc! {"$inc": {"token_version": 1}})
        .await?;
    Ok(())
}

pub async fn verify_team_session(
    db: &Database,
    team_id: ObjectId,
    token_version: u32,
) -> Result<bool, Errors> {
    let count = db
        .collection::<Team>("teams")
        .count_documents(doc! {"_id": team_id, "token_version": token_version as i64})
        .await?;
    Ok(count > 0)
}
pub async fn recalculate_tournament_scores(db: &Database) -> Result<(), Errors> {
    let settings = get_tournament_settings(db).await?;
    let teams = db
        .collection::<Team>("teams")
        .find(doc! {})
        .await?
        .try_collect::<Vec<Team>>()
        .await?;
    let matches = db::get_all_matches(db).await?;
    let submissions = db
        .collection::<Submission>("submissions")
        .find(doc! { "status": "Approved" })
        .await?
        .try_collect::<Vec<Submission>>()
        .await?;
    let mut match_to_game: HashMap<ObjectId, ObjectId> = HashMap::new();
    for m in matches {
        if let Some(match_id) = m.id {
            match_to_game.insert(match_id, m.game_id);
        }
    }
    let mut game_team_scores: HashMap<ObjectId, HashMap<ObjectId, u32>> = HashMap::new();
    for sub in submissions {
        if let Some(&game_id) = match_to_game.get(&sub.match_id) {
            let team_scores = game_team_scores.entry(game_id).or_default();
            let current = team_scores.entry(sub.team_id).or_insert(0);
            *current = current.saturating_add(sub.calculated_points);
        }
    }
    let mut tournament_points_per_team: HashMap<ObjectId, u32> = HashMap::new();
    for (_game_id, team_scores) in game_team_scores {
        let mut ranked_teams: Vec<(ObjectId, u32)> = team_scores.into_iter().collect();
        ranked_teams.sort_by(|a, b| b.1.cmp(&a.1));

        for (index, (team_id, _score)) in ranked_teams.into_iter().enumerate() {
            let rank = index + 1;
            let pts = settings.points_for_rank(rank);
            let total = tournament_points_per_team.entry(team_id).or_insert(0);
            *total = total.saturating_add(pts);
        }
    }
    for team in teams {
        if let Some(team_id) = team.id {
            let final_points = tournament_points_per_team
                .get(&team_id)
                .copied()
                .unwrap_or(0);
            set_points_to_team(db, team_id, final_points).await?;
        }
    }
    Ok(())
}
pub async fn get_game_leaderboards(db: &Database) -> Result<Vec<GameLeaderboardView>, Errors> {
    let settings = get_tournament_settings(db).await?;
    let games = db::list_games(db).await?;
    let teams = db
        .collection::<Team>("teams")
        .find(doc! {})
        .await?
        .try_collect::<Vec<Team>>()
        .await?;
    let matches = db::get_all_matches(db).await?;
    let submissions = db
        .collection::<Submission>("submissions")
        .find(doc! { "status": "Approved" })
        .await?
        .try_collect::<Vec<Submission>>()
        .await?;

    let team_names: HashMap<ObjectId, String> = teams
        .into_iter()
        .filter_map(|t| t.id.map(|id| (id, t.name)))
        .collect();

    let mut match_to_game: HashMap<ObjectId, ObjectId> = HashMap::new();
    for m in matches {
        if let Some(match_id) = m.id {
            match_to_game.insert(match_id, m.game_id);
        }
    }

    let mut game_team_scores: HashMap<ObjectId, HashMap<ObjectId, u32>> = HashMap::new();
    for sub in submissions {
        if let Some(&game_id) = match_to_game.get(&sub.match_id) {
            let team_scores = game_team_scores.entry(game_id).or_default();
            let current = team_scores.entry(sub.team_id).or_insert(0);
            *current = current.saturating_add(sub.calculated_points);
        }
    }

    let mut views = Vec::new();
    for game in games {
        let game_id = game.id.map(|id| id.to_hex()).unwrap_or_default();
        let game_name = game.name;
        let format = format!("{}", game.format);

        let mut entries = Vec::new();
        if let Some(game_oid) = game.id {
            if let Some(scores) = game_team_scores.get(&game_oid) {
                let mut ranked: Vec<(ObjectId, u32)> = scores
                    .iter()
                    .map(|(&t_id, &score)| (t_id, score))
                    .collect();
                ranked.sort_by(|a, b| b.1.cmp(&a.1));

                for (index, (team_id, raw_score)) in ranked.into_iter().enumerate() {
                    let rank = index + 1;
                    let tournament_points = settings.points_for_rank(rank);
                    let team_name = team_names
                        .get(&team_id)
                        .cloned()
                        .unwrap_or_else(|| "Équipe inconnue".to_string());

                    entries.push(GameLeaderboardEntry {
                        rank,
                        team_id: team_id.to_hex(),
                        team_name,
                        raw_score,
                        tournament_points,
                    });
                }
            }
        }

        views.push(GameLeaderboardView {
            game_id,
            game_name,
            format,
            entries,
        });
    }

    Ok(views)
}

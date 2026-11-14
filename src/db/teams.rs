use std::collections::HashMap;

use crate::db;
use crate::db::get_tournament_settings;
use crate::errors::Errors;
use crate::models::{Game, GameLeaderboardEntry, GameLeaderboardView, PublicTeam, Submission, Team};
use futures::TryStreamExt;
use mongodb::{
    Database,
    bson::{doc, oid::ObjectId},
    options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument},
};


pub async fn find_team_by_captain_username(
    db: &Database,
    captain_username:  &str,
) -> Result<Team, Errors> {
    let cap_norm = captain_username.trim().to_lowercase();
    let team = db
        .collection::<Team>("teams")
        .find_one(doc! {"captain_username": cap_norm}, None)
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
        .find_one(doc! {"_id":team_id}, None)
        .await?;
    match team {
        Some(g) => Ok(g),
        None => Err(Errors::Error404NotFound("Équipe introuvable".to_string())),
    }
}

pub async fn get_leaderboard(db: &Database) -> Result<Vec<PublicTeam>, Errors> {
    let opts = FindOptions::builder().sort(doc! {"total_points":-1}).build();
    let sorted_teams: Vec<Team> = db
        .collection::<Team>("teams")
        .find(doc! {"is_verified": {"$ne": false}}, opts)
        .await?
        .try_collect()
        .await?;
    let games = crate::db::list_games(db).await.unwrap_or_default();
    let games_map: HashMap<ObjectId, String> = games
        .into_iter()
        .filter_map(|g| g.id.map(|id| (id, g.name)))
        .collect();
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
        let game_name = t.game_id.and_then(|gid| games_map.get(&gid).cloned());
        leaderboard.push(PublicTeam {
            id,
            name: t.name,
            captain_username: t.captain_username,
            total_points: t.total_points,
            game_id: t.game_id.map(|gid| gid.to_hex()),
            game_name,
        });
    }
    Ok(leaderboard)
}

pub async fn get_pending_teams(db: &Database) -> Result<Vec<Team>, Errors> {
    let opts = FindOptions::builder().sort(doc! {"_id": -1}).build();
    let teams: Vec<Team> = db
        .collection::<Team>("teams")
        .find(doc! {"is_verified": false}, opts)
        .await?
        .try_collect()
        .await?;
    Ok(teams)
}

pub async fn approve_team(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .update_one(
            doc! {"_id": team_id},
            doc! {"$set": {"is_verified": true}},
            None,
        )
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Équipe introuvable".to_string()));
    }
    Ok(())
}

pub async fn create_team(
    db: &Database,
    name: &str,
    captain_username: &str,
    captain_password_hash: &str,
    game_id: Option<ObjectId>,
) -> Result<Team, Errors> {
    let mut equipe = Team {
        id: None,
        name: name.to_string(),
        captain_username: captain_username.trim().to_lowercase(),
        captain_password_hash: captain_password_hash.to_string(),
        total_points: 0,
        token_version: 0,
        is_verified: true,
        game_id,
    };
    let res = db.collection::<Team>("teams").insert_one(&equipe, None).await?;
    equipe.id = res.inserted_id.as_object_id();

    Ok(equipe)
}

pub async fn register_team(
    db: &Database,
    name: &str,
    captain_username: &str,
    captain_password_hash: &str,
    game_id: Option<ObjectId>,
) -> Result<Team, Errors> {
    let mut equipe = Team {
        id: None,
        name: name.to_string(),
        captain_username: captain_username.trim().to_lowercase(),
        captain_password_hash: captain_password_hash.to_string(),
        total_points: 0,
        token_version: 0,
        is_verified: false,
        game_id,
    };
    let res = db.collection::<Team>("teams").insert_one(&equipe, None).await?;
    equipe.id = res.inserted_id.as_object_id();

    Ok(equipe)
}
pub async fn delete_team(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .delete_one(doc! {"_id":team_id}, None)
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
    let coll = db.collection::<Team>("teams");
    let opts = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After)
        .build();
    if adjustment >= 0 {
        let updated = coll
            .find_one_and_update(
                doc! { "_id": team_id },
                doc! { "$inc": { "total_points": adjustment } },
                opts,
            )
            .await?;

        match updated {
            Some(team) => Ok(team.total_points),
            None => Err(Errors::Error404NotFound("Équipe introuvable".to_string())),
        }
    } else {
        let updated = coll
            .find_one_and_update(
                doc! { "_id": team_id, "total_points": { "$gte": -adjustment as i64 } },
                doc! { "$inc": { "total_points": adjustment } },
                opts.clone(),
            )
            .await?;

        if let Some(team) = updated {
            return Ok(team.total_points);
        }

        let reset = coll
            .find_one_and_update(
                doc! { "_id": team_id },
                doc! { "$set": { "total_points": 0 } },
                opts,
            )
            .await?;

        match reset {
            Some(team) => Ok(team.total_points),
            None => Err(Errors::Error404NotFound("Équipe introuvable".to_string())),
        }
    }
}

pub async fn set_points_to_team(
    db: &Database,
    team_id: ObjectId,
    points: u32,
) -> Result<(), Errors> {
    let res = db
        .collection::<Team>("teams")
        .update_one(doc! {"_id":team_id}, doc! {"$set":{"total_points":points}}, None)
        .await?;
    if res.matched_count == 0 {
        return Err(Errors::Error404NotFound("Équipe introuvable".to_string()));
    }
    Ok(())
}

pub async fn increment_team_token_version(db: &Database, team_id: ObjectId) -> Result<(), Errors> {
    db.collection::<Team>("teams")
        .update_one(doc! {"_id": team_id}, doc! {"$inc": {"token_version": 1}}, None)
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
        .count_documents(doc! {"_id": team_id, "token_version": token_version as i64}, None)
        .await?;
    Ok(count > 0)
}
pub async fn recalculate_tournament_scores(db: &Database) -> Result<(), Errors> {
    let settings = get_tournament_settings(db).await?;
    let teams = db
        .collection::<Team>("teams")
        .find(doc! {"is_verified": {"$ne": false}}, None)
        .await?
        .try_collect::<Vec<Team>>()
        .await?;
    let games = db::list_games(db).await?;
    let games_map: HashMap<ObjectId, Game> = games
        .into_iter()
        .filter_map(|g| g.id.map(|id| (id, g)))
        .collect();
    let matches = db::get_all_matches(db).await?;
    let submissions = db
        .collection::<Submission>("submissions")
        .find(doc! { "status": "Approved" }, None)
        .await?
        .try_collect::<Vec<Submission>>()
        .await?;

    let mut match_to_game: HashMap<ObjectId, ObjectId> = HashMap::new();
    for m in matches {
        if let Some(match_id) = m.id {
            match_to_game.insert(match_id, m.game_id);
        }
    }

    let mut match_submissions: HashMap<ObjectId, Vec<Submission>> = HashMap::new();
    for sub in submissions {
        match_submissions.entry(sub.match_id).or_default().push(sub);
    }

    // Pour les jeux à classement par manche (ex: GeoGuessr), ordonner les scores bruts
    for (match_id, subs) in &mut match_submissions {
        if let Some(&game_id) = match_to_game.get(match_id) {
            if let Some(game) = games_map.get(&game_id) {
                if game.is_score_ranked() {
                    subs.sort_by(|a, b| b.score.secondary_stat.cmp(&a.score.secondary_stat));
                    for (idx, sub) in subs.iter_mut().enumerate() {
                        let rank = (idx + 1) as u8;
                        let pts = game
                            .placement_points
                            .get(idx)
                            .copied()
                            .unwrap_or(game.default_placement_points);
                        if sub.calculated_points != pts || sub.score.placement != rank {
                            if let Some(sub_id) = sub.id {
                                let _ = db
                                    .collection::<Submission>("submissions")
                                    .update_one(
                                        doc! { "_id": sub_id },
                                        doc! { "$set": { "calculated_points": pts, "score.placement": rank as i32 } },
                                        None,
                                    )
                                    .await;
                            }
                            sub.calculated_points = pts;
                            sub.score.placement = rank;
                        }
                    }
                }
            }
        }
    }

    let mut game_team_scores: HashMap<ObjectId, HashMap<ObjectId, u32>> = HashMap::new();
    let mut game_team_tiebreak: HashMap<ObjectId, HashMap<ObjectId, u32>> = HashMap::new();
    for subs in match_submissions.values() {
        for sub in subs {
            if let Some(&game_id) = match_to_game.get(&sub.match_id) {
                let team_scores = game_team_scores.entry(game_id).or_default();
                let current = team_scores.entry(sub.team_id).or_insert(0);
                *current = current.saturating_add(sub.calculated_points);

                let tb_scores = game_team_tiebreak.entry(game_id).or_default();
                let current_tb = tb_scores.entry(sub.team_id).or_insert(0);
                *current_tb = current_tb.saturating_add(sub.score.secondary_stat);
            }
        }
    }

    let mut tournament_points_per_team: HashMap<ObjectId, u32> = HashMap::new();
    for (game_id, team_scores) in &game_team_scores {
        let tb_map = game_team_tiebreak.get(game_id);
        let mut ranked_teams: Vec<(ObjectId, u32)> = team_scores
            .iter()
            .map(|(&t_id, &score)| (t_id, score))
            .collect();
        ranked_teams.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| {
                let tb_a = tb_map.and_then(|m| m.get(&a.0)).copied().unwrap_or(0);
                let tb_b = tb_map.and_then(|m| m.get(&b.0)).copied().unwrap_or(0);
                tb_b.cmp(&tb_a)
            })
        });

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
        .find(doc! {"is_verified": {"$ne": false}}, None)
        .await?
        .try_collect::<Vec<Team>>()
        .await?;
    let matches = db::get_all_matches(db).await?;
    let submissions = db
        .collection::<Submission>("submissions")
        .find(doc! { "status": "Approved" }, None)
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
    let mut game_team_tiebreak: HashMap<ObjectId, HashMap<ObjectId, u32>> = HashMap::new();
    for sub in submissions {
        if let Some(&game_id) = match_to_game.get(&sub.match_id) {
            let team_scores = game_team_scores.entry(game_id).or_default();
            let current = team_scores.entry(sub.team_id).or_insert(0);
            *current = current.saturating_add(sub.calculated_points);

            let tb_scores = game_team_tiebreak.entry(game_id).or_default();
            let current_tb = tb_scores.entry(sub.team_id).or_insert(0);
            *current_tb = current_tb.saturating_add(sub.score.secondary_stat);
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
                let tb_map = game_team_tiebreak.get(&game_oid);
                let mut ranked: Vec<(ObjectId, u32)> = scores
                    .iter()
                    .map(|(&t_id, &score)| (t_id, score))
                    .collect();
                ranked.sort_by(|a, b| {
                    b.1.cmp(&a.1).then_with(|| {
                        let tb_a = tb_map.and_then(|m| m.get(&a.0)).copied().unwrap_or(0);
                        let tb_b = tb_map.and_then(|m| m.get(&b.0)).copied().unwrap_or(0);
                        tb_b.cmp(&tb_a)
                    })
                });

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

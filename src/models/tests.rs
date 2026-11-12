use crate::models::{Game, GameFormat, SubmissionScore, TournamentSettings};

#[test]
fn test_tournament_settings_points_for_rank() {
    let settings = TournamentSettings::default();

    assert_eq!(settings.points_for_rank(0), 0);
    assert_eq!(settings.points_for_rank(1), 25);
    assert_eq!(settings.points_for_rank(2), 18);
    assert_eq!(settings.points_for_rank(3), 15);
    assert_eq!(settings.points_for_rank(4), 12);
    assert_eq!(settings.points_for_rank(5), 10);
    assert_eq!(settings.points_for_rank(6), 8);
    assert_eq!(settings.points_for_rank(7), 6);
    assert_eq!(settings.points_for_rank(8), 4);
    assert_eq!(settings.points_for_rank(9), 2);
    assert_eq!(settings.points_for_rank(10), 1);
    assert_eq!(settings.points_for_rank(11), 0);
    assert_eq!(settings.points_for_rank(100), 0);
}

#[test]
fn test_game_calculate_points() {
    let game = Game {
        id: None,
        name: "Apex Legends".to_string(),
        cover_image: String::new(),
        format: GameFormat::Brawl,
        rules: "ALGS".to_string(),
        placement_points: vec![12, 9, 7, 5, 4, 3, 3, 2, 2, 2, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        default_placement_points: 0,
        points_per_secondary_stat: 1,
        secondary_stat_label: Some("Frags".to_string()),
        max_placement: 20,
        max_secondary_stat: 60,
        team_player_count: None,
    };

    // Top 1 avec 10 frags = 12 + 10 = 22
    let score1 = SubmissionScore { placement: 1, secondary_stat: 10 };
    assert_eq!(game.calculate_points(&score1), 22);

    // Top 2 avec 5 frags = 9 + 5 = 14
    let score2 = SubmissionScore { placement: 2, secondary_stat: 5 };
    assert_eq!(game.calculate_points(&score2), 14);

    // Top 6 sans frags = 3 + 0 = 3
    let score6 = SubmissionScore { placement: 6, secondary_stat: 0 };
    assert_eq!(game.calculate_points(&score6), 3);

    // Hors bornes du tableau placement_points (utilise default_placement_points: 0)
    let score_out = SubmissionScore { placement: 25, secondary_stat: 3 };
    assert_eq!(game.calculate_points(&score_out), 3);
}

#[test]
fn test_game_validate_score() {
    let game = Game {
        id: None,
        name: "Duel Test".to_string(),
        cover_image: String::new(),
        format: GameFormat::Duel,
        rules: "Duel Rules".to_string(),
        placement_points: vec![3, 1],
        default_placement_points: 0,
        points_per_secondary_stat: 0,
        secondary_stat_label: None,
        max_placement: 2,
        max_secondary_stat: 0,
        team_player_count: None,
    };

    let valid_win = SubmissionScore { placement: 1, secondary_stat: 0 };
    assert!(game.validate_score(&valid_win).is_ok());

    let invalid_placement = SubmissionScore { placement: 3, secondary_stat: 0 };
    assert!(game.validate_score(&invalid_placement).is_err());

    let invalid_stat = SubmissionScore { placement: 1, secondary_stat: 1 };
    assert!(game.validate_score(&invalid_stat).is_err());
}

#[test]
fn test_game_with_multi_players_serde() {
    use crate::models::PlayerSubmission;

    let game = Game {
        id: None,
        name: "Fall Guys".to_string(),
        cover_image: String::new(),
        format: GameFormat::Brawl,
        rules: "Solo 3 players".to_string(),
        placement_points: vec![3, 0],
        default_placement_points: 0,
        points_per_secondary_stat: 1,
        secondary_stat_label: Some("Manches".to_string()),
        max_placement: 2,
        max_secondary_stat: 15,
        team_player_count: Some(3),
    };

    assert_eq!(game.team_player_count, Some(3));

    let players = vec![
        PlayerSubmission { player_number: 1, score: 3, proof_url: "s3://proof1.jpg".to_string() },
        PlayerSubmission { player_number: 2, score: 2, proof_url: "s3://proof2.jpg".to_string() },
        PlayerSubmission { player_number: 3, score: 4, proof_url: "s3://proof3.jpg".to_string() },
    ];
    let computed_sec: u32 = players.iter().map(|p| p.score).sum();
    assert_eq!(computed_sec, 9);

    let score = SubmissionScore { placement: 1, secondary_stat: computed_sec };
    assert!(game.validate_score(&score).is_ok());
    // 3 (Top 1) + 9 (manches) = 12
    assert_eq!(game.calculate_points(&score), 12);
}

#[test]
fn test_team_deserialization_defaults() {
    use crate::models::Team;

    // Team sans le champ `is_verified` (rétrocompatibilité MongoDB)
    let json_legacy = r#"{
        "name": "Old Team",
        "captain_username": "oldcap",
        "captain_password_hash": "hash123",
        "total_points": 10
    }"#;
    let team: Team = serde_json::from_str(json_legacy).expect("Deserialization failed");
    assert!(team.is_verified, "Une équipe existante sans is_verified doit être vérifiée par défaut");

    // Team avec `is_verified: false`
    let json_pending = r#"{
        "name": "Pending Team",
        "captain_username": "newcap",
        "captain_password_hash": "hash123",
        "total_points": 0,
        "is_verified": false
    }"#;
    let pending_team: Team = serde_json::from_str(json_pending).expect("Deserialization failed");
    assert!(!pending_team.is_verified, "Une équipe avec is_verified: false doit rester non vérifiée");
}

#[test]
fn test_game_is_score_ranked() {
    let geoguessr = Game {
        id: None,
        name: "GeoGuessr".to_string(),
        cover_image: String::new(),
        format: GameFormat::Brawl,
        rules: "Rules".to_string(),
        placement_points: vec![25, 18, 15, 12, 10, 8, 6, 4, 2, 1],
        default_placement_points: 0,
        points_per_secondary_stat: 0,
        secondary_stat_label: Some("Score brut".to_string()),
        max_placement: 10,
        max_secondary_stat: 75000,
        team_player_count: Some(3),
    };
    assert!(geoguessr.is_score_ranked());

    let smash = Game {
        id: None,
        name: "SSBU".to_string(),
        cover_image: String::new(),
        format: GameFormat::Duel,
        rules: "Rules".to_string(),
        placement_points: vec![3, 1],
        default_placement_points: 0,
        points_per_secondary_stat: 0,
        secondary_stat_label: None,
        max_placement: 2,
        max_secondary_stat: 0,
        team_player_count: None,
    };
    assert!(!smash.is_score_ranked());

    let mario_kart = Game {
        id: None,
        name: "MK8".to_string(),
        cover_image: String::new(),
        format: GameFormat::Brawl,
        rules: "Rules".to_string(),
        placement_points: vec![0],
        default_placement_points: 0,
        points_per_secondary_stat: 1,
        secondary_stat_label: Some("Points de course".to_string()),
        max_placement: 1,
        max_secondary_stat: 200,
        team_player_count: Some(3),
    };
    assert!(!mario_kart.is_score_ranked());
}

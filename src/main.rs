// #![allow(dead_code)]

use crate::errors::Errors;
use crate::routes::create_router;
use axum::extract::DefaultBodyLimit;

use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod auth;
mod db;
mod errors;
mod models;
mod routes;
mod services;
mod ws;

struct Config {
    mongo_uri: String,
    jwt_secret: String,
    port: u16,
    cellar_host: String,
    cellar_key_id: String,
    cellar_key_secret: String,
    cellar_bucket: String,
}

fn get_env(key: &str) -> String {
    match std::env::var(key) {
        Ok(val) => val,
        Err(_) => {
            eprintln!("❌ Variable d'environnement obligatoire manquante : '{key}'");
            std::process::exit(1);
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let jwt_secret = get_env("JWT_SECRET");
        let mongo_uri = get_env("MONGODB_ADDON_URI");
        // Variables Cellar S3
        let cellar_host = get_env("CELLAR_ADDON_HOST");
        let cellar_key_id = get_env("CELLAR_ADDON_KEY_ID");
        let cellar_key_secret = get_env("CELLAR_ADDON_KEY_SECRET");
        let cellar_bucket = get_env("CELLAR_BUCKET_NAME");
        // Pour le PORT (qui est optionnel et doit être converti en u16)
        let port = match std::env::var("PORT") {
            Ok(p) => match p.parse::<u16>() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("❌ La variable 'PORT' doit être un nombre valide (reçu : '{p}')");
                    std::process::exit(1);
                }
            },
            Err(_) => 8080,
        };

        Self {
            mongo_uri,
            jwt_secret,
            port,
            cellar_host,
            cellar_key_id,
            cellar_key_secret,
            cellar_bucket,
        }
    }
}

async fn run() -> Result<(), Errors> {
    let config = Config::from_env();

    let client = mongodb::Client::with_uri_str(&config.mongo_uri).await?;
    let db = client.database("lan_tournament");
    db::init_indexes(&db).await?;
    let s3=services::S3Service::new(config.cellar_host, config.cellar_key_id, config.cellar_key_secret, config.cellar_bucket);
    let (ws_tx, _) = tokio::sync::broadcast::channel(100);
    let rate_limiter = auth::RateLimiter::new(5, std::time::Duration::from_secs(60));
    let state = std::sync::Arc::new(models::AppState {
        db,
        jwt_secret: config.jwt_secret,
        ws_tx,
        s3,
        rate_limiter,
    });

    let routeur = create_router()
        .with_state(state)
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        .fallback_service(ServeDir::new("static"));

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await?;
    println!("Serveur lancé sur http://localhost:{}", config.port);
    axum::serve(listener, routeur).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    dotenvy::dotenv().ok(); // Charge les variables du .env en local
    if let Err(err) = run().await {
        eprintln!("❌ Erreur fatale au démarrage : {err}");
        std::process::exit(1);
    }
}
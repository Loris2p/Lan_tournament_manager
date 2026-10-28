// #![allow(dead_code)]

use crate::routes::create_router;
use axum::extract::DefaultBodyLimit;

use tokio::net::TcpListener;
use tower_http::services::ServeDir;

pub mod auth;
mod db;
mod errors;
mod models;
mod routes;
mod ws;

pub struct Config {
    mongo_uri: String,
    jwt_secret: String,
    port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        // 1. JWT_SECRET est obligatoire (prod & local)
        let jwt_secret = std::env::var("JWT_SECRET")
            .map_err(|_| "La variable d'environnement 'JWT_SECRET' est obligatoire".to_string())?;
        // 2. MONGODB_ADDON_URI : fournie par Clever Cloud en prod, ou fallback local Docker
        let mongo_uri = std::env::var("MONGODB_ADDON_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017/lan_tournament".to_string());
        // 3. PORT : fourni par Clever Cloud ou 8080 par défaut
        let port = match std::env::var("PORT") {
            Ok(p) => p.parse::<u16>().map_err(|_| {
                format!("La variable 'PORT' doit être un entier valide (reçu : '{p}')")
            })?,
            Err(_) => 8080,
        };
        Ok(Self {
            mongo_uri,
            jwt_secret,
            port,
        })
    }
}

#[tokio::main]
pub async fn main() {
    println!("Hello, world!");
    dotenvy::dotenv().ok(); // En local : charge les variables du .env. en prod : fichier non chargé (récup immédiate dans les secrets)
    let config = match Config::from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("❌ Erreur de configuration : {err}");
            std::process::exit(1);
        }
    };
    let client = match mongodb::Client::with_uri_str(&config.mongo_uri).await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Impossible d'initialiser le client MongoDB : {err}");
            std::process::exit(1);
        }
    };
    let db = client.database("lan_tournament");
    if let Err(err) = db::init_indexes(&db).await {
        eprintln!("Impossible d'initialiser les index de MongoDB : {err}");
        std::process::exit(1);
    }

    let (ws_tx, _) = tokio::sync::broadcast::channel(100);
    let state = std::sync::Arc::new(models::AppState {
        db,
        jwt_secret: config.jwt_secret,
        ws_tx,
    });
    let routeur = create_router()
        .with_state(state)
        .layer(DefaultBodyLimit::max(5 * 1024 * 1024))
        .fallback_service(ServeDir::new("static"));
    let addr = format!("0.0.0.0:{}", config.port);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(err) => {
            eprintln!("Impossible d'écouter sur {addr} : {err}");
            std::process::exit(1);
        }
    };
    print!("Serveur lancé sur http://localhost:{}", config.port);
    if let Err(err) = axum::serve(listener, routeur).await {
        eprintln!("Erreur lors de l'exécution du serveur : {err}");
        std::process::exit(1);
    }
}

use crate::auth::{verify_jwt, UserRole};
use crate::db;
use askama::Template;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::CookieJar;
use serde::Serialize;

pub struct HtmlTemplate<T>(pub T);

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub code: u16,
    pub title: String,
    pub message: String,
}

pub fn render_error(
    code: StatusCode,
    title: impl Into<String>,
    message: impl Into<String>,
) -> (StatusCode, HtmlTemplate<ErrorTemplate>) {
    (
        code,
        HtmlTemplate(ErrorTemplate {
            code: code.as_u16(),
            title: title.into(),
            message: message.into(),
        }),
    )
}

pub async fn handle_404(
    uri: axum::http::Uri,
    headers: axum::http::HeaderMap,
) -> Response {
    let is_api = uri.path().starts_with("/api");
    let accepts_html = headers
        .get(axum::http::header::ACCEPT)
        .and_then(|h| h.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false);

    if is_api || !accepts_html {
        (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "error": "Ressource introuvable",
                "path": uri.path()
            })),
        )
            .into_response()
    } else {
        render_error(
            StatusCode::NOT_FOUND,
            "Secteur Introuvable",
            "La page demandée n'existe pas ou a été déplacée.",
        )
        .into_response()
    }
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erreur lors du rendu du template : {err}"),
            )
                .into_response(),
        }
    }
}

#[allow(dead_code)]
pub struct NavAuth {
    pub is_logged_in: bool,
    pub target_url: String,
    pub label: String,
}

impl NavAuth {
    pub fn from_jar(jar: &CookieJar, secret: &str) -> Self {
        if let Some(c) = jar.get("lan_session") {
            if let Ok(claims) = verify_jwt(c.value(), secret) {
                return match claims.role {
                    UserRole::Admin => Self {
                        is_logged_in: true,
                        target_url: "/admin".to_string(),
                        label: "Espace Arbitre".to_string(),
                    },
                    UserRole::Captain { .. } => Self {
                        is_logged_in: true,
                        target_url: "/captain".to_string(),
                        label: "Espace Équipe".to_string(),
                    },
                };
            }
        }
        Self {
            is_logged_in: false,
            target_url: "/login".to_string(),
            label: "Connexion".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TournamentStatusView {
    pub label: String,
    pub dot_color: String,
    pub start_at: Option<String>,
    pub end_at: Option<String>,
    pub is_paused: bool,
    pub is_running: bool,
}

pub async fn get_tournament_status_view(db: &mongodb::Database) -> TournamentStatusView {
    let settings = db::get_tournament_settings(db).await.unwrap_or_default();
    let (label, dot_color) = if settings.is_paused {
        ("TOURNOI EN PAUSE".to_string(), "#f97316".to_string())
    } else {
        let now = chrono::Utc::now();
        let parse_dt = |s: &str| -> Option<chrono::DateTime<chrono::Utc>> {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").map(|naive| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            naive,
                            chrono::Utc,
                        )
                    })
                })
                .ok()
        };

        let start_dt = settings.start_at.as_deref().and_then(parse_dt);
        let end_dt = settings.end_at.as_deref().and_then(parse_dt);

        if let Some(start) = start_dt {
            if now < start {
                ("TOURNOI À VENIR".to_string(), "#eab308".to_string())
            } else if let Some(end) = end_dt {
                if now > end {
                    ("TOURNOI TERMINÉ".to_string(), "#64748b".to_string())
                } else {
                    ("TOURNOI EN COURS".to_string(), "#10b981".to_string())
                }
            } else {
                ("TOURNOI EN COURS".to_string(), "#10b981".to_string())
            }
        } else if let Some(end) = end_dt {
            if now > end {
                ("TOURNOI TERMINÉ".to_string(), "#64748b".to_string())
            } else {
                ("TOURNOI EN COURS".to_string(), "#10b981".to_string())
            }
        } else {
            ("TOURNOI EN COURS".to_string(), "#10b981".to_string())
        }
    };

    TournamentStatusView {
        label,
        dot_color,
        start_at: settings.start_at,
        end_at: settings.end_at,
        is_paused: settings.is_paused,
        is_running: !settings.is_paused,
    }
}

/// Cookie de préférence posé par le sélecteur de disposition (static/js/theme-switcher.js).
/// Le markup des pages publiques dépend de la disposition : "arena" par défaut, sauf si l'utilisateur a choisi un autre layout.
pub fn wants_arena(jar: &CookieJar) -> bool {
    jar.get("ltm_layout")
        .map(|c| c.value() == "arena")
        .unwrap_or(true)
}

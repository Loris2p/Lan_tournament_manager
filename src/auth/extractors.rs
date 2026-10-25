use crate::auth::claims::Claims;
use crate::auth::{UserRole, verify_jwt};
use crate::errors::Errors;
use crate::models::{ AppState};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use std::sync::Arc;
use crate::auth::UserRole::Captain;

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub claims: Claims,
}
impl FromRequestParts<Arc<AppState>> for AuthenticatedUser {
    type Rejection = Errors;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookies_token = match jar.get("lan_session") {
            Some(c) => c.value(),
            None => {
                return Err(Self::Rejection::Error401Unauthorized(
                    "Cookie non valable".to_string(),
                ));
            }
        };
        let claims = verify_jwt(cookies_token, &state.jwt_secret)?;
        Ok(Self { claims })
    }
}
#[derive(Debug, Clone)]
pub struct CaptainUser {
    pub claims: Claims,
}
impl FromRequestParts<Arc<AppState>> for CaptainUser {
    type Rejection = Errors;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;
        match auth_user.claims.role {
            Captain {..} => Ok(CaptainUser {
                claims: auth_user.claims,
            }),
            _ => Err(Self::Rejection::Error403Forbidden(
                "Accès réservé aux capitaines".to_string(),
            )),
        }
    }

}
#[derive(Debug, Clone)]
pub struct AdminUser {
    pub claims: Claims,
}
impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = Errors;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthenticatedUser::from_request_parts(parts, state).await?;
        match auth_user.claims.role {
            UserRole::Admin => Ok(AdminUser {
                claims: auth_user.claims,
            }),
            _ => Err(Self::Rejection::Error403Forbidden(
                "Accès réservé aux Admins".to_string(),
            )),
        }
    }
}

use axum::{extract::FromRequestParts, http::request::Parts, Json};
use serde_json::json;
use sqlx::FromRow;
use uuid::Uuid;

use crate::{jwt, state::AppState};

/// The authenticated principal, resolved from the estate's shared JWT.
pub struct CurrentUser {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

impl CurrentUser {
    pub fn is_admin(&self) -> bool {
        self.role.eq_ignore_ascii_case("ADMIN")
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = (axum::http::StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "missing bearer token" })),
                )
            })?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                (
                    axum::http::StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "missing bearer token" })),
                )
            })?;

        let claims = jwt::validate_token(&state.config.jwt_secret, token).ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "invalid or expired token" })),
            )
        })?;

        Ok(Self {
            user_id: claims.sub,
            username: claims.username,
            role: claims.role,
        })
    }
}

/// Simple writer identity stored on each article (name + auth user id).
#[derive(Debug, Clone, FromRow)]
pub struct AuthorRow {
    pub id: Uuid,
    pub auth_user_id: String,
    pub role: String,
    pub name: String,
    pub email: String,
}

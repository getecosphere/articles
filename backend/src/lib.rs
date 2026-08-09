pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod jwt;
pub mod request_id;
pub mod routes;
pub mod state;

use axum::Router;
use anyhow::Result;

pub async fn bootstrap() -> Result<Router> {
    let config = config::AppConfig::from_env()?;
    let state = state::AppState::connect(config).await?;
    db::migrate(&state.pool).await?;
    Ok(routes::build_router(state))
}

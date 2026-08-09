use std::time::Duration;

use axum::{
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{
    cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer},
    limit::RequestBodyLimitLayer,
};

use crate::{handlers, state::AppState};

const MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

pub fn build_router(state: AppState) -> Router {
    let origins: Vec<_> = state
        .config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(AllowMethods::list([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
            axum::http::Method::HEAD,
        ]))
        .allow_headers(AllowHeaders::mirror_request())
        .expose_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    let api_routes = Router::new()
        .route("/health", get(handlers::articles::health))
        .route("/", get(handlers::articles::health))
        .route(
            "/articles",
            get(handlers::articles::list_published).post(handlers::articles::create_article),
        )
        .route(
            "/articles/all",
            get(handlers::articles::list_all),
        )
        .route(
            "/articles/:id",
            axum::routing::put(handlers::articles::update_article)
                .delete(handlers::articles::delete_article),
        )
        .route(
            "/articles/slug/:slug",
            get(handlers::articles::get_published_by_slug),
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
                .layer(axum::middleware::map_response(security_headers)),
        )
        .with_state(state);

    Router::new()
        .nest("/api", api_routes)
        .layer(cors)
        .layer(axum::middleware::from_fn(crate::request_id::propagate))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

async fn security_headers(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", HeaderValue::from_static("nosniff"));
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("x-xss-protection", HeaderValue::from_static("0"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "cache-control",
        HeaderValue::from_static("no-cache, no-store, max-age=0, must-revalidate"),
    );
    headers.insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

pub fn rate_limit_response() -> Response {
    (
        StatusCode::TOO_MANY_REQUESTS,
        axum::Json(serde_json::json!({ "error": "Too many requests" })),
    )
        .into_response()
}

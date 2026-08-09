use articles_backend::bootstrap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "articles_backend=info,tower_http=info".into()),
        )
        .json()
        .init();

    let app = bootstrap().await?;
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".into());
    let addr: std::net::SocketAddr = format!("0.0.0.0:{port}").parse()?;
    tracing::info!(port = %port, "starting articles-backend");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

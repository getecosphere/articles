use std::env;

const KNOWN_PLACEHOLDER_SECRETS: &[&str] = &[
    "your-secret-key-change-in-production",
    "change-this-secret",
    "secret",
];

const MIN_JWT_SECRET_BYTES: usize = 32;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub server_port: u16,
    pub cors_allowed_origins: Vec<String>,
    pub public_site_url: String,
    pub admin_only: bool,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let server_port = env::var("SERVER_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();
        if jwt_secret.is_empty() {
            anyhow::bail!("JWT_SECRET is not set. Refusing to start.");
        }
        if KNOWN_PLACEHOLDER_SECRETS.contains(&jwt_secret.as_str()) {
            anyhow::bail!("JWT_SECRET is set to a known placeholder value. Refusing to start.");
        }
        if jwt_secret.len() < MIN_JWT_SECRET_BYTES {
            anyhow::bail!("JWT_SECRET is too short for HS512 signing.");
        }

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://articles_user:articles@localhost:5432/articles".to_string()),
            jwt_secret,
            server_port,
            cors_allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            public_site_url: env::var("PUBLIC_SITE_URL")
                .unwrap_or_else(|_| "http://localhost:4321".to_string()),
            admin_only: env::var("ARTICLES_ADMIN_ONLY")
                .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(true),
        })
    }
}

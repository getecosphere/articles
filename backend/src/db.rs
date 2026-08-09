use sqlx::postgres::PgPool;

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    let schema = include_str!("migrations.sql");
    for statement in schema.split(';').filter(|s| !s.trim().is_empty()) {
        sqlx::query(statement).execute(pool).await?;
    }
    tracing::info!("schema migration applied");
    Ok(())
}

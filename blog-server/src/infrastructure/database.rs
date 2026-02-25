use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn create_pool(database_url: &str) -> Result<PgPool> {
    let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse()
        .unwrap_or(5);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;

    tracing::info!("Database connection pool created");
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;

    tracing::info!("Database migrations completed");
    Ok(())
}

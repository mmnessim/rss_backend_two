use std::fs;

use sqlx::SqlitePool;

pub mod crud;

pub async fn initialize_database() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    let mut db_path = std::env::current_dir()?;
    db_path.push("feeds.db");

    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !db_path.exists() {
        fs::File::create(&db_path)?;
        tracing::info!("Created database file at {}", db_path.display());
    }

    let conn_str = format!("sqlite://{}", db_path.display());
    tracing::info!("Connecting to database at {}", conn_str);

    let pool = sqlx::SqlitePool::connect(&conn_str).await?;

    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rss_source TEXT NOT NULL,
            title TEXT NOT NULL,
            link TEXT,
            description TEXT,
            guid TEXT UNIQUE,
            time_added INTEGER NOT NULL,
            pub_date INTEGER,
            categories TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    {
        Ok(res) => tracing::info!("articles table initialized {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    match sqlx::migrate!("src/data/migrations").run(&pool).await {
        Ok(_) => tracing::info!("Migrations applied successfully"),
        Err(e) => {
            tracing::error!("Error applying migrations: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    }

    Ok(pool)
}

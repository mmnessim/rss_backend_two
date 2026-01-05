use std::fs;

pub async fn initialize_database() -> Result<(), Box<dyn std::error::Error>> {
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
        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            url TEXT NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    {
        Ok(res) => tracing::info!("Query executed {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id INTEGER PRIMARY KEY,
            title TEXT,
            description TEXT
        );
        "#,
    )
    .execute(&pool)
    .await
    {
        Ok(res) => tracing::info!("Query executed {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    Ok(())
}

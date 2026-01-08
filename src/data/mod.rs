pub mod crud;

#[cfg(feature = "sqlite")]
pub type DbPool = sqlx::SqlitePool;
#[cfg(feature = "postgres")]
pub type DbPool = sqlx::PgPool;

pub async fn initialize_database() -> Result<DbPool, Box<dyn std::error::Error>> {
    let pool = match connect().await {
        Ok(p) => p,
        Err(e) => return Err(e),
    };

    if let Err(e) = create_table(&pool).await {
        return Err(e);
    }

    // match sqlx::migrate!("src/data/migrations").run(&pool).await {
    //     Ok(_) => tracing::info!("Migrations applied successfully"),
    //     Err(e) => {
    //         tracing::error!("Error applying migrations: {:?}", e);
    //         return Err(Box::new(e) as Box<dyn std::error::Error>);
    //     }
    // }

    Ok(pool)
}

#[cfg(feature = "sqlite")]
async fn connect() -> Result<DbPool, Box<dyn std::error::Error>> {
    let mut db_path = std::env::current_dir()?;
    db_path.push("feeds.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if !db_path.exists() {
        std::fs::File::create(&db_path)?;
        tracing::info!("Created database file at {}", db_path.display());
    }

    let conn_str = format!("sqlite://{}", db_path.display());
    tracing::info!("Connecting to database at {}", conn_str);

    let pool = sqlx::SqlitePool::connect(&conn_str).await?;
    return Ok(pool);
}

#[cfg(feature = "sqlite")]
async fn create_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source TEXT NOT NULL,
            url TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    {
        Ok(res) => tracing::info!("articles table initialized {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

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
    .execute(pool)
    .await
    {
        Ok(res) => tracing::info!("articles table initialized {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    Ok(())
}

#[cfg(feature = "postgres")]
async fn connect() -> Result<DbPool, Box<dyn std::error::Error>> {
    let conn_string = "postgres://rss_user:rss_pass@db:5432/rss_db";
    let pool = match sqlx::PgPool::connect(conn_string).await {
        Ok(p) => p,
        Err(e) => return Err(Box::new(e)),
    };
    Ok(pool)
}

#[cfg(feature = "postgres")]
async fn create_table(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS feeds (
            id BIGSERIAL PRIMARY KEY,
            source TEXT NOT NULL,
            url TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    {
        Ok(res) => tracing::info!("feeds table initialized {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS articles (
            id BIGSERIAL PRIMARY KEY,
            rss_source TEXT NOT NULL,
            title TEXT NOT NULL,
            link TEXT,
            description TEXT,
            guid TEXT UNIQUE,
            time_added BIGINT NOT NULL,
            pub_date BIGINT,
            categories TEXT
        );
        "#,
    )
    .execute(pool)
    .await
    {
        Ok(res) => tracing::info!("articles table initialized {:?}", res),
        Err(e) => {
            tracing::error!("Error executing query: {:?}", e);
            return Err(Box::new(e) as Box<dyn std::error::Error>);
        }
    };

    Ok(())
}

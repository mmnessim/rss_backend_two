use crate::data::DbPool;
use crate::data::models::{Article, DbArticle};

/// Returns all articles
/// Could be a very expensive call as articles will probably average over 50k
pub async fn get_all_articles(pool: &DbPool) -> Vec<Article> {
    let articles = match sqlx::query_as::<_, DbArticle>("SELECT * FROM articles;")
        .fetch_all(pool)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            println!("Error fetching: {:?}", e);
            vec![]
        }
    };

    return articles.into_iter().map(Article::from).collect();
}

/// Simple search with DB queries
pub async fn get_like(query: &str, pool: &DbPool) -> Vec<Article> {
    let sql = if cfg!(feature = "postgres") {
        r#"
            SELECT * FROM articles
            WHERE title ILIKE $1
            OR description ILIKE $2
            ORDER BY pub_date DESC NULLS LAST
            LIMIT 100
        "#
    } else {
        r#"
            SELECT * FROM articles
            WHERE title LIKE ? COLLATE NOCASE
            OR description LIKE ? COLLATE NOCASE
            ORDER BY (pub_date IS NULL), pub_date DESC
            LIMIT 100
        "#
    };

    let articles = match sqlx::query_as::<_, DbArticle>(sql)
        .bind(query)
        .bind(query)
        .fetch_all(pool)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Error searching articles: {:?}", e);
            vec![]
        }
    };

    return articles.into_iter().map(Article::from).collect();
}

/// Not currently used
/// Simple search but sorting by feed
pub async fn _get_by_feed(feed: &str, pool: &DbPool) -> Vec<Article> {
    let sql = if cfg!(feature = "postgres") {
        r#"
           SELECT * FROM articles
           WHERE rss_source ILIKE $1
           ORDER BY pub_date DESC NULLS LAST
           LIMIT 100
           "#
    } else {
        r#"
           SELECT * FROM articles
           WHERE rss_source LIKE ? COLLATE NOCASE
           ORDER BY (pub_date IS NULL), pub_date DESC
           LIMIT 100
           "#
    };

    let articles = match sqlx::query_as::<_, DbArticle>(sql)
        .bind(feed)
        .fetch_all(pool)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            tracing::error!("Error fetching articles: {:?}", e);
            return vec![];
        }
    };

    articles.into_iter().map(Article::from).collect()
}

/// Parses feeds, checks for duplication, removes HTML and inserts into DB
pub async fn insert_from_rss(
    rss: feed_rs::model::Feed,
    source: &str,
    pool: &DbPool,
    meili_articles: &meilisearch_sdk::indexes::Index,
) -> u64 {
    let items = rss.entries;
    if items.is_empty() {
        tracing::debug!("No items found in feed from source: {}", source);
        return 0;
    }
    let mut rows = 0;

    for item in items {
        let mut article = row_to_article(item, source);

        let insert_sql = if cfg!(feature = "postgres") {
            r#"INSERT INTO articles (rss_source, title, link, description, guid, time_added, pub_date, categories)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (guid) DO NOTHING
            RETURNING id
                "#
        } else {
            r#"INSERT OR IGNORE INTO articles
            (rss_source, title, link, description, guid, time_added, pub_date, categories)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#
        };

        match sqlx::query_scalar::<_, i64>(insert_sql)
            .bind(source)
            .bind(article.title.clone())
            .bind(article.link.clone())
            .bind(article.description.clone())
            .bind(article.guid.clone())
            .bind(article.time_added)
            .bind(article.pub_date)
            .bind(article.categories.join(","))
            .fetch_optional(pool)
            .await
        {
            Ok(Some(inserted)) => {
                article.id = inserted;
                rows += 1;
                let _ = meili_articles.add_documents(&[&article], Some("id")).await;
            }
            Ok(None) => {}
            Err(e) => {
                match e {
                    sqlx::Error::Database(_e) => {
                        //tracing::error!("Duplicate article: {:?}", e)
                    }
                    other => tracing::error!("Insert failed: {:?}", other),
                };
            }
        };
    }

    return rows;
}

/// Helper function to turn individual RSS item into Article struct
fn row_to_article(item: feed_rs::model::Entry, source: &str) -> Article {
    let title = item
        .title
        .map_or_else(|| String::from("No Title"), |t| t.content);

    // Remove any html tags that might exist
    let clean_title = crate::util::remove_html(title);

    let mut description = item
        .content
        .map_or_else(|| String::from(""), |d| d.body.unwrap_or_default());

    // Use summary if no description
    if description.is_empty() {
        let summary = item.summary.map_or_else(|| String::from(""), |s| s.content);
        description = summary;
    }

    // Remove any html tags that might exist
    let clean_description = crate::util::remove_html(description);

    let link = item.links.first().map(|l| l.href.clone());

    let guid = item.id.clone();

    let time_added = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let pub_date = match item.published {
        Some(t) => t.timestamp_millis() as i64,
        None => std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64,
    };
    let categories = item
        .categories
        .iter()
        .map(|c| c.term.clone())
        .collect::<Vec<String>>()
        .join(",");

    return Article {
        id: 0,
        rss_source: source.to_string(),
        title: clean_title.clone(),
        link: link.clone(),
        description: clean_description.clone(),
        guid: Some(guid.clone()),
        time_added: time_added as i64,
        pub_date: Some(pub_date),
        categories: categories.split(',').map(|s| s.to_string()).collect(),
    };
}

/// Return article count
pub async fn count_articles(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

/// Delete articles that are older than 30 days
pub async fn delete_old_articles(pool: &DbPool) -> Result<i64, sqlx::Error> {
    tracing::info!("Checking for old articles...");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;

    let ms_per_month: i64 = 24 * 60 * 60 * 1000 * 30;
    let cutoff = now_ms - ms_per_month;

    let sql = if cfg!(feature = "postgres") {
        "DELETE FROM articles WHERE time_added < $1"
    } else {
        "DELETE FROM articles WHERE time_added < ?"
    };

    let res = match sqlx::query(sql).bind(cutoff).execute(pool).await {
        Ok(res) => res,
        Err(e) => {
            // tracing::error!("Error deleting articles: {:?}", e);
            return Err(e);
        }
    };
    let deleted = res.rows_affected() as i64;
    tracing::info!("Deleted {} articles older than 30 days", deleted,);
    Ok(deleted)
}

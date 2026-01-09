use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::data::DbPool;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DbArticle {
    pub id: i64,
    pub rss_source: String,
    pub title: String,
    pub link: Option<String>,
    pub description: String,
    pub guid: Option<String>,
    // Just needed for deleting old articles
    pub time_added: i64,
    pub pub_date: Option<i64>,
    pub categories: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: i64,
    #[serde(rename = "rssSource")]
    pub rss_source: String,
    pub title: String,
    pub link: Option<String>,
    pub description: String,
    pub guid: Option<String>,
    // Just needed for deleting old articles
    pub time_added: i64,
    pub pub_date: Option<i64>,
    // Exposed as an array for the frontend
    pub categories: Vec<String>,
}

impl From<DbArticle> for Article {
    fn from(d: DbArticle) -> Self {
        let categories = d
            .categories
            .map(|s| {
                if s.is_empty() {
                    vec![]
                } else {
                    s.split(',').map(|s| s.to_string()).collect()
                }
            })
            .unwrap_or_default();

        Article {
            id: d.id,
            rss_source: d.rss_source,
            title: d.title,
            link: d.link,
            description: d.description,
            guid: d.guid,
            time_added: d.time_added,
            pub_date: d.pub_date,
            categories,
        }
    }
}

pub async fn get_articles(pool: &DbPool) -> Vec<Article> {
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

pub async fn get_like(query: &str, pool: &DbPool) -> Vec<Article> {
    let sql = if cfg!(feature = "postgres") {
        r#"
            SELECT * FROM articles
            WHERE title LIKE $1
            OR description LIKE $2
            ORDER BY pub_date DESC NULLS LAST
            LIMIT 100
        "#
    } else {
        r#"
            SELECT * FROM articles
            WHERE title LIKE ?
            OR description LIKE ?
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

pub async fn insert_from_rss(rss: feed_rs::model::Feed, source: &str, pool: &DbPool) -> u64 {
    let items = rss.entries;
    if items.is_empty() {
        println!("No items found in feed from source: {}", source);
        return 0;
    }
    let mut rows = 0;

    for item in items {
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

        let pub_date = item.published.map(|d| d.timestamp_millis() as i64);
        let categories = item
            .categories
            .iter()
            .map(|c| c.term.clone())
            .collect::<Vec<String>>()
            .join(",");

        let insert_sql = if cfg!(feature = "postgres") {
            "INSERT INTO articles (rss_source, title, link, description, guid, time_added, pub_date, categories) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
        } else {
            "INSERT INTO articles (rss_source, title, link, description, guid, time_added, pub_date, categories) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        };

        let res = match sqlx::query(insert_sql)
            .bind(source)
            .bind(clean_title)
            .bind(link)
            .bind(clean_description)
            .bind(guid)
            .bind(time_added as i64)
            .bind(pub_date)
            .bind(categories)
            .execute(pool)
            .await
        {
            Ok(r) => {
                r.rows_affected()
                // tracing::info!("Inserted article from rss source: {}", source); r.rows_affected()
            }
            Err(e) => {
                match e {
                    sqlx::Error::Database(_e) => {
                        //tracing::error!("Duplicate article: {:?}", e)
                    }
                    other => tracing::error!("Insert failed: {:?}", other),
                };
                0
            }
        };
        rows += res;
    }

    return rows;
}

pub async fn count_articles(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM articles")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

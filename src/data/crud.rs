use sqlx::{Pool, Sqlite, prelude::FromRow};

#[derive(Debug, Clone, FromRow)]
pub struct Article {
    pub id: u64,
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

pub async fn get_articles(pool: &Pool<Sqlite>) -> Vec<Article> {
    let articles = match sqlx::query_as::<_, Article>("SELECT * FROM articles;")
        .fetch_all(pool)
        .await
    {
        Ok(a) => a,
        Err(e) => {
            println!("Error fetching: {:?}", e);
            vec![]
        }
    };

    return articles;
}

pub async fn put_article(pool: &Pool<Sqlite>, article: Article) -> u64 {
    let row = match sqlx::query("INSERT INTO articles (title, description) VALUES (?, ?)")
        .bind(article.title)
        .bind(article.description)
        .execute(pool)
        .await
    {
        Ok(r) => r.rows_affected(),
        Err(e) => {
            println!("Error inserting article: {:?}", e);
            0
        }
    };
    return row;
}

pub async fn insert_from_rss(rss: feed_rs::model::Feed, source: &str, pool: &Pool<Sqlite>) -> u64 {
    let items = rss.entries;
    if items.is_empty() {
        println!("No items found in feed from source: {}", source);
        return 0;
    }

    for item in items {
        let title = item
            .title
            .map_or_else(|| String::from("No Title"), |t| t.content);

        let mut description = item
            .content
            .map_or_else(|| String::from(""), |d| d.body.unwrap_or_default());

        let summary = item.summary.map_or_else(|| String::from(""), |s| s.content);

        if description.is_empty() {
            description = summary;
        }

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

        let res = match sqlx::query(
        "INSERT INTO articles (rss_source, title, link, description, guid, time_added, pub_date, categories) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
        .bind(source)
        .bind(title)
        .bind(link)
        .bind(description)
        .bind(guid)
        .bind(time_added as i64)
        .bind(pub_date)
        .bind(categories)
        .execute(pool)
        .await {
            Ok(r) =>{
                r.rows_affected()
                // tracing::info!("Inserted article from rss source: {}", source); r.rows_affected()
            },
            Err(e) => {
                match e {
                    sqlx::Error::Database(e) => {
                        // tracing::error!("Duplicate article: {:?}", e)
                    },
                    other => tracing::error!("Insert failed: {:?}", other),
                };
                0
            }
        };
    }

    return 0;
}

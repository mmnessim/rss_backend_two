use sqlx::{Pool, Sqlite, prelude::FromRow};

#[derive(Debug, Clone, FromRow)]
pub struct Article {
    pub id: u64,
    pub title: String,
    pub description: String,
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

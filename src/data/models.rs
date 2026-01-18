use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

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
    #[serde(rename = "pubDateMs")]
    pub pub_date: Option<i64>,
    // Exposed as an array for the frontend
    pub categories: Vec<String>,
}

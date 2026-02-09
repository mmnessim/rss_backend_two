use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::data::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub feeds: Arc<RwLock<Vec<SourceFeed>>>,
    pub meili: meilisearch_sdk::indexes::Index,
}

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
    #[serde(rename = "pubDateMs", default)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceFeed {
    pub source: String,
    pub url: String,
}

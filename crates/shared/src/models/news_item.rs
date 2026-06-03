/*
crates/shared/src/models/news_item.rs
ニュースのItemを定義
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};

/// ニュース1つのItem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
  pub id: usize,
  pub category: String,
  pub title: String,
  pub url: String,
  pub body: Option<String>,
}

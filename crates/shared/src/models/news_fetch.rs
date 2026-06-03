/*
crates/shared/src/models/news_fetch.rs
RSSの取得や本文を取得するときに使う構造体の定義
*/
// 非同期トレイト用
use super::super::errors::AppResult;
use super::news_item::NewsItem;
use async_trait::async_trait;
use serde::Serialize;

// RSSのitem
#[derive(Debug, Serialize)]
pub struct RSSItem {
  // ニュースにidを振るときの初期番号
  pub id_start: usize,
  // ニュースのカテゴリ
  pub category: &'static str,
  // RSSのURL
  pub rss_url: &'static str,
}

// NewsFetcherのトレイト型
#[async_trait]
pub trait NewsFetcher {
  // RSS取得・パースしてrss_itemsを構築する
  async fn rss_feed(&mut self) -> AppResult<()>;

  // rss_itemsからニュース本文を取得してnews_itemsを構築する
  async fn fetch_news(&mut self) -> AppResult<()>;

  // idでフィルタしてnews_itemsを絞り込む
  fn extract_news_items(&mut self, ids: Vec<usize>) -> AppResult<()>;
}

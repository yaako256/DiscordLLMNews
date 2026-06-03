// 非同期トレイト用
use async_trait::async_trait;

use config::AppConfig;
use logger;
use shared::NewsFetcher;
use shared::{NewsItem, RSSItem, errors::AppResult};

pub struct LivedoorNewsFetcher {
  config: AppConfig,
  rss_items: Vec<RSSItem>,
  news_items: Vec<NewsItem>,
}
#[async_trait]
impl NewsFetcher for LivedoorNewsFetcher {
  // RSS取得・パースしてnews_itemsを構築する
  async fn rss_feed(&mut self) -> AppResult<()> {
    Ok(())
  }

  // rss_itemsからニュース本文を取得してnews_itemsを構築する
  async fn fetch_news(&mut self) -> AppResult<()> {
    Ok(())
  }

  // idでフィルタしてnews_itemsを絞り込む
  fn extract_news_items(&mut self, ids: Vec<usize>) -> AppResult<()> {
    Ok(())
  }
}

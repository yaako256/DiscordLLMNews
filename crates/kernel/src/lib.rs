/*
crates/kernel/src/lib.rs
*/
use std::sync::Arc;

// 時間型用
use chrono::{DateTime, FixedOffset, Utc};
// 通常ログ用
use tracing::{error, info, warn};
// 通知ログ用
use logger;
// 共通型
use shared::errors::AppResult;
// config
use config::AppConfig;
// news_fetch
use news_fetch::LivedoorNewsFetcher;
use shared::NewsFetcher;

pub struct Kernel {
  config: Arc<AppConfig>,
  started_at: DateTime<FixedOffset>,
}

impl Kernel {
  pub fn new(config: Arc<AppConfig>) -> Self {
    Self {
      config,
      started_at: Utc::now().fixed_offset(), // インスタンス化時点の時刻
    }
  }

  pub async fn feed(&mut self) -> AppResult<()> {
    println!("feed");

    logger::info("debug", "infoデバッグメッセージ");
    info!("debug infoデバッグメッセージ");
    logger::warn("debug", "warnデバッグメッセージ");
    warn!("debug warnデバッグメッセージ");
    logger::error("debug", "errorデバッグメッセージ");
    error!("debug errorデバッグメッセージ");

    // 書き込み
    infra::write_notification_log().await?;

    for e in logger::to_chunks(5000) {
      info!("{}", e);
    }

    // NewsFetcherのインスタンス
    let mut news_fetcher = LivedoorNewsFetcher::new(Arc::clone(&self.config));
    // RSSの取得
    news_fetcher.rss_feed().await?;

    info!("{:#?}", news_fetcher);

    // started_at を infra に渡す
    //storage::init_data_dir(self.started_at).await?;
    // ...以降のfeed処理
    Ok(())
  }

  pub async fn send(&mut self) -> AppResult<()> {
    println!("send");
    // ファイルがなければStorage エラー
    //storage::read_news_summary().await?;
    // ...以降のsend処理

    Ok(())
  }
}

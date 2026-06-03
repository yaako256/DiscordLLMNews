/*
crates/kernel/src/lib.rs
*/

// 時間型用
use chrono::{DateTime, FixedOffset, Utc};

// ロガー
use logger;
// 共通型
use shared::errors::AppResult;
// config
use config::AppConfig;

pub struct Kernel {
  config: AppConfig,
  started_at: DateTime<FixedOffset>,
}

impl Kernel {
  pub fn new(config: AppConfig) -> Self {
    Self {
      config,
      started_at: Utc::now().fixed_offset(), // インスタンス化時点の時刻
    }
  }

  pub async fn feed(&mut self) -> AppResult<()> {
    println!("feed");

    logger::info("debug", "infoデバッグメッセージ");
    logger::warn("debug", "warnデバッグメッセージ");
    logger::error("debug", "errorデバッグメッセージ");

    // 書き込み
    infra::write_notification_log().await?;

    for e in logger::to_chunks(5000) {
      println!("{}", e);
    }

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

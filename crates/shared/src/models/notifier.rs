/*
crates/shared/src/models/notifier.rs
送信用トレイトの定義
*/
// 非同期トレイト用
use super::super::errors::AppResult;
use async_trait::async_trait;

// Notifierのトレイト型
#[async_trait]
pub trait Notifier {
  // 本文を送信
  async fn send_summary(&self) -> AppResult<()>;

  // ログを送信
  async fn send_logs(&self) -> AppResult<()>;
}

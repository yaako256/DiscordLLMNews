/*
crates/shared/src/errors.rs
エラー型の定義
*/
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
  #[error("Configエラー: {0}")]
  Config(String),
  #[error("RssFeedエラー: {0}")]
  RSSFeed(String),
  #[error("ニュース本文取得エラー: {0}")]
  ArticleFeed(#[from] serde_json::Error),
  #[error("LLMリクエストエラー: {0}")]
  LLMRequest(String),
  #[error("Jsonパースエラー: {0}")]
  JsonParse(String),
  #[error("通知エラー: {0}")]
  Notifier(String),
  #[error("データI/Oエラー: {0}")]
  Storage(String),
}

pub type AppResult<T> = Result<T, AppError>;

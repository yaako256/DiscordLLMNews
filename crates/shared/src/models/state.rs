/*
crates/shared/src/models/state.rs
news_summary.jsonで定義される型を定義する
statusキーをタグとして定義する
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};
// 時間型用
use chrono::{DateTime, FixedOffset};

/// NewsSummary.jsonで定義される方の定義
/// statusをタグとしてenumで定義する
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum NewsSummary {
  Running {
    started_at: DateTime<FixedOffset>,
  },
  Ready {
    prepared_at: DateTime<FixedOffset>,
    message_body: String,
  },
  Failed {
    started_at: DateTime<FixedOffset>,
    finished_at: DateTime<FixedOffset>,
    error_summary: String,
  },
  Sent {
    sent_at: DateTime<FixedOffset>,
    prepared_at: DateTime<FixedOffset>,
    message_body: String,
  },
}

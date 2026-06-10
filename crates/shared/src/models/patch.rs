/*
crates/shared/src/models/patch.rs
patch_summary.jsonで定義される型を定義する
statusキーをタグとして定義する
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};
// 時間型用
use chrono::{DateTime, FixedOffset};

// 送信時に使う方のファイル
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PatchSummary {
  Ready {
    prepared_at: DateTime<FixedOffset>,
    message_body: String,
  },
  Sent {
    sent_at: DateTime<FixedOffset>,
    prepared_at: DateTime<FixedOffset>,
    message_body: String,
  },
  Failed {
    prepared_at: DateTime<FixedOffset>,
    error_summary: String,
  },
}

// パッチ送信記録で使う
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PatchHistory {
  pub version: String,
  pub sent_at: String, // yyyymmdd形式
  pub summary: String, // パッチノート本文
}

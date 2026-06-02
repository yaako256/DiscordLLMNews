/*
crates/shared/src/models/process_history.rs
process_history.jsonで定義される型を定義する
*/
// JSONシリアライズ用
use serde::Serialize;
// 時間型用
use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessHistory {
  pub process: String,
  pub started_at: DateTime<FixedOffset>,
  pub finished_at: DateTime<FixedOffset>,
  pub success: bool,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_stage: Option<String>,
}

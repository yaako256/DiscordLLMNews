/*
crates/shared/src/models/trivia_history.rs
trivia_history.jsonで定義される型を定義する
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TriviaHistory {
  pub time: String,   // yyyymmdd形式
  pub trivia: String, // 雑学本文
}

/*
crates/logger/src/model.rs
ロガー構造体の定義
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};
use serde_json;
// 時間型用
use chrono::{DateTime, FixedOffset, Utc};

// ----------------
// ロガー本体
// ----------------
#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationLogLogger {
  entries: Vec<NotificationLogEntry>,
}
impl NotificationLogLogger {
  pub fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  // impl Into<String>で渡すと String も str も両対応？
  // infoログ追加
  pub fn info(&mut self, stage: impl Into<String>, msg: impl Into<String>) {
    self.push(LogLevel::Info, stage, msg);
  }
  // warnログ追加
  pub fn warn(&mut self, stage: impl Into<String>, msg: impl Into<String>) {
    self.push(LogLevel::Warn, stage, msg);
  }
  // errorログ追加
  pub fn error(&mut self, stage: impl Into<String>, msg: impl Into<String>) {
    self.push(LogLevel::Error, stage, msg);
  }

  // ログをpushする共通型
  fn push(&mut self, level: LogLevel, stage: impl Into<String>, msg: impl Into<String>) {
    self.entries.push(NotificationLogEntry {
      logged_at: Utc::now().with_timezone(&super::time::jst()),
      level,
      stage: stage.into(),
      message: msg.into(),
    });
  }

  // 蓄積したエントリを jsonl 形式で書きだす
  pub fn to_jsonl_string(&self) -> String {
    self
      .entries
      .iter()
      .filter_map(|e| serde_json::to_string(e).ok())
      .collect::<Vec<_>>()
      .join("\n")
      + "\n"
  }

  // 蓄積したエントリを整形済み文字列として書きだす
  pub fn to_formatted_string(&self) -> String {
    self
      .entries
      .iter()
      .map(|e| e.to_formatted_string())
      .collect::<Vec<_>>()
      .join("")
  }
}

// ----------------
// ログレベル
// ----------------
#[derive(Debug, Deserialize, Serialize)]
enum LogLevel {
  Info,
  Warn,
  Error,
}
impl LogLevel {
  // 文字列を返す関数
  fn as_str(&self) -> &'static str {
    match self {
      LogLevel::Info => "Info",
      LogLevel::Warn => "Warn",
      LogLevel::Error => "Error",
    }
  }
}
// serde が必要な場合（jsonl出力用）
/*
impl serde::Serialize for LogLevel {
  fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(self.as_str())
  }
}
*/
// ----------------
// エントリ
// ----------------
#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationLogEntry {
  logged_at: DateTime<FixedOffset>,
  level: LogLevel,
  stage: String,
  message: String,
}
impl NotificationLogEntry {
  // 文字列として整形
  pub fn to_formatted_string(&self) -> String {
    format!(
      "[{}] {:<7}[{}] {}\n",
      self.logged_at.format("%Y/%m/%d %H:%M:%S"),
      format!("[{}]", self.level.as_str()),
      self.stage,
      self.message
    )
  }
}

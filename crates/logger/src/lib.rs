/*
crates/logger/src/lib.rs
グローバルで持つloggerの定義
*/
mod time;
// 標準ライブラリ
// 共通のグローバル変数とするよう
use std::sync::{Mutex, OnceLock};

// ロガー構造体
mod model;
pub use model::NotificationLogEntry;
use model::NotificationLogLogger;

static LOGGER: OnceLock<Mutex<NotificationLogLogger>> = OnceLock::new();

pub fn init() {
  LOGGER
    .set(Mutex::new(NotificationLogLogger::new()))
    .unwrap();
}

pub fn info(stage: impl Into<String>, msg: impl Into<String>) {
  if let Some(logger) = LOGGER.get() {
    logger
      .lock()
      .expect("logger mutex poisoned")
      .info(stage, msg);
  }
}

pub fn warn(stage: impl Into<String>, msg: impl Into<String>) {
  if let Some(logger) = LOGGER.get() {
    logger
      .lock()
      .expect("logger mutex poisoned")
      .warn(stage, msg);
  }
}

pub fn error(stage: impl Into<String>, msg: impl Into<String>) {
  if let Some(logger) = LOGGER.get() {
    logger
      .lock()
      .expect("logger mutex poisoned")
      .error(stage, msg);
  }
}

pub fn to_jsonl_string() -> String {
  LOGGER
    .get()
    .map(|l| l.lock().unwrap().to_jsonl_string())
    .unwrap_or_default()
}

/// 蓄積したエントリを整形済み文字列として返す（Discord通知用）
pub fn to_formatted_string() -> String {
  LOGGER
    .get()
    .map(|l| l.lock().unwrap().to_formatted_string())
    .unwrap_or_default()
}

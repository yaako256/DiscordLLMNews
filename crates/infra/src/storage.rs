/*
crates/infra/src/storage.rs
dataファイルへのI/O処理
*/
// 標準ライブラリ
use std::path::Path;

use shared::ProcessHistory;

// 外部クレート
// 非同期I/O処理
use tokio::fs;
// 一時ファイル名生成
use uuid::Uuid;
// 時間型用
use chrono::{DateTime, FixedOffset};

// workspace内クレート
use logger;
use shared::constants::file::{
  DATA_DIR_PATH, NEWS_SUMMARY_FILE_NAME, NOTIFICATION_LOG_FILE_NAME, PROCESS_HISTORY_FILE_NAME,
  TRIVIA_HISTORY_FILE_NAME,
};
use shared::{
  NewsSummary, TriviaHistory,
  errors::{AppError, AppResult},
};

// ---------------------------------------------------------------
// 初期化
// ---------------------------------------------------------------
/// data ディレクトリと必要なファイルを初期化する
/// - ディレクトリが存在しなければ作成
/// - news_summary.json が存在しなければ初期状態で作成
pub async fn init_data_dir(started_at: DateTime<FixedOffset>) -> AppResult<()> {
  // データフォルダのパスを作成
  let data_dir: &Path = Path::new(DATA_DIR_PATH);

  // dataフォルダがあるか確認。なかったら再帰的に作成する
  ensure_dir(data_dir).await?;

  // news_summary.jsonをrunningで上書き
  write_news_summary(&NewsSummary::Running { started_at }).await?;

  // notification_log.jsonを初期化
  write_notification_log_init().await?;

  Ok(())
}

/// ディレクトリが存在しなければ再帰的に作成する
async fn ensure_dir(dir: &Path) -> AppResult<()> {
  fs::create_dir_all(dir)
    .await
    .map_err(|e| AppError::Storage(format!("ディレクトリ作成失敗 {}: {e}", dir.display())))
}

// ---------------------------------------------------------------
// news_summary.json
// ---------------------------------------------------------------
/// news_summary.json を atomic に書き込む
/// 一時ファイルに書いてからリネームする（書き込み途中でプロセスが死んでも壊れない）
pub async fn write_news_summary(summary: &NewsSummary) -> AppResult<()> {
  // データフォルダのパスを作成
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // news_summary.jsonのパスを作成
  let summary_path = data_dir.join(NEWS_SUMMARY_FILE_NAME);

  // jsonにシリアライズ
  let json = serde_json::to_string_pretty(summary)
    .map_err(|e| AppError::Storage(format!("news_summary シリアライズ失敗: {e}")))?;

  // 書き込み
  atomic_write(&summary_path, json.as_bytes()).await
}

/// news_summary.json を読み込む(send処理で使用)
pub async fn read_news_summary() -> AppResult<NewsSummary> {
  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // news_summary.jsonのパスを作成
  let summary_path = data_dir.join(NEWS_SUMMARY_FILE_NAME);

  // ファイル存在チェックを明示的に行い、専用メッセージで返す
  if !summary_path.exists() {
    return Err(AppError::Storage(
      "news_summary.json が存在しない。feed が未実行の可能性があり".to_string(),
    ));
  }

  // ファイル読み込み
  let bytes = fs::read(summary_path)
    .await
    .map_err(|e| AppError::Storage(format!("news_summary 読み込み失敗: {e}")))?;

  // jsonをデシリアライズ
  serde_json::from_slice(&bytes)
    .map_err(|e| AppError::JsonParse(format!("news_summary パース失敗: {e}")))
}

// ---------------------------------------------------------------
// notification_log.jsonl
// ---------------------------------------------------------------
/// notification_log.json を 初期化(空) する
pub async fn write_notification_log_init() -> AppResult<()> {
  // データフォルダのパスを作成
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // notification_log.jsonlのパスを作成
  let notification_path = data_dir.join(NOTIFICATION_LOG_FILE_NAME);
  // 書き込み
  atomic_write(&notification_path, "".as_bytes()).await
}
/// notification_log.json を atomic に書き込む
/// 一時ファイルに書いてからリネームする（書き込み途中でプロセスが死んでも壊れない）
pub async fn write_notification_log() -> AppResult<()> {
  // データフォルダのパスを作成
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // notification_log.jsonlのパスを作成
  let notification_path = data_dir.join(NOTIFICATION_LOG_FILE_NAME);

  // jsonlシリアライズ
  let json = logger::to_jsonl_string();

  // 書き込み
  atomic_write(&notification_path, json.as_bytes()).await
}

/// notification_log.jsonl を読み込む(send処理で使用)
pub async fn read_notification_log() -> AppResult<Vec<NotificationLogEntry>> {
  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // notification_log.jsonlのパスを作成
  let notification_path = data_dir.join(NOTIFICATION_LOG_FILE_NAME);

  // ファイル存在チェックを明示的に行い、専用メッセージで返す
  if !notification_path.exists() {
    return Err(AppError::Storage(
      "notification_log.jsonl が存在しない。feed が未実行の可能性があり".to_string(),
    ));
  }

  // ファイル読み込み
  let bytes = fs::read(notification_path)
    .await
    .map_err(|e| AppError::Storage(format!("notification_log 読み込み失敗: {e}")))?;

  // jsonをデシリアライズ
  serde_json::from_slice(&bytes)
    .map_err(|e| AppError::JsonParse(format!("notification_log パース失敗: {e}")))
}

// ---------------------------------------------------------------
// trivia_history.jsonl
// ---------------------------------------------------------------
/// trivia_history.jsonl を読み込む(send処理で使用)
pub async fn read_trivia_history(get_num: usize) -> AppResult<Vec<TriviaHistory>> {
  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // trivia_history.jsonlのパスを作成
  let trivia_history_path = data_dir.join(TRIVIA_HISTORY_FILE_NAME);

  // ファイル存在チェックを明示的に行い、専用メッセージで返す
  if !trivia_history_path.exists() {
    return Err(AppError::Storage(
      "trivia_history.jsonl が存在しない。feed が未実行の可能性があり".to_string(),
    ));
  }

  // ファイル読み込み
  let bytes = fs::read(trivia_history_path)
    .await
    .map_err(|e| AppError::Storage(format!("trivia_history 読み込み失敗: {e}")))?;

  // jsonをデシリアライズ
  let histories: Vec<TriviaHistory> = serde_json::from_slice(&bytes)
    .map_err(|e| AppError::JsonParse(format!("trivia_history パース失敗: {e}")))?;

  // 最新n件に絞り込む
  let latest_histories: Vec<TriviaHistory> = histories
    .into_iter() //所有権ごともらう
    .rev() // 後ろが新しいため、これで新しい順にする
    .take(get_num) // n件だけ取得
    .collect(); // ベクターに変換

  Ok(latest_histories)
}

/// trivia_history.jsonl に1行追記する
pub async fn append_trivia_history(entry: &TriviaHistory) -> AppResult<()> {
  use tokio::io::AsyncWriteExt as _;

  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // trivia_history.jsonlのパスを作成
  let trivia_history_path = data_dir.join(TRIVIA_HISTORY_FILE_NAME);

  // jsonシリアライズ
  let mut line = serde_json::to_string(entry)
    .map_err(|e| AppError::Storage(format!("trivia_history シリアライズ失敗: {e}")))?;
  line.push('\n');

  // ファイル読み込み
  let mut file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(trivia_history_path)
    .await
    .map_err(|e| AppError::Storage(format!("trivia_history オープン失敗: {e}")))?;

  // ファイル書き込み
  file
    .write_all(line.as_bytes())
    .await
    .map_err(|e| AppError::Storage(format!("trivia_history 書き込み失敗: {e}")))?;

  Ok(())
}

// ---------------------------------------------------------------
// process_history.jsonl
// ---------------------------------------------------------------
/// process_history.jsonl に1行追記する
pub async fn append_process_history(entry: &ProcessHistory) -> AppResult<()> {
  use tokio::io::AsyncWriteExt as _;

  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // process_history.jsonlのパスを作成
  let process_history_path = data_dir.join(PROCESS_HISTORY_FILE_NAME);

  // jsonシリアライズ
  let mut line = serde_json::to_string(entry)
    .map_err(|e| AppError::Storage(format!("process_history シリアライズ失敗: {e}")))?;
  line.push('\n');

  // ファイル読み込み
  let mut file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(process_history_path)
    .await
    .map_err(|e| AppError::Storage(format!("process_history オープン失敗: {e}")))?;

  // ファイル書き込み
  file
    .write_all(line.as_bytes())
    .await
    .map_err(|e| AppError::Storage(format!("process_history 書き込み失敗: {e}")))?;

  Ok(())
}

// ---------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------
/// 同ディレクトリに一時ファイルを作り、書き込み後にアトミックにリネームする
async fn atomic_write(path: &Path, data: &[u8]) -> AppResult<()> {
  let dir = path.parent().unwrap_or(Path::new("."));
  let tmp_path = dir.join(format!(".tmp_{}", Uuid::new_v4()));

  // 一時ファイルに書き込み
  fs::write(&tmp_path, data)
    .await
    .map_err(|e| AppError::Storage(format!("一時ファイル書き込み失敗: {e}")))?;

  // renameして変換
  fs::rename(&tmp_path, path).await.map_err(|e| {
    let tmp = tmp_path.clone();
    tokio::spawn(async move {
      let _ = fs::remove_file(tmp).await;
    });
    AppError::Storage(format!("atomic rename 失敗: {e}"))
  })?;

  Ok(())
}

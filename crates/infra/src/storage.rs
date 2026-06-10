/*
crates/infra/src/storage.rs
dataファイルへのI/O処理
*/
// 標準ライブラリ
use std::path::Path;

use logger::NotificationLogEntry;
use shared::ProcessHistory;

// 外部クレート
// 非同期I/O処理
use tokio::fs;
// 一時ファイル名生成
use uuid::Uuid;
// 時間型用
use chrono::{DateTime, FixedOffset};
use tracing::warn;
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
  fs::create_dir_all(data_dir)
    .await
    .map_err(|e| AppError::Storage(format!("ディレクトリ作成失敗 {}: {e}", data_dir.display())))?;

  // news_summary.jsonをrunningで上書き
  write_news_summary(&NewsSummary::Running { started_at }).await?;

  // notification_log.jsonを初期化
  write_notification_log_init().await?;

  Ok(())
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

  // ファイル読み込み
  let content = read_file_string(&summary_path, "news_summary").await?;

  // jsonをデシリアライズ
  serde_json::from_str(&content)
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

  // ファイル読み込み
  let content = read_file_string(&notification_path, "notification_log").await?;

  // 空ファイルの場合は空Vecを返す
  if content.trim().is_empty() {
    return Ok(Vec::new());
  }

  // JSONLを1行ずつパース
  // パースに失敗した行はスキップする（壊れた行で全体が失敗しないように）
  let entries = content
    .lines()
    .filter(|line| !line.trim().is_empty())
    .filter_map(|line| {
      serde_json::from_str(line)
        .map_err(|e| {
          warn!("notification_log 行パース失敗(スキップ): {e} | 行: {line}");
          logger::warn(
            "read_notification_log",
            format!("notification_log 行パース失敗(スキップ): {e} | 行: {line}"),
          );
          e
        })
        .ok()
    })
    .collect();

  Ok(entries)
}

// ---------------------------------------------------------------
// trivia_history.jsonl
// ---------------------------------------------------------------
/// trivia_history.jsonl を読み込む(feed処理で使用)
pub async fn read_trivia_history(get_num: usize) -> AppResult<Vec<TriviaHistory>> {
  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // trivia_history.jsonlのパスを作成
  let trivia_history_path = data_dir.join(TRIVIA_HISTORY_FILE_NAME);

  // ファイル読み込み
  let content = read_file_string(&trivia_history_path, "trivia_history").await?;

  // 1行ずつパースする
  let histories: Vec<TriviaHistory> = content
    .lines()
    .filter(|line| !line.trim().is_empty())
    .map(|line| {
      serde_json::from_str(line)
        .map_err(|e| AppError::JsonParse(format!("trivia_history 行パース失敗: {e} / 行: {line}")))
    })
    .collect::<AppResult<Vec<_>>>()?;

  // 最新n件を取得
  let latest_histories: Vec<TriviaHistory> = histories.into_iter().rev().take(get_num).collect();

  Ok(latest_histories)
}

/// trivia_history.jsonl に1行追記する
pub async fn append_trivia_history(entry: &TriviaHistory) -> AppResult<()> {
  //use tokio::io::AsyncWriteExt as _;

  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // trivia_history.jsonlのパスを作成
  let trivia_history_path = data_dir.join(TRIVIA_HISTORY_FILE_NAME);

  /*
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
  */
  append_jsonl(&trivia_history_path, entry, "trivia_history").await
}

// ---------------------------------------------------------------
// process_history.jsonl
// ---------------------------------------------------------------
/// process_history.jsonl に1行追記する
pub async fn append_process_history(entry: &ProcessHistory) -> AppResult<()> {
  //use tokio::io::AsyncWriteExt as _;

  // &strをパス型に変換
  let data_dir: &Path = Path::new(DATA_DIR_PATH);
  // process_history.jsonlのパスを作成
  let process_history_path = data_dir.join(PROCESS_HISTORY_FILE_NAME);
  /*
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
  */
  append_jsonl(&process_history_path, entry, "process_history").await
}

// ---------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------
/// ファイルを文字列として読み込む（存在チェック付き）
async fn read_file_string(path: &Path, label: &str) -> AppResult<String> {
  // ファイルの存在確認
  if !path.exists() {
    return Err(AppError::Storage(format!(
      "{label} が存在しない。feed が未実行の可能性があり"
    )));
  }
  // 文字列としてファイルを読む
  fs::read_to_string(path)
    .await
    .map_err(|e| AppError::Storage(format!("{label} 読み込み失敗: {e}")))
}

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

/// 値をJSONL形式で1行追記する
async fn append_jsonl<T: serde::Serialize>(path: &Path, entry: &T, label: &str) -> AppResult<()> {
  use tokio::io::AsyncWriteExt as _;

  // jsonにシリアライズ
  let mut line = serde_json::to_string(entry)
    .map_err(|e| AppError::Storage(format!("{label} シリアライズ失敗: {e}")))?;

  // 改行を追加
  line.push('\n');

  // ファイルを開く
  let mut file = fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(path)
    .await
    .map_err(|e| AppError::Storage(format!("{label} オープン失敗: {e}")))?;

  // 1行付け足して書き込む
  file
    .write_all(line.as_bytes())
    .await
    .map_err(|e| AppError::Storage(format!("{label} 書き込み失敗: {e}")))
}

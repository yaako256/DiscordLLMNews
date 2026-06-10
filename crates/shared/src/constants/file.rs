/*
crates/shared/src/constants/file.rs
ファイルの定数設定
*/

// データフォルダのパス
pub const DATA_DIR_PATH: &str = "./data/";
// データファイルパス
pub const NEWS_SUMMARY_FILE_NAME: &str = "news_summary.json";
pub const NOTIFICATION_LOG_FILE_NAME: &str = "notification_log.jsonl";
pub const PROCESS_HISTORY_FILE_NAME: &str = "process_history.jsonl";
pub const TRIVIA_HISTORY_FILE_NAME: &str = "trivia_history.jsonl";

// パッチ通知関連のデータファイルパス
pub const PATCH_SUMMARY_FILE_NAME: &str = "patch_summary.json";
pub const PATCH_HISTORY_FILE_NAME: &str = "patch_history.jsonl";

// process_history.jsonl関連
// 保存項目の上限
// pub const PROCESS_HISTORY_MAX_ENTRIES: usize = 450usize;
// 整理後に維持する件数（古いログ削除後の目標値）
// pub const PROCESS_HISTORY_RETAINED_ENTRIES: usize = 365usize;

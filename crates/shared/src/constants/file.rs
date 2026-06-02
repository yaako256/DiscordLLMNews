/*
crates/shared/src/constants/file.rs
ファイルの定数設定
*/

// データファイル名
pub const NEWS_SUMMARY_FILE_NAME: &str = "news_summary.json";
pub const NOTIFICATION_LOG_FILE_NAME: &str = "notification_log.jsonl";

// notification_log.jsonl関連
// 保存項目の上限
pub const NOTIFICATION_LOG_MAX_ENTRIES: usize = 450usize;
/// 整理後に維持する件数（古いログ削除後の目標値）
pub const NOTIFICATION_LOG_RETAINED_ENTRIES: usize = 365usize;

/*
infra_config/src/config/models.rs
app.tomlの内容を格納する構造体
*/
// 標準ライブラリ
// デシリアライズ用
use serde::Deserialize;

/// 設定まとめ
#[derive(Debug, Deserialize)]
pub struct AppConfig {
  pub system: SystemConfig,
  pub rss: RSSConfig,
  pub llm: LLMConfig,
  pub discord: DiscordConfig,

  #[serde(skip)]
  pub prompts: PromptConfig, // loaderで後から詰める

  pub patch: PatchConfig,
}

/// このシステムの全体的なconfig
/// 後で変数名は編集すべきかも
#[derive(Debug, Deserialize)]
pub struct SystemConfig {
  // 何秒ごとにfeed結果を確認するか
  pub poll_interval_secs: u64,

  // feedがhangしたとみなす時間
  pub hang_threshold_minutes: i64,
}

/// RSS関連の設定
#[derive(Debug, Deserialize)]
pub struct RSSConfig {
  // RSSを取得する個数
  pub feed_fetch_limit: usize,

  // サーバ負荷対策
  // RSS取得のクールタイム
  pub rss_fetch_interval_ms: usize,
  // 本文取得のクールタイム
  pub body_fetch_interval_ms: usize,

  // HTTPのタイムアウト秒数
  pub timeout_s: usize,
}

/// LLM関連の設定
#[derive(Debug, Deserialize)]
pub struct LLMConfig {
  // APIキー
  pub gemini_api_key: Vec<String>,
  // モデルの定義
  pub fallback_models: Vec<String>,

  // タイトルから選出する個数
  pub title_select_limit: usize,
  // 本文から選出する個数
  pub body_select_limit: usize,

  // HTTPのタイムアウト秒数
  pub timeout_s: usize,

  // 最大リトライ数
  pub max_retry: usize,

  // クールタイム関連
  pub sleep: LLMSleep,
}

// LLMのクールタイム関連を定義
#[derive(Debug, Deserialize, Clone)]
pub struct LLMSleep {
  // LLMリクエストの標準クールタイム
  pub request_interval_ms: usize,
  // 指数バックオフ関連
  // 初期値[ms]
  pub backoff_initial_delay: usize,
  // 累乗対象[ms]
  pub backoff_base: f64,
  // 累乗の量
  pub backoff_exponent_factor: f64,
  // 最大待機時間[ms]
  pub backoff_max_time: usize,
  // LLMのクールタイム計算式
  // min(sleep_llm_time + backoff_initial_delay * backoff_base ^ (backoff_attempt_factor * count),backoff_max_time)
}

/// 実際に使うプロンプト文字列
#[derive(Debug, Default)] // Deserializeは不要
pub struct PromptConfig {
  pub select_title: String,
  pub select_body: String,
  pub summarize: String,
}

/// Discord関連の設定
#[derive(Debug, Deserialize)]
pub struct DiscordConfig {
  // 通知用webhook
  pub news_webhooks: Vec<String>,
  // ログ通知用webhook
  pub logs_webhooks: Vec<String>,
}

// パッチ通知関連の設定
#[derive(Debug, Deserialize)]
pub struct PatchConfig {
  // パッチのバージョン
  pub version: String,
  // パッチノートのパス
  pub patch_note_path: String,
}

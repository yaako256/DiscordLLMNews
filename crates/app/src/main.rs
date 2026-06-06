/*
crates/app/src/main.rs
コマンド引数を受け取り、kernelに処理を委譲する
*/
use std::sync::Arc;

// エラー型用
use shared::constants::time;
use shared::errors::{AppError, AppResult};
// 外部ライブラリ
// ログ出力用
use tracing_subscriber::{EnvFilter, fmt, prelude::*};
// 時間型用
use chrono::Utc;

#[tokio::main]
async fn main() -> AppResult<()> {
  println!("Hello, world!");
  // ----------------------
  // 初期設定
  // ----------------------
  // スタート時間の取得
  let start_at = Utc::now().with_timezone(&time::jst());
  // configのロード
  let config = Arc::new(config::load_config()?);
  // 通知用ロガーの起動
  logger::init();
  // ログ出力の設定
  // ログファイル
  {
    let log_file = std::fs::File::create("app.log")
      .map_err(|e| AppError::Config(format!("ログファイル作成失敗: {}", e)))?;
    // 表示フィルタ
    let env_filter = "info";
    // 環境変数 RUST_LOG からログレベルを読み込み、無ければデフォルトで「info」にする
    // ファイル出力とかをすぐ増やせて拡張性ましまし、最近流行の定義方法らしい
    tracing_subscriber::registry()
      // ターミナルログ出力定義。
      .with(fmt::layer().with_writer(std::io::stdout).with_ansi(true))
      // ログファイル出力定義。
      .with(fmt::layer().with_writer(log_file).with_ansi(false))
      // 標準出力の設定。
      .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter)))
      .init();
  }
  // HTTP Clientの起動(グローバルで宣言してそれを使いまわす)
  http_client::init(config.rss.timeout_s as u64, config.llm.timeout_s as u64);

  // ----------------------
  // 処理フロー
  // ----------------------
  let mut knl = kernel::Kernel::new(Arc::clone(&config), start_at);
  // feed処理
  //knl.feed().await?;

  knl.send().await?;

  Ok(())
}

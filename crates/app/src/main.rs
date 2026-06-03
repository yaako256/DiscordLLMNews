/*
crates/app/src/main.rs
コマンド引数を受け取り、kernelに処理を委譲する
*/
// エラー型用
use shared::errors::{AppError, AppResult};

// 外部ライブラリ
// ログ出力用
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

#[tokio::main]
async fn main() -> AppResult<()> {
  println!("Hello, world!");
  // ----------------------
  // 初期設定
  // ----------------------
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

  // configのロード
  let config = config::load_config()?;

  // HTTP Clientの起動(グローバルで宣言してそれを使いまわす)
  http_client::init(config.rss.timeout_s as u64, config.llm.timeout_s as u64);

  //println!("{:#?}", config);

  // DataのI/Oのデバッグ
  /*
  let started_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().fixed_offset();
  // init処理
  infra::init_data_dir(started_at).await?;
  // news_summaryの読み込み
  let aa = infra::read_news_summary().await?;
  println!("{:#?}", aa);
  // news_summaryの書き込み
  infra::write_news_summary(&shared::NewsSummary::Ready {
    prepared_at: chrono::Utc::now().fixed_offset(),
    message_body: "ニュース完成！".to_string(),
  })
  .await?;
  // process_historyの追加
  infra::append_process_history(&shared::ProcessHistory {
    process: "debug".to_string(),
    started_at: started_at,
    finished_at: chrono::Utc::now().fixed_offset(),
    success: true,
    error_stage: None,
  })
  .await?;
  */

  let mut knl = kernel::Kernel::new(config);
  knl.feed().await?;

  Ok(())
}

/*
crates/app/src/main.rs
コマンド引数を受け取り、kernelに処理を委譲する
*/
// エラー型用
use shared::errors::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
  println!("Hello, world!");

  let config = config::load_config()?;

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

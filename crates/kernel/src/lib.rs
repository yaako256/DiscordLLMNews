/*
crates/kernel/src/lib.rs
*/
use std::sync::Arc;

// 時間型用
use chrono::{DateTime, FixedOffset, Utc};
// 通常ログ用
use tracing::{debug, error, info, warn};
// 通知ログ用
use logger;
// 共通型
use shared::{
  TriviaHistory,
  errors::{AppError, AppResult},
};
// config
use config::AppConfig;
// LLMリクエスト構造体
use llm::LLMClient;
// news_fetch
use news_fetch::LivedoorNewsFetcher;
use shared::NewsFetcher;

pub struct Kernel {
  config: Arc<AppConfig>,
  started_at: DateTime<FixedOffset>,
}

impl Kernel {
  pub fn new(config: Arc<AppConfig>, started_at: DateTime<FixedOffset>) -> Self {
    Self { config, started_at }
  }

  // feed処理
  pub async fn feed(&mut self) -> AppResult<()> {
    info!(
      "[{}] feed処理開始",
      self.started_at.format("%Y/%m/%d %H:%M:%S")
    );
    // ----------------------
    // 初期処理
    // ----------------------
    // init処理
    // ディレクトリの作成やstatusをrunningにする
    if let Err(e) = infra::init_data_dir(self.started_at).await {
      // 通常エラーログ
      error!("init_data_dir失敗: {e}");
      // 通知用エラーログ
      logger::error("feed", "init_data_dir失敗");

      // 終了処理
      self
        .feed_fail_finish("init_data_dir失敗".to_string())
        .await?;

      // init失敗時はfeed_fail_finishも呼べない可能性があるためそのまま返す
      return Err(e);
    }

    // ----------------------
    // 処理フロー
    // ----------------------
    // 処理フローを別関数に分けて、成功と失敗の処理を楽にする
    let result = self.feed_inner().await;

    // 正常終了したか
    match result {
      Ok(()) => {
        info!(
          "[{}] feed処理正常終了",
          Utc::now().fixed_offset().format("%Y/%m/%d %H:%M:%S")
        );
        Ok(())
      }
      Err(e) => {
        error!("feed処理失敗: {e}");
        logger::error("kernel", format!("feed処理失敗: {e}"));
        // feed_fail_finishのエラーは握りつぶす
        if let Err(fe) = self.feed_fail_finish(e.to_string()).await {
          error!("feed_fail_finish失敗(握りつぶし): {fe}");
        }
        Err(e)
      }
    }
  }

  /// feedのメイン処理
  /// エラーが起きたらそのまま返し、feedでキャッチする
  async fn feed_inner(&mut self) -> AppResult<()> {
    // ----------------------
    // 事前準備
    // ----------------------
    // NewsFetcherのインスタンス
    let mut news_fetcher = LivedoorNewsFetcher::new(Arc::clone(&self.config));
    // LLMリクエストのインスタンス
    let mut llm_client: LLMClient = LLMClient::new(Arc::clone(&self.config)).await?;

    // ----------------------
    // メイン処理
    // ----------------------
    // RSS取得
    info!("RSS取得開始");
    news_fetcher.rss_feed().await?;
    info!("RSS取得完了");

    // 1回目LLMリクエスト(タイトルで選出)
    let news_items = news_fetcher.get_news_items();
    info!("LLM 1回目リクエスト開始 件数:{}", news_items.len());
    let filter: Vec<usize> = llm_client.request_select_title(&news_items).await?;
    info!("LLM 1回目リクエスト完了");
    debug!("LLM 1回目出力: {:?}", filter);
    // IDでフィルタをかけて選出対象だけ抽出
    info!("IDフィルタ開始");
    news_fetcher.extract_news_items(filter)?;
    info!("IDフィルタ完了");

    // ニュース本文の取得
    info!("ニュース本文取得開始");
    news_fetcher.fetch_news().await?;
    info!("ニュース本文取得完了");

    // LLMの2回目リクエスト(本文から選出)
    let news_items = news_fetcher.get_news_items();
    info!("LLM 2回目リクエスト開始 件数:{}", news_items.len());
    let filter: Vec<usize> = llm_client.request_select_body(&news_items).await?;
    info!("LLM 2回目リクエスト完了");
    debug!("LLM 2回目出力: {:?}", filter);
    // IDでフィルタをかけて選出対象だけ抽出
    info!("IDフィルタ開始");
    news_fetcher.extract_news_items(filter)?;
    info!("IDフィルタ完了");

    // 3回目LLMリクエスト(要約・整形)
    let news_items = news_fetcher.get_news_items();
    info!("LLM 3回目リクエスト開始 件数:{}", news_items.len());
    let res_text: String = llm_client.request_summarize(&news_items).await?;
    info!("LLM 3回目リクエスト完了");
    debug!("LLM 3回目出力: {}", res_text);

    // ----------------------
    // 終了処理(データ記録)
    // ----------------------
    // 終了時刻取得
    let finish_at = Utc::now().fixed_offset();

    // news_summaryに書き込む
    infra::write_news_summary(&shared::NewsSummary::Ready {
      prepared_at: finish_at,
      message_body: res_text,
    })
    .await?;

    // トリビアを履歴に保存
    infra::append_trivia_history(&TriviaHistory {
      // yyyy/mm/dd hh:mm の形式に変換
      time: finish_at.format("%Y/%m/%d %H:%M").to_string(),
      trivia: "後でここに豆知識・雑学を入れる!!!".to_string(),
    })
    .await?;

    // process_historyの追加
    infra::append_process_history(&shared::ProcessHistory {
      process: "feed".to_string(),
      started_at: self.started_at,
      finished_at: finish_at,
      success: true,
      error_summary: None,
    })
    .await?;

    // notification_logに書き込む
    infra::write_notification_log().await?;

    Ok(())
  }

  /// feed失敗時の後処理
  /// このメソッド自体のエラーは呼び出し元で握りつぶす
  async fn feed_fail_finish(&self, error_summary: String) -> AppResult<()> {
    info!("feed失敗時の後処理開始");
    // 終了時刻取得
    let finish_at = chrono::Utc::now().fixed_offset();

    // news_summaryに書き込む
    infra::write_news_summary(&shared::NewsSummary::Failed {
      started_at: self.started_at,
      finished_at: finish_at,
      error_summary: error_summary.clone(),
    })
    .await?;

    // process_historyの追加
    infra::append_process_history(&shared::ProcessHistory {
      process: "feed".to_string(),
      started_at: self.started_at,
      finished_at: finish_at,
      success: false,
      error_summary: Some(error_summary),
    })
    .await?;

    // notification_logに書き込む
    infra::write_notification_log().await?;

    Ok(())
  }

  // sendの処理フロー(未実装)
  pub async fn send(&mut self) -> AppResult<()> {
    println!("send");
    // ファイルがなければStorage エラー
    //storage::read_news_summary().await?;
    // ...以降のsend処理
    /*
     // news_summaryの読み込み
     let aa = infra::read_news_summary().await?;
     println!("{:#?}", aa);

    */
    Ok(())
  }
}

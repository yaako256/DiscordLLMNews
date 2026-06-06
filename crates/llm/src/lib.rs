/*
crates/llm/src/lib.rs
LLMへのリクエストを定義
*/
// 標準ライブラリ
use std::sync::Arc;

// 外部クレート
// 非同期処理用
use tokio::time::{Duration, sleep};
// 通常ログ
use tracing::{error, info};

// workspace内クレート
use config::AppConfig;
use logger;
use shared::{
  NewsItem, NewsItemLite, SelectByBodyRequest, SelectByTitleRequest, SelectResponse,
  SummarizeRequest, SummaryResponse,
  errors::{AppError, AppResult},
};

use serde::de::DeserializeOwned;

mod gemini;
mod request;

// -------------------------
// LLMClient
// -------------------------
pub struct LLMClient {
  // replace済みプロンプト
  select_title_prompt: String,
  select_body_prompt: String,
  summarize_prompt: String,

  // ローテーション用インデックス
  api_key_index: usize,
  model_index: usize,

  // configから取り出したもの
  api_keys: Vec<String>,
  fallback_models: Vec<String>,
  max_retry: usize,
  sleep: config::LLMSleep,
}

impl LLMClient {
  pub async fn new(config: Arc<AppConfig>) -> AppResult<Self> {
    // trivia_historyを取得してJSON文字列に変換
    //let trivia_history = infra::read_trivia_history(10).await?;
    //let trivia_history_json = serde_json::to_string(&trivia_history)
    //  .map_err(|e| AppError::Storage(format!("trivia_history シリアライズ失敗: {e}")))?;

    // new時点でreplace可能なプレースホルダーをすべて変換
    // {DATA_JSON}はリクエスト時に埋め込むためここでは触らない
    let select_title_prompt = config
      .prompts
      .select_title
      .replace("{SELECT_LIMIT}", &config.llm.title_select_limit.to_string());

    let select_body_prompt = config
      .prompts
      .select_body
      .replace("{SELECT_LIMIT}", &config.llm.body_select_limit.to_string());

    // summarizeのみTRIVIA_HISTORYもここで埋め込む
    let summarize_prompt = config
      .prompts
      .summarize
      //.replace("{TRIVIA_HISTORY}", &trivia_history_json);
      .replace("{DATE}", "2026年6月3日");

    Ok(Self {
      select_title_prompt,
      select_body_prompt,
      summarize_prompt,
      api_key_index: 0,
      model_index: 0,
      api_keys: config.llm.gemini_api_key.clone(),
      fallback_models: config.llm.fallback_models.clone(),
      max_retry: config.llm.max_retry,
      sleep: config.llm.sleep.clone(),
    })
  }

  // ---------------------------------------------------------------
  // 公開インターフェース
  // ---------------------------------------------------------------
  /// 1回目: タイトルのみで選出
  pub async fn request_select_title(&mut self, items: &[NewsItem]) -> AppResult<Vec<usize>> {
    info!("LLM 1回目リクエスト開始 件数:{}", items.len());
    let request = SelectByTitleRequest {
      items: items.iter().map(|item| NewsItemLite::from(item)).collect(),
    };
    let items_json = serde_json::to_string(&request)
      .map_err(|e| AppError::LLMRequest(format!("リクエストシリアライズ失敗: {e}")))?;
    let prompt = self.select_title_prompt.clone();
    let res: SelectResponse = self.request_with_retry(&prompt, &items_json).await?;
    info!("LLM 1回目リクエスト完了");
    Ok(res.selected_ids)
  }

  /// 2回目: 本文も含めて選出
  pub async fn request_select_body(&mut self, items: &[NewsItem]) -> AppResult<Vec<usize>> {
    info!("LLM 2回目リクエスト開始 件数:{}", items.len());
    let request = SelectByBodyRequest {
      items: items.to_vec(),
    };
    let items_json = serde_json::to_string(&request)
      .map_err(|e| AppError::LLMRequest(format!("リクエストシリアライズ失敗: {e}")))?;
    let prompt = self.select_body_prompt.clone();
    let res: SelectResponse = self.request_with_retry(&prompt, &items_json).await?;
    info!("LLM 2回目リクエスト完了");
    Ok(res.selected_ids)
  }

  /// 3回目: 要約・整形
  pub async fn request_summarize(&mut self, items: &[NewsItem]) -> AppResult<String> {
    info!("LLM 3回目リクエスト開始 件数:{}", items.len());
    let request = SummarizeRequest {
      items: items.to_vec(),
    };
    let items_json = serde_json::to_string(&request)
      .map_err(|e| AppError::LLMRequest(format!("リクエストシリアライズ失敗: {e}")))?;
    let prompt = self.summarize_prompt.clone();
    let res: SummaryResponse = self.request_with_retry(&prompt, &items_json).await?;
    info!("LLM 3回目リクエスト完了");
    Ok(res.contents)
  }

  // ---------------------------------------------------------------
  // 内部処理
  // ---------------------------------------------------------------
  /// リトライ・バックオフ・モデルローテーションを含む共通リクエスト処理
  async fn request_with_retry<T: DeserializeOwned>(
    &mut self,
    prompt: &str,
    items_json: &str,
  ) -> AppResult<T> {
    // {DATA_JSON}をここでreplace
    let prompt = prompt.replace("{DATA_JSON}", items_json);

    for attempt in 0..=self.max_retry {
      // 借用競合を避けるためStringにクローン
      let api_key = self.current_api_key();
      let model = self.current_model();

      match request::send_request::<T>(&api_key, &model, &prompt).await {
        Ok(result) => {
          // 成功時もAPIキーをローテーション
          self.api_key_index += 1;
          return Ok(result);
        }
        Err(e) => {
          error!("試行 {}/{} model:{} {e}", attempt, self.max_retry, model);
          logger::warn(
            "llm",
            format!("試行 {}/{} model:{} {e}", attempt, self.max_retry, model),
          );
          // APIキーとモデルを両方ローテーション
          self.api_key_index += 1;
          self.model_index += 1;

          if attempt < self.max_retry {
            let wait = self.backoff_duration(attempt);
            info!("{:.2}秒待機開始", wait.as_secs_f64());
            sleep(wait).await;
            info!("待機完了");
          }
        }
      }
    }

    Err(AppError::LLMRequest("リトライ上限到達".to_string()))
  }

  /// 現在のAPIキーをStringで返す（借用競合回避のためclone）
  fn current_api_key(&self) -> String {
    self.api_keys[self.api_key_index % self.api_keys.len()].clone()
  }

  /// 現在のモデルをStringで返す（借用競合回避のためclone）
  fn current_model(&self) -> String {
    self.fallback_models[self.model_index % self.fallback_models.len()].clone()
  }

  /// 指数バックオフの待機時間を計算
  /// min(backoff_initial_delay + backoff_base ^ (backoff_exponent_factor * attempt), backoff_max_time)
  fn backoff_duration(&self, attempt: usize) -> Duration {
    let ms = self.sleep.backoff_initial_delay as f64
      + self
        .sleep
        .backoff_base
        .powf(self.sleep.backoff_exponent_factor * attempt as f64);
    let ms = ms.min(self.sleep.backoff_max_time as f64) as u64;
    Duration::from_millis(ms)
  }
}

/*
crates/llm/src/request.rs
request用共通関数の定義
*/
// グローバルで定義したHTTP Client
use http_client;
use shared::errors::{AppError, AppResult};

// トレイト型用
use serde::de::DeserializeOwned;
// ログ用
//use tracing::info;
// geminiの内部通信用構造体
use super::gemini;

// ---------------------------------------------------------------
// Gemini API へのHTTPリクエスト
// ---------------------------------------------------------------
/// Gemini APIにリクエストを送り、TにDeserializeして返す
pub async fn send_request<T: DeserializeOwned>(
  api_key: &str,
  model: &str,
  prompt: &str,
) -> AppResult<T> {
  let url = format!(
    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
    model, api_key
  );

  // Geminiの内部構造(送信の仕組み)に合わせる
  let request_body = gemini::Request {
    contents: vec![gemini::Content {
      parts: vec![gemini::Part {
        text: prompt.to_string(),
      }],
    }],
  };

  //info!("リクエストボディ");
  //info!("{:#?}", request_body);

  // リクエスト送信
  let response = http_client::llm()
    .post(&url)
    .json(&request_body)
    .send()
    .await;

  // エラーだった場合の処理
  let response = match response {
    Ok(res) => res,
    Err(e) => {
      // エラーの種類を判定して簡潔なラベルを作成
      let error_kind = if e.is_timeout() {
        "Timeout"
      } else if e.is_connect() {
        "ConnectError"
      } else if e.is_body() {
        "BodyError"
      } else if e.is_request() {
        "RequestError"
      } else {
        //"Unknown"
        "UnexpectedError"
      };

      return Err(AppError::LLMRequest(format!(
        "通信エラー(kind: {})",
        error_kind
      )));
    }
  };

  // ステータスコード確認
  let status = response.status();
  if !status.is_success() {
    let text = response.text().await.unwrap_or_default();
    return Err(AppError::LLMRequest(format!(
      "APIエラー: status:{status} body:{text}"
    )));
  }

  // body取得
  let body = response
    .text()
    .await
    .map_err(|e| AppError::LLMRequest(format!("body取得失敗: {e}")))?;

  // GeminiレスポンスのJSONパース
  let gemini_res: gemini::Response = serde_json::from_str(&body)
    .map_err(|e| AppError::LLMRequest(format!("Geminiレスポンスパース失敗: {e}\nbody:{body}")))?;

  // candidatesからテキスト抽出
  let text = gemini_res
    .candidates
    .into_iter()
    .next()
    .and_then(|c| c.content.parts.into_iter().next())
    .map(|p| p.text)
    .ok_or_else(|| AppError::LLMRequest("レスポンスにtextが含まれていません".to_string()))?;

  // ```jsonなどのコードブロックを除去
  let clean = extract_json(&text);

  serde_json::from_str::<T>(clean)
    .map_err(|e| AppError::JsonParse(format!("LLMレスポンスのJSONパース失敗: {e} raw:{clean}")))
}

/// LLMレスポンスからJSONを抽出する
/// ```json ... ``` や ``` ... ``` のコードブロックを除去する
fn extract_json(text: &str) -> &str {
  let text = text.trim();

  // ```json〜``` または ```〜``` を除去
  if let Some(inner) = text
    .strip_prefix("```json")
    .or_else(|| text.strip_prefix("```"))
  {
    inner.strip_suffix("```").unwrap_or(inner).trim()
  } else {
    text
  }
}

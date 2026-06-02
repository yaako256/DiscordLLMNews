/*
crates/shared/src/models/dtos.rs
LLMのDTOを定義
*/
// JSONシリアライズ用
use serde::{Deserialize, Serialize};

// 自クレート
use super::news_item::NewsItem;

/// ニュースのItem
/// タイトルのみで本文は含まない
/// 1回目のリクエストで使用する
#[derive(Debug, Serialize)]
pub struct NewsItemLite {
  pub id: usize,
  pub category: String,
  pub title: String,
}

// =========================
// LLMリクエスト1回目
// タイトル情報から選出
// =========================
/// リクエスト型
/// タイトル情報を渡す
#[derive(Debug, Serialize)]
pub struct SelectByTitleRequest {
  pub items: Vec<NewsItemLite>,
}

/// レスポンス型
/// 選出したニュースのIDを戻り値とする
/// LLMリクエスト2回目もこれを使用する
#[derive(Debug, Deserialize)]
pub struct SelectResponse {
  pub selected_ids: Vec<usize>,
}

// =========================
// LLMリクエスト2回目
// 本文も含めた情報から選出
// =========================
/// リクエスト型
/// 本文も含めたニュース情報を渡す
#[derive(Debug, Serialize)]
pub struct SelectByBodyRequest {
  pub items: Vec<NewsItem>,
}

// =========================
// LLMリクエスト3回目
// 本文要約と整形をしてもらう
// =========================
/// リクエスト型
/// 本文を含むニュース情報を渡す
#[derive(Debug, Serialize)]
pub struct SummarizeRequest {
  pub items: Vec<NewsItem>,
}

/// レスポンス型
/// 要約・整形後の文字列を戻り値とする
#[derive(Debug, Deserialize)]
pub struct SummaryResponse {
  pub text: String,
}

/*
crates/llm/src/gemini.rs
gemini用構造体の定義
*/
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------
// Gemini API 通信用の内部構造体
// ---------------------------------------------------------------
// リクエスト用
#[derive(Debug, Serialize)]
pub struct Request {
  pub contents: Vec<Content>,
}

#[derive(Debug, Serialize)]
pub struct Content {
  pub parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
pub struct Part {
  pub text: String,
}

// レスポンス用
#[derive(Debug, Deserialize)]
pub struct Response {
  pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
  pub content: ResContent,
}

#[derive(Debug, Deserialize)]
pub struct ResContent {
  pub parts: Vec<ResPart>,
}

#[derive(Debug, Deserialize)]
pub struct ResPart {
  pub text: String,
}

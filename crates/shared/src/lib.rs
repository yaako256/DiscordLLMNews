/*
crates/shared/src/lib.rs
共通型などを定義する
*/
pub mod constants;
pub mod errors;
mod models;

// 再エクスポート
pub use models::{
  dtos::{
    NewsItemLite, SelectByBodyRequest, SelectByTitleRequest, SelectResponse, SummarizeRequest,
    SummaryResponse,
  },
  news_fetch::{NewsFetcher, RSSItem},
  news_item::NewsItem,
  notifier::Notifier,
  process_history::ProcessHistory,
  state::NewsSummary,
  trivia_history::TriviaHistory,
};

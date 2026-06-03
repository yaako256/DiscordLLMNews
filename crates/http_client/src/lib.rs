/*
crates/http_client/src/lib.rs
グローバルなHTTP Clientを作るためのクレート
*/

// crates/infra/src/http_client.rs

use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();
static LLM_CLIENT: OnceLock<Client> = OnceLock::new();

pub fn init(http_timeout_s: u64, llm_timeout_s: u64) {
  let http = Client::builder()
    .timeout(Duration::from_secs(http_timeout_s))
    .build()
    .expect("HTTPクライアントの初期化失敗");

  let llm = Client::builder()
    .timeout(Duration::from_secs(llm_timeout_s))
    .build()
    .expect("LLMクライアントの初期化失敗");

  HTTP_CLIENT.set(http).unwrap();
  LLM_CLIENT.set(llm).unwrap();
}

pub fn http() -> &'static Client {
  HTTP_CLIENT.get().expect("HTTPクライアントが未初期化です")
}

pub fn llm() -> &'static Client {
  LLM_CLIENT.get().expect("LLMクライアントが未初期化です")
}

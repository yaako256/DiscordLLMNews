/*
crates/config/src/loader.rs
configをapp.tomlや.envからloadする関数の定義
`config/`はバイナリファイルと同じ階層(実行場所)に置く
*/
// 標準ライブラリ
// デシリアライズ用
use serde::Deserialize;

// 外部ライブラリ
// config用
use config::{Config, Environment, File};

// 内部ライブラリ(別クレート)
// エラー型
use shared::errors::{AppError, AppResult};

// 内部ライブラリ(自クレート)
use super::models::{AppConfig, DiscordConfig, LLMConfig, PromptConfig, RSSConfig};

/// プロンプトのパスを入れる構造体
/// loader内だけで使う中間型。外部には公開しない
#[derive(Deserialize)]
struct PromptPaths {
  pub select_title: String,
  pub select_body: String,
  pub summarize: String,
}

// loaderの内部型。AppConfigとprompt_pathsを同時に取り出す
#[derive(Deserialize)]
struct RawConfig {
  rss: RSSConfig,
  llm: LLMConfig,
  discord: DiscordConfig,
  prompt_paths: PromptPaths,
}

/// configをロードする
pub fn load_config() -> AppResult<AppConfig> {
  // dotenv
  dotenvy::from_path(".config/.env").ok();

  // 設定ファイルをロードする
  let settings = Config::builder()
    // 1. TOMLベース
    .add_source(File::with_name(".config/app.toml").required(false))
    // 2. ENV上書き（APP__PATHS__DATA_DIR_PATH形式）
    .add_source(
      Environment::with_prefix("APP")
        .separator("__")
        .try_parsing(true)
        .list_separator(","),
    )
    .build()
    .map_err(|e| AppError::Config(e.to_string()))?;

  // 1回のdeserializeでprompt_pathsも含めてデシリアライズ
  let raw = settings
    .try_deserialize::<RawConfig>()
    .map_err(|e| AppError::Config(e.to_string()))?;

  //プロンプト本文を取得
  let prompts = load_prompts(&raw.prompt_paths)?;

  Ok(AppConfig {
    rss: raw.rss,
    llm: raw.llm,
    discord: raw.discord,
    prompts,
  })
}

/// プロンプト達をロードする
fn load_prompts(paths: &PromptPaths) -> AppResult<PromptConfig> {
  Ok(PromptConfig {
    select_title: read_prompt(&paths.select_title)?,
    select_body: read_prompt(&paths.select_body)?,
    summarize: read_prompt(&paths.summarize)?,
  })
}

/// プロンプト(1つ)をロードする
fn read_prompt(path: &str) -> AppResult<String> {
  std::fs::read_to_string(path)
    .map_err(|e| AppError::Config(format!("プロンプトファイル読込失敗: {path}: {e}")))
}

/*
crates/config/src/lib.rs
設定の構造体とローダーの定義
*/
mod loader;
mod models;

pub use loader::load_config;
pub use models::AppConfig;

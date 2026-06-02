/*
crates/app/src/main.rs
コマンド引数を受け取り、kernelに処理を委譲する
*/
// エラー型用
use shared::errors::{AppError, AppResult};

fn main() -> AppResult<()> {
  println!("Hello, world!");

  let config = config::load_config()?;

  println!("{:#?}", config);
  Ok(())
}

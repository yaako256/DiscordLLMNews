/*
crates/logger/src/time.rs
logger内部のみで使うオフセットを定義する
*/

use chrono::{DateTime, FixedOffset, Utc};

// (crate)でこのクレートだけで使うことを明示できる
fn offset_jst() -> FixedOffset {
  FixedOffset::east_opt(9 * 3600).unwrap()
}

// 現在時刻(日本時間)を返す
pub(crate) fn now_jst() -> DateTime<FixedOffset> {
  Utc::now().with_timezone(&offset_jst())
}

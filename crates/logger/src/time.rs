/*
crates/logger/src/time.rs
logger内部のみで使うオフセットを定義する
*/

use chrono::FixedOffset;

// (crate)でこのクレートだけで使うことを明示できる
pub(crate) fn jst() -> FixedOffset {
  FixedOffset::east_opt(9 * 3600).unwrap()
}

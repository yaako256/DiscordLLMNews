/*
crates/shared/src/constants/time.rs
日本時間を返す関数を作成する
*/

use chrono::FixedOffset;

/// JSTオフセット (+09:00)
pub fn jst() -> FixedOffset {
  FixedOffset::east_opt(9 * 3600).unwrap()
}

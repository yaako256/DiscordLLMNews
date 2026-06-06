/*
crates/shared/src/utils.rs
全体で使うutilsの定義。
現在はtimeのみとなっている。
他にも定義するものが増えたらディレクトリに昇格させる
*/

use chrono::{DateTime, FixedOffset, Utc};

// JSTオフセット (+09:00)を返す
pub fn offset_jst() -> FixedOffset {
  FixedOffset::east_opt(9 * 3600).unwrap()
}

// 現在時刻(日本時間)を返す
pub fn now_jst() -> DateTime<FixedOffset> {
  Utc::now().with_timezone(&offset_jst())
}

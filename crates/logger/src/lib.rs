/*
crates/logger/src/lib.rs
グローバルで持つloggerの定義
*/
use std::sync::{Mutex, OnceLock};
mod model;

pub use model::NotificationLogLogger;

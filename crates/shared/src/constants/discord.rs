/*
crates/shared/src/constants/discord.rs
Discord関連の定数設定
*/

// Discordの制限文字数
// 本文の制限
pub const DISCORD_CONTENT_HARD_LIMIT: usize = 2_000usize;
// 安全マージンを引いた実運用上限
pub const DISCORD_CONTENT_SOFT_LIMIT: usize = 1_800usize;

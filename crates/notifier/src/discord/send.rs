/*
crates/notifier/src/discord/send.rs
*/

use async_trait::async_trait;
use config::AppConfig;
use infra;
use logger::NotificationLogEntry;
use serde_json::json;
use shared::{
  NewsSummary, Notifier, PatchNotifier, PatchSummary,
  errors::{AppError, AppResult},
  utils,
};
use std::sync::Arc;
use tracing::{info, warn};

// Discord定数定義
const DISCORD_MAX_CHARS: usize = 2000;

// ---------------------------------------------------------------
// DiscordSender（ニュース送信用）
// ---------------------------------------------------------------
pub struct DiscordSender {
  config: Arc<AppConfig>,
  send_item: NewsSummary,
  log_items: Vec<NotificationLogEntry>,
}
impl DiscordSender {
  /// configだけでインスタンスを生成する（ログ送信のみ可能な最小構成）
  pub fn new(config: Arc<AppConfig>) -> Self {
    Self {
      config,
      send_item: NewsSummary::Running {
        started_at: utils::now_jst(),
      },
      log_items: Vec::new(),
    }
  }

  /// news_summary.json と notification_log.jsonl を読み込んでインスタンスを生成する
  pub async fn try_load(config: Arc<AppConfig>) -> AppResult<Self> {
    // news_summary.jsonのロード
    let send_item = infra::read_news_summary().await?;

    // notification_log.jsonlのロード
    let log_items = infra::read_notification_log().await?;

    Ok(Self {
      config,
      send_item,
      log_items,
    })
  }

  /// フィールドを最新のファイル内容で上書きする（ポーリング時に使用）
  pub async fn reload(&mut self) -> AppResult<()> {
    self.send_item = infra::read_news_summary().await?;
    self.log_items = infra::read_notification_log().await?;
    Ok(())
  }

  /// send_itemのゲッター関数
  pub fn get_send_item(&self) -> &NewsSummary {
    &self.send_item
  }
}

#[async_trait]
impl Notifier for DiscordSender {
  /// ニュース本文を news_webhooks の全URLへ送信する
  async fn send_summary(&self) -> AppResult<()> {
    let message_body = match &self.send_item {
      NewsSummary::Ready { message_body, .. } => message_body,
      // ready以外のステータスでこのメソッドが呼ばれることは想定しないが、
      // 万が一の場合はエラーとして返す
      other => {
        return Err(AppError::Notifier(format!(
          "send_summary: 送信対象は Ready のみだが {:?} が渡された",
          other
        )));
      }
    };

    info!("ニュース本文送信開始 文字数: {}", message_body.len());
    post_to_webhooks(&self.config.discord.news_webhooks, message_body).await?;
    info!("ニュース本文送信完了");

    Ok(())
  }

  /// notification_log を logs_webhooks の全URLへ送信する
  /// ログが空の場合は送信しない
  /// notification_log(feedのログ) と グローバルlogger(sendのログ) を結合して送信する
  async fn send_logs(&self) -> AppResult<()> {
    // feedのログ(ファイルから読んだもの)
    let file_part = if self.log_items.is_empty() {
      String::new()
    } else {
      format_log_entries(&self.log_items)
    };

    // sendのログ(グローバルloggerから取得)
    let global_part = logger::to_formatted_string();

    // 結合（どちらかが空でも自然につながる）
    let content = match (file_part.is_empty(), global_part.trim().is_empty()) {
      (true, true) => {
        info!("送信するログが空のためスキップ");
        return Ok(());
      }
      (true, false) => global_part,
      (false, true) => file_part,
      (false, false) => format!("{file_part}\n{global_part}"),
    };

    // コードブロック分(8文字)を差し引いた上限でトリミング
    let max_inner = DISCORD_MAX_CHARS.saturating_sub(8);
    let content = if content.len() > max_inner {
      warn!(
        "ログが {max_inner} 文字を超えているためトリミングします ({}文字)",
        content.len()
      );
      content.chars().take(max_inner).collect::<String>()
    } else {
      content
    };

    // send_logs: 呼び出し前にコードブロックで囲む
    let content = format!("```\n{content}\n```");

    info!("通知ログ送信開始");
    post_to_webhooks(&self.config.discord.logs_webhooks, &content).await?;
    info!("通知ログ送信完了");

    Ok(())
  }
}

// ---------------------------------------------------------------
// PatchSender（パッチノート送信用）
// ---------------------------------------------------------------
pub struct DiscordPatchSender {
  config: Arc<AppConfig>,
  send_item: PatchSummary,
}
impl DiscordPatchSender {
  /// patch_summary.json と notification_log.jsonl を読み込んでインスタンスを生成する
  pub async fn try_load(config: Arc<AppConfig>) -> AppResult<Self> {
    // news_summary.jsonのロード
    let send_item = infra::read_patch_summary().await?;

    Ok(Self { config, send_item })
  }
}

#[async_trait]
impl PatchNotifier for DiscordPatchSender {
  /// パッチノートを送信する
  async fn send_patch_note(&self) -> AppResult<()> {
    let message_body = match &self.send_item {
      PatchSummary::Ready { message_body, .. } => message_body,
      // ready以外のステータスでこのメソッドが呼ばれることは想定しないが、
      // 万が一の場合はエラーとして返す
      other => {
        return Err(AppError::Notifier(format!(
          "send_patch_note: 送信対象は Ready のみだが {:?} が渡された",
          other
        )));
      }
    };

    info!("ニュース通知用Webhookに送信");
    info!("パッチノート送信開始 文字数: {}", message_body.len());
    post_to_webhooks(&self.config.discord.news_webhooks, message_body).await?;
    info!("パッチノート送信完了");

    // ログ通知webhookにもパッチ通知したことを通知しておく
    info!("ログ通知用Webhookにも送信");

    // ログ追加
    logger::info(
      "patch",
      format!("{}のパッチノートを通知しました", self.config.patch.version),
    );
    // sendのログ(グローバルloggerから取得)
    let content = logger::to_formatted_string();
    // send_logs: 呼び出し前にコードブロックで囲む
    let content = format!("```\n{content}\n```");

    info!("通知ログ送信開始");
    post_to_webhooks(&self.config.discord.logs_webhooks, &content).await?;
    info!("通知ログ送信完了");

    Ok(())
  }
}

// ---------------------------------------------------------------
// 自由関数（DiscordSender・PatchSender共通）
// ---------------------------------------------------------------
/// 指定された全URLに同じテキストを送信する
async fn post_to_webhooks(urls: &[String], content: &str) -> AppResult<()> {
  // 全 Webhook URL に送る
  for url in urls {
    // 仕様のjson型にする
    let body = json!({ "content": content });

    // 送信する
    let resp = http_client::http()
      .post(url)
      .json(&body)
      .send()
      .await
      .map_err(|e| AppError::Notifier(format!("Discord Webhook 送信失敗: {e}")))?;

    // 送信成功チェック
    if !resp.status().is_success() {
      let status = resp.status();
      return Err(AppError::Notifier(format!(
        "Discord Webhook エラーレスポンス: status={status}"
      )));
    }
  }

  Ok(())
}

/// Vec<NotificationLogEntry> を整形済み文字列に変換する
fn format_log_entries(entries: &[NotificationLogEntry]) -> String {
  entries
    .iter()
    .map(|e| e.to_formatted_string())
    .collect::<Vec<_>>()
    .join("")
}

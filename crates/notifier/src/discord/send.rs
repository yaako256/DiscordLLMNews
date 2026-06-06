use async_trait::async_trait;
use config::AppConfig;
use infra;
use logger::NotificationLogEntry;
use serde_json::json;
use shared::{
  NewsSummary, Notifier,
  errors::{AppError, AppResult},
};
use std::sync::Arc;
use tracing::{info, warn};

// 仮でここに定義
const DISCORD_MAX_CHARS: usize = 2000;

pub struct DiscordSender {
  config: Arc<AppConfig>,
  send_item: NewsSummary,
  log_items: Vec<NotificationLogEntry>,
}
impl DiscordSender {
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

  /// 指定された全URLに同じテキストを送信する
  async fn post_to_webhooks(&self, urls: &[String], content: &str) -> AppResult<()> {
    for url in urls {
      // コードブロックで囲む
      let body = json!({ "content":  format!("```\n{content}\n```") });
      let resp = http_client::http()
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Notifier(format!("Discord Webhook 送信失敗: {e}")))?;

      if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(AppError::Notifier(format!(
          "Discord Webhook エラーレスポンス: status={status}, body={text}"
        )));
      }
    }

    Ok(())
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
    self
      .post_to_webhooks(&self.config.discord.news_webhooks, message_body)
      .await?;
    info!("ニュース本文送信完了");

    Ok(())
  }
  /// notification_log を logs_webhooks の全URLへ送信する
  /// ログが空の場合は送信しない
  async fn send_logs(&self) -> AppResult<()> {
    if self.log_items.is_empty() {
      info!("notification_log が空のため送信スキップ");
      return Ok(());
    }

    // logger クレートの to_chunks でDISCORD_MAX_CHARS単位に分割する
    // NOTE: to_chunksはNotificationLogLoggerのメソッドだが、
    //       ここでは読み込んだVec<NotificationLogEntry>を直接整形する
    let content = format_log_entries(&self.log_items);

    // DISCORD_MAX_CHARSを超える場合は警告を出してトリミング（今回は分割省略）
    let content = if content.len() > DISCORD_MAX_CHARS {
      warn!(
        "notification_log が {DISCORD_MAX_CHARS} 文字を超えているためトリミングします ({}文字)",
        content.len()
      );
      content.chars().take(DISCORD_MAX_CHARS).collect::<String>()
    } else {
      content
    };

    info!("通知ログ送信開始");
    self
      .post_to_webhooks(&self.config.discord.logs_webhooks, &content)
      .await?;
    info!("通知ログ送信完了");

    Ok(())
  }
}

/// Vec<NotificationLogEntry> を Discord 用の文字列に整形する
fn format_log_entries(entries: &[NotificationLogEntry]) -> String {
  // NotificationLogEntryのフィールドはprivateなため、
  // Debug出力を使わずserde_json経由でJSONL文字列として整形する
  // （表示形式はto_formatted_stringに合わせたいが、privateのため代替）
  entries
    .iter()
    .filter_map(|e| serde_json::to_string(e).ok())
    .collect::<Vec<_>>()
    .join("\n")
}

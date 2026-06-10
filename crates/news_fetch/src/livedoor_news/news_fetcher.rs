/*
crates/news_fetch/src/livedoor_news/news_fetcher.rs
ライブドアニュースのNewsFetcher
*/
// 標準ライブラリ
use std::sync::Arc;
use std::sync::OnceLock;
// Vecのカテゴリ分け用
use std::collections::{BTreeMap, VecDeque};

// 非同期処理用
use tokio::time::{Duration, sleep};
// 非同期トレイト用
use async_trait::async_trait;
// 正規表現用
use regex::Regex;
// HTMLセレクタ
use scraper::{Html, Selector};
use tracing::warn;
// 通常ログ
use tracing::{debug, info};
// 乱数用
use rand::seq::SliceRandom;

// workspace内クレート
use config::AppConfig;
use http_client;
use logger;
use shared::NewsFetcher;
use shared::{
  NewsItem, RSSItem,
  errors::{AppError, AppResult},
};

// 自クレート
use super::constants::{
  CATEGORY_UNIT, GOOGLE_AD_REGEX, ID_INCREMENT, LIVEDOOR_BODY_SELECTOR, LIVEDOOR_NEWS_RSS,
  LIVEDOOR_TITLE_SELECTOR,
};

#[derive(Debug)]
pub struct LivedoorNewsFetcher {
  config: Arc<AppConfig>,
  rss_items: &'static [RSSItem],
  news_items: Vec<NewsItem>,
}
impl LivedoorNewsFetcher {
  pub fn new(config: Arc<AppConfig>) -> Self {
    Self {
      config,
      rss_items: LIVEDOOR_NEWS_RSS,
      news_items: Vec::new(),
    }
  }

  /// news_itemsのゲッター関数
  pub fn get_news_items(&self) -> Vec<NewsItem> {
    self.news_items.clone()
  }

  /// カテゴリをバラけさせながら、ランダムにソートする関数
  fn spread_by_id_category(&mut self, category_unit: usize) {
    let mut rng = rand::thread_rng();

    // news_items を一旦取り出す
    let items = std::mem::take(&mut self.news_items);

    // カテゴリごとに分割
    let mut groups: BTreeMap<usize, Vec<NewsItem>> = BTreeMap::new();

    for item in items {
      let category_key = item.id / category_unit;

      groups.entry(category_key).or_default().push(item);
    }

    // 各カテゴリ内をシャッフル
    let mut queues: Vec<VecDeque<NewsItem>> = groups
      .into_values()
      .map(|mut group| {
        group.shuffle(&mut rng);
        VecDeque::from(group)
      })
      .collect();

    // カテゴリ順もシャッフル
    queues.shuffle(&mut rng);

    // ラウンドロビンで再構築
    let mut result = Vec::new();

    loop {
      let mut added = false;

      for queue in &mut queues {
        if let Some(item) = queue.pop_front() {
          result.push(item);
          added = true;
        }
      }

      if !added {
        break;
      }
    }

    self.news_items = result;
  }
}

#[async_trait]
impl NewsFetcher for LivedoorNewsFetcher {
  // ------------------------------------------
  // RSS取得・パース → news_items(bodyなし)を構築
  // ------------------------------------------
  async fn rss_feed(&mut self) -> AppResult<()> {
    info!("RSS取得開始");
    // configからこれらだけ先に編集化
    let limit = self.config.rss.feed_fetch_limit;
    let interval = self.config.rss.rss_fetch_interval_ms;

    // 要素数取得
    let items_len = self.rss_items.len();

    // 各RSS itemごとに処理を行う
    for (i, rss_item) in self.rss_items.iter().enumerate() {
      match fetch_single_rss(rss_item, limit, CATEGORY_UNIT * (i + 1)).await {
        Ok(items) => self.news_items.extend(items),
        Err(e) => {
          // 1カテゴリ失敗しても続行し、ログに残す
          logger::warn(
            "rss_feed",
            format!("[{}] RSS取得失敗: {e}", rss_item.category),
          );
          warn!("[{}] RSS取得失敗: {e}", rss_item.category);
        }
      }
      // 最後の要素はsleepしない
      if i < items_len - 1 {
        debug!(
          "[rss_feed] [{}] 今から{}ms待機",
          rss_item.category, interval
        );
        sleep(Duration::from_millis(interval as u64)).await;
        debug!("[rss_feed] [{}] 待機終了", rss_item.category);
      }
    }

    if self.news_items.is_empty() {
      return Err(AppError::RSSFeed(
        "全カテゴリのRSS取得に失敗しました".to_string(),
      ));
    }

    info!("RSS取得完了");
    Ok(())
  }

  // ----------------------------------------
  // news_itemsのURLからHTML取得 → bodyを埋める
  // ----------------------------------------
  // rss_itemsからニュース本文を取得してnews_itemsを構築する
  async fn fetch_news(&mut self) -> AppResult<()> {
    info!("ニュース本文取得開始");
    // 本文取得インターバル
    let interval = self.config.rss.body_fetch_interval_ms;

    // タイトルセレクタ
    let title_selector = Selector::parse(LIVEDOOR_TITLE_SELECTOR)
      .map_err(|e| AppError::ArticleFeed(format!("セレクタのパース失敗: {e:?}")))?;
    // 本文セレクタ
    let body_selector = Selector::parse(LIVEDOOR_BODY_SELECTOR)
      .map_err(|e| AppError::ArticleFeed(format!("セレクタのパース失敗: {e:?}")))?;

    // 要素数取得
    let items_len = self.news_items.len();

    // 取得順をいい感じにするようなソート
    // カテゴリをバラけさせる
    self.spread_by_id_category(CATEGORY_UNIT);
    info!("{:#?}", self.news_items);
    // それぞれ実行する
    for (i, item) in self.news_items.iter_mut().enumerate() {
      info!("ID:{}の本文を取得開始", item.id);
      match fetch_single_body(&item.url, &title_selector, &body_selector).await {
        Ok((title, body)) => {
          if !title.is_empty() {
            item.title = title; // 空でなければ上書き
          }
          item.body = Some(body);
        }
        Err(e) => {
          // 1記事失敗しても続行
          logger::warn("fetch_news", format!("[id:{}] 本文取得失敗: {e}", item.id));
          warn!("[fetch_news] [id:{}] 本文取得失敗: {e}", item.id)
        }
      }
      // 最後の要素はsleepしない
      if i < items_len - 1 {
        debug!("[fetch_news] [id:{}] {}ms待機", item.id, interval);
        sleep(Duration::from_millis(interval as u64)).await;
        debug!("[fetch_news] [id:{}] 待機終了", item.id);
      }
    }

    // body: None の記事をフィルタして除外
    let before_len = self.news_items.len();
    self.news_items.retain(|item| {
      if item.body.is_none() {
        logger::warn(
          "fetch_news",
          format!("[id:{}] bodyがNoneのため除外", item.id),
        );
        false
      } else {
        true
      }
    });
    let after_len = self.news_items.len();

    if before_len != after_len {
      info!(
        "[fetch_news] body取得失敗により {}件除外 (残り{}件)",
        before_len - after_len,
        after_len
      );
    }

    if self.news_items.is_empty() {
      return Err(AppError::ArticleFeed(
        "全記事の本文取得に失敗しました".to_string(),
      ));
    }

    // ソートを元に戻す(ID順にソート)
    self.news_items.sort_by_key(|item| item.id);

    info!("ニュース本文取得完了");
    Ok(())
  }

  // ---------------------------------------
  // IDフィルタでnews_itemsを絞り込む
  // ---------------------------------------
  fn extract_news_items(&mut self, ids: Vec<usize>) -> AppResult<()> {
    info!("IDフィルタ開始");
    let extracted: Vec<NewsItem> = ids
      .iter()
      .filter_map(
        |&id| match self.news_items.iter().find(|item| item.id == id) {
          Some(item) => Some(item.clone()),
          None => {
            logger::warn(
              "extract_news_items",
              format!("id:{id} がnews_itemsに存在しません"),
            );
            None
          }
        },
      )
      .collect();

    if extracted.len() == 0 {
      return Err(AppError::RSSFeed(
        "IDフィルタでitemが0個になりました".to_string(),
      ));
    }
    self.news_items = extracted;
    info!("IDフィルタ完了");
    Ok(())
  }
}

// ---------------------------------------------------------------
// 内部ユーティリティ
// ---------------------------------------------------------------
// RSS 1つを取得する関数
async fn fetch_single_rss(
  rss_item: &RSSItem,
  limit: usize,
  id_start: usize,
) -> AppResult<Vec<NewsItem>> {
  let bytes = http_client::http()
    .get(rss_item.rss_url)
    .send()
    .await
    .map_err(|e| AppError::RSSFeed(format!("RSSリクエスト失敗: {e}")))?
    .bytes()
    .await
    .map_err(|e| AppError::RSSFeed(format!("RSSレスポンス読み込み失敗: {e}")))?;

  let channel = rss::Channel::read_from(&bytes[..])
    .map_err(|e| AppError::RSSFeed(format!("RSSパース失敗: {e}")))?;

  let items = channel
    .items()
    .iter()
    .enumerate()
    .take(limit)
    .filter_map(|(i, item)| {
      // titleかlinkがNoneの記事は読み飛ばす
      let title = item.title()?;
      let url = item.link()?;

      // HTMLエンティティとタグを除去
      let clean_title = ammonia::clean_text(title);
      let clean_title = html_escape::decode_html_entities(&clean_title).into_owned();

      Some(NewsItem {
        id: id_start + i * ID_INCREMENT,
        category: rss_item.category.to_string(),
        title: clean_title,
        url: url.to_string(),
        body: None,
      })
    })
    .collect();

  Ok(items)
}

async fn fetch_single_body(
  url: &str,
  title_selector: &Selector,
  body_selector: &Selector,
) -> AppResult<(String, String)> {
  let html = http_client::http()
    .get(url)
    .send()
    .await
    .map_err(|e| AppError::ArticleFeed(format!("本文リクエスト失敗: {e}")))?
    .text()
    .await
    .map_err(|e| AppError::ArticleFeed(format!("本文レスポンス読み込み失敗: {e}")))?;

  let document = Html::parse_document(&html);

  // セレクタで本文要素を取得しテキストだけ抽出
  // タイトル
  let title = document
    .select(title_selector)
    .next()
    .map(|el| el.text().collect::<Vec<_>>().join(""))
    .unwrap_or_default();
  // 本文
  let body = document
    .select(body_selector)
    .next()
    .map(|el| el.text().collect::<Vec<_>>().join(""))
    .unwrap_or_default();

  // 広告をなくすための正規表現
  // コンパイルを高速化するため、正規表現のパターンを一度だけ生成
  static RE_AD: OnceLock<Regex> = OnceLock::new();
  let re_ad = RE_AD.get_or_init(|| {
    // 「googletag.cmd.push( ... );」の形にマッチする正規表現
    Regex::new(GOOGLE_AD_REGEX).unwrap()
  });

  // 空白行・前後の空白を整理
  let clean_title = title
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join(" "); // タイトルは1行なのでスペース結合
  let clean_body = body
    .lines()
    .map(str::trim)
    .filter(|l| !l.is_empty())
    .collect::<Vec<_>>()
    .join("\n");

  // Googleの広告部分を除去
  let clean_body = re_ad.replace_all(&clean_body, "").to_string();

  // 全角スペース（\u{3000}）を半角スペースに置き換える
  let clean_title = clean_title.replace('\u{3000}', " ");
  let clean_body = clean_body.replace('\u{3000}', " ");

  // 「写真拡大」という文字列が写真の下にあるため、それを消し去る
  let clean_body = clean_body.replace("写真拡大\n", "");

  if clean_body.is_empty() {
    return Err(AppError::ArticleFeed("本文が空でした".to_string()));
  }

  Ok((clean_title, clean_body))
}

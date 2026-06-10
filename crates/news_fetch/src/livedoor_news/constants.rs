/*
crates/news_fetch/src/livedoor_news/constants.rs
ライブドアニュースの定数を定義する
*/

use shared::RSSItem;

// ニュースのパースに使うやつ
// fetch_newsで使うセレクタ
// タイトルのHTMLセレクタ
pub const LIVEDOOR_TITLE_SELECTOR: &str = "h1.articleTtl";
// 本文のHTMLセレクタ
pub const LIVEDOOR_BODY_SELECTOR: &str = "div.articleBody";

// Googleの広告を除去するための正規表現
pub const GOOGLE_AD_REGEX: &str = r"(?s)googletag\.cmd\.push\(function\(\)\s*\{.*?\}\);";

// カテゴリの分割単位(千の位をカテゴリ判別するなど)
pub const CATEGORY_UNIT: usize = 1000;

// idを増やす間隔(LLMの検証用も兼ねて)
pub const ID_INCREMENT: usize = 10;

// 取得するRSSたち
pub const LIVEDOOR_NEWS_RSS: &[RSSItem] = &[
  RSSItem {
    category: "主要",
    rss_url: "https://news.livedoor.com/topics/rss/top.xml",
  },
  RSSItem {
    category: "国内",
    rss_url: "https://news.livedoor.com/topics/rss/dom.xml",
  },
  RSSItem {
    category: "海外",
    rss_url: "https://news.livedoor.com/topics/rss/int.xml",
  },
  RSSItem {
    category: "IT・経済",
    rss_url: "https://news.livedoor.com/topics/rss/eco.xml",
  },
  RSSItem {
    category: "芸能",
    rss_url: "https://news.livedoor.com/topics/rss/ent.xml",
  },
  RSSItem {
    category: "海外",
    rss_url: "https://news.livedoor.com/topics/rss/spo.xml",
  },
];

# DiscordLLMNews 設計書
## 1. 概要
### 1.1 本プロジェクトについて
DiscordLLMNews は、ニュースサイトのRSSを取得し、LLMに選定・要約をしてもらい、Discordに通知するシステムである。
ニュースサイトにはライブドアニュースを採用する。
監視処理は単発実行型であり、定期実行は cron / supercronic など外部スケジューラに委ねる。

本システムでは、より面白い記事の送信を目指し、LLMリクエストを選定フェーズと要約フェーズに分け、計3回のLLMリクエストで処理を行う。

### 1.2 採用言語
Rustを採用する

## 設計方針
### 2.1 責務分離
本システムは以下の 4 層で考える。
* **http**
  対象サイトからRSSを取得・パースを行う。
  また、ニュース本文の取得も行う。
* **llm**
  タイトルや本文情報から使用するニュースを選定、要約をしてもらい、送信内容などをjsonに記入する。
* **infra**
  外部 I/O、ファイル保存、ログ出力などを扱う。
* **notifier**
  Discord Webhook 送信を扱う。

### 2.2 設計の基本姿勢
* panic を避け、Result ベースで伝播する
* 外部要因による失敗を前提にする(LLMの形式ミスなど)
* 後から対象サイトがを変更しても壊れにくい構造にする

---
## 3. システム構成
### 3.1 実行方式
```text
Docker 本番コンテナ起動
   ↓
cron / supercronic
   ↓
Rust バイナリ起動(ニュース要約)
   ↓
jsonに内容とログを保存

cron / supercronic
   ↓
Rust バイナリ起動(通知)
   ↓
Discord 通知
   ↓
終了
```

### 3.2 処理モデル
* 常駐アプリではなく、**1回の起動で 1 回分の監視を行う Batch 型**
* スケジュール管理は外部に委譲する
* 同一処理を繰り返し安全に実行できるようにする
* feed と send は異なる役割を持つ独立したプロセスであり、同時起動の可能性があることを前提とする
* 多重起動防止はcronのスケジュール設計で担保する

### 3.3 LLMリクエスト
本プロジェクトでは合計で3回のLLMリクエストを行う。
```
1. 選定フェーズ(選出)
  タイトル情報のみを渡し、各カテゴリごとにn個の面白そうな記事を選んでもらう

2. 選定フェーズ(決定)
  タイトル情報と本文情報を渡し、各カテゴリn個の中から1番面白そうな記事を選んでもらう

3. 要約フェーズ
  タイトル情報と本文情報を渡し、要約と豆知識生成など、Discordにそのまま送れる文章を作成してもらう
```
LLMレスポンスは**serdeで構造体へ変換できた場合のみ採用**とする。
失敗したらリトライ処理を実行する。(指数バックオフやモデルの変更)


## 4. Workspace 設計
### 4.1 全体構造
親クレート `DiscordLLMNews` は workspace ルートのみとし、実装は子 crate に分割する。
```text
DiscordLLMNews/
└──crates/
  ├── app/        # 全体実行用バイナリ
  ├── kernel/     # 実行制御・ユースケース起点
  ├── shared/     # 共通型・共通ユーティリティ
  ├── config/     # 設定ファイル読込
  ├── notifier/   # Discord Webhook 送信
  ├── infra/      # 外部I/O、ファイル、日付取得など
  ├── logger/     # 通知用ログの構造を定義
  ├── news_fetch/ # RSSの取得とニュース本文の取得
  └── llm/        # LLMへのリクエストを定義
```

### 4.2 各 crate の責務
| crate        | 責務                                         |
| ------------ | -------------------------------------------- |
| `app`        | CLIのエントリポイントやロガーの起動など |
| `kernel`     | 監視フローの起動、処理順制御、失敗時の扱い |
| `shared`     | 全 crate で共有する型、エラー型など|
| `config`     | `.config/app.toml` と `.config/.env` と各プロンプトの読込|
| `notifier`   | Webhook 送信、通知メッセージ整形 |
| `infra`      | ファイル保存、state 読込書込、ログ初期化 |
| `logger`     | 実行ログをメモリに蓄積し、JSONL形式での書き出しを提供する |
| `news_fetch` | RSSの取得とニュース本文の取得 |
| `llm`        | LLMへのリクエスト |

### 4.3 推奨 workspace root
```toml
[workspace]
resolver = "3"
members = ["crates/*"]
```

---
## 5. ディレクトリ構成案
```text
DiscordLLMNews/
├── .dockerignore
├── .gitignore
├── Cargo.toml
├── compose.yaml
├── generate_tree_ver2.py
├── Makefile
├── rustfmt.toml
├── .docs/
├── crates/
├── .docker/
│   ├── dev/
│   │   └── Dockerfile
│   └── prod/
│       ├── crontab
│       ├── Dockerfile
│       └── entrypoint.sh
└── .config/
    ├── dev/
    │   ├── .env
    │   ├── app.toml
    │   └── prompts/
    │       ├── select.md
    │       └── summarize.md
    └── prod/
        ├── .env
        ├── app.toml
        └── prompts/
            ├── select.md
            └── summarize.md
```

---
## 6. 処理フロー
### 6.1 ニュース要約
```text
起動
↓
設定読込
↓
status:runningを書き込む
notification_logを初期化
↓
RSS取得
↓
LLMに1回目の選定リクエスト
↓
ニュース本文取得
↓
LLMに2回目の選定リクエスト
↓
LLMに要約リクエスト
↓
全処理成功 → news_summaryを status: ready に更新。notification_logも保存
処理失敗 → news_summaryを status: failed に更新。notification_logも保存
↓
process_historyを更新
↓
終了
```

### 6.2 通知
```text
起動
↓
設定読込
↓
通知内容とログ内容を読込
↓
状態:running → started_atを確認し、hang判定未満なら30秒ごとに再読込を繰り返す
状態:running + hang判定経過 → hang扱いでエラー通知して終了
状態:sent → 通知済みをログ通知して終了
状態:failed → notification_logをDiscordに通知して終了
状態:ready → Discord通知 → news_summaryを status: sent に更新。
↓
process_historyを更新
↓
終了
```

---
## 7. 状態管理
### 7.1 news_summary の保存内容
news_summaryには要約されたニュース(送信内容)を保存する。
```json
// running時
{
  "status": "running",
  "started_at": "2026-06-01T22:59:00+09:00"
}

// ready時
{
  "status": "ready",
  "prepared_at": "2026-06-01T23:00:00+09:00",
  "message_body": "...",
}

// failed時
{
  "status": "failed",
  "started_at": "2026-06-01T22:59:00+09:00",
  "error_summary": "LLMリクエスト失敗: リトライ上限到達"
}

// sent時
{
  "status": "sent",
  "sent_at": "2026-06-01T22:59:00+09:00",
  "prepared_at": "2026-06-01T23:00:00+09:00",
  "message_body": "...",
}

```
statusフィールド値でsend側が分岐する。  
hang判定は30分を想定している。
| status  | 実行内容 |
| ------- | ------- |
| ready   | 通常フロー |
| running | started_atを確認し、hang判定未満ならn秒ごとに再読込を繰り返す |
| running + hang判定分経過 | hang扱いでログ通知 |
| failed  | ログだけdiscord通知 |
| sent   | 既に通知済みであり、内容が更新されていないことをエラーログ通知 |

Rustでは serde の #[serde(tag = "status")] を用いたenumで表現し、statusごとにフィールドを厳密に型付けする



### 7.2 notification_log の保存内容
notification_logには通知用のログを保存する。
jsonl形式を採用する。  
基本的にエラーやWARNのみを通知する。
```json
{"logged_at":"2026-06-01T23:01:00+09:00","level":"ERROR","message":"LLM選定フェーズ失敗: retry 1/3"}
{"logged_at":"2026-06-01T23:00:00+09:00","level":"ERROR","message":"Jsonパースエラー"}
```

---
## 8. 永続化設計
### 8.1 ファイル方針
* state は小さな JSON として保存
* 実行履歴は JSONL に残す
* 人間向けログは別途ファイルに残す

### 8.2 保存先候補
```text
data/news_summary.json      # 送信内容
data/notification_log.jsonl # 通知用ログ
data/process_history.jsonl  # 実行ログ
./app.log                   # 1回ごとの詳細ログ
```

### 8.3 process_history.jsonl
実行履歴を残す。
```json
{"process":"news","started_at":"...","finished_at":"...","success":true}
{"process":"send","started_at":"...","finished_at":"...","success":true}
{"process":"news","started_at":"...","finished_at":"...","success":false,"error_stage":"feed"}
{"process":"send","started_at":"...","finished_at":"...","success":true}
```

### app.logについて
app.logには、実行にtracingで作られた全ログが入る。
このファイルは1実行ごとに更新される。
Discordにエラー通知が来た時などに、その詳細内容を確認する用である。

---
## 9. Discord 通知設計
### 9.1 通知内容
```text
# 📅 20xx年x月xx日(X)のニュース

## カテゴリ1
### タイトル1
要約されたニュース本文

### タイトル2
要約されたニュース本文

...カテゴリごとに表示...

## 本日の豆知識！
今日の豆知識

## 締めの一言
締めの一言
```

### 9.2 送信単位
* 原則は送信内容の文字列をそのまま送る
* 送信制限文字数を超えていた場合は複数メッセージに分割する

---
## 10. エラー設計
### 10.1 エラー分類

```rust
enum AppError {
    Config,
    RSSFeed,
    ArticleFeed,
    LLMRequest,
    JsonParse,
    Notifier,
    Storage,
}
```
### 10.2 方針
* panic を原則禁止
* 失敗地点と理由をログに残す
* 送信用ログは別で定義する

---
## 11. ログ設計
### 11.1 ログレベル
* INFO
* WARN
* ERROR

### 11.2 必ず記録するもの
* 起動
* 更新有無
* 記録内容の更新結果
* Discord 送信結果
* 成功終了or失敗終了
* 失敗時の原因

---
## 12. Docker 設計
### 12.1 方針
Docker は以下の 2 系統で運用する。
* **開発用コンテナ**
  Rust の開発、テスト、`cargo watch`
* **本番用コンテナ**
  Rust バイナリを常駐させ、supercronic で定期実行

### 12.2 開発用
* Rust
* cargo-watch

### 12.3 本番用
* Rust バイナリ
* cron 実行環境

### 12.4 運用イメージ
```text
compose.yaml
  ├─ discord_llm_news_dev
  └─ discord_llm_news
```

### 12.5 cron 設計
ニュース要約と送信を分けて定義
```cron
50 7 * * * /app/discord_llm_news -- feed
0 8 * * * /app/discord_llm_news -- send
```


---
## 13. CLI 設計
### 13.1 app crate
`app` は全体実行用のバイナリクレートとする。

### 13.2 想定コマンド
* `feed`
* `send`

例:
```bash
cargo run -p app -- feed
cargo run -p app -- send
```
ただし実運用では Docker 内でバイナリを直接実行する。

---

## 14. 将来拡張
* 複数サイト監視
* Slack等への通知対応

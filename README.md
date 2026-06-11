# DiscordLLMNews

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-compose-blue.svg)](https://www.docker.com/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

ニュースサイトのRSSを取得し、LLMによる記事選定・要約・豆知識生成を行い、Discordへ毎日決まった時刻にニュースを配信するシステムです。  
本プロジェクトは **DiscordLLMNews Ver2 の作り直し版** です。  
Ver2の運用を通じて見えてきた課題を整理し、Docker化、Workspace化、状態管理の見直し、定刻送信対応などを行い、より保守しやすく拡張しやすい構成を目指して再設計しました。

---
## なぜ作り直したのか
Ver2は実際に運用できる状態まで完成していましたが、以下のような課題がありました。
* 開発環境から本番運用までの導線が整備されていない
* 開発環境と本番環境の差異が大きい
* ニュース生成と送信処理が密結合になっている
* ニュースサイト変更時の影響範囲が広い
* スケジュール管理をコンテナへ集約したい
* 状態管理やエラー処理をさらに改善したい
* 今後の機能追加を考えると構造を整理したい

そのため、小規模な改修ではなく、プロジェクト単位で再設計することにしました。

---
## Ver2からの主な変更点
### Docker化
開発環境と本番環境を分離し、cron / supercronic を含めた実行環境をコンテナ内へ集約しました。

### Cargo Workspace化
責務ごとにクレートを分離し、保守性と拡張性を向上させました。

### 定刻送信対応
ニュース生成処理とDiscord送信処理を分離しました。 
これにより、
```text
07:50 ニュース生成
08:00 Discord送信
```
のような運用が可能となり、「8時に処理開始」ではなく「8時に送信」を実現しています。

### 豆知識履歴の活用
過去の豆知識履歴をプロンプトへ渡し、重複を避けながら話題の連続性を持たせられるよう改善しました。

### ニュースサイトの差し替えを容易化
ニュース取得処理を分離し、対象サイトの追加や変更を行いやすい設計へ見直しました。

### 設定周りの整理
`.env` や `app.toml` の構成を見直し、設定変更や運用を容易にしました。

---
## 主な機能
* RSSからニュースを取得
* LLMによる記事選定
* LLMによるニュース要約
* LLMによる豆知識生成
* Discordへの自動配信
* ニュース生成と送信処理の分離
* JSONベースの状態管理
* エラーログのDiscord通知
* Dockerによる本番運用
* Cargo Workspaceによる責務分離

---
## 通知サンプル
```text
# 📅 本日のニュース
## 🇯🇵 国内
...

## 🌎 海外
...

## ⚽ スポーツ
...

## 💡 本日の豆知識
...

🍀 締めの一言
今日があなたにとって素敵な一日になりますように！
```

---
## 技術スタック
### Rust
* serde
* chrono
* reqwest
* tracing
* thiserror

### インフラ
* Docker
* Docker Compose
* cron / supercronic

### LLM
* Gemini (利用モデルは app.toml から変更可能)

---
## フォルダ構成
```text
DiscordLLMNews/
├── Cargo.toml
├── compose.yaml
├── Makefile
├── .docker/
├── .config/
└── crates/
    ├── app/        # エントリポイント
    ├── kernel/     # 実行制御・ユースケース起点
    ├── shared/     # 共通型
    ├── config/     # 設定読込
    ├── notifier/   # Discord通知
    ├── infra/      # ファイルI/O等
    ├── logger/     # 通知用ログの構造を管理
    ├── news_fetch/ # ニュース取得
    └── llm/        # LLM関連

```

---
## クイックスタート
### 1. 環境変数の設定
まず環境変数ファイルを作成します。
```bash
cp .config/prod/.env.example .config/prod/.env
```

`.env` を編集し、必要な情報を設定してください。
例:
```env
# 通知用webhook
APP__DISCORD__NOTIFY_WEBHOOK="
  https://discord.com/api/webhooks/...,
  https://discord.com/api/webhooks/...
  "

# ログ通知用webhook
APP__DISCORD__LOGS_WEBHOOK="
  https://discord.com/api/webhooks/...,
  https://discord.com/api/webhooks/...
  "
```

---
### 2. デプロイ
プロジェクトルートで以下を実行します。
```bash
make deploy
```
内部では以下が実行されます。
```bash
docker compose up -d --build
```
これにより、
* Rustアプリケーションのビルド
* 本番コンテナの起動
* cron / supercronic の起動  
が自動で行われます。

---
## 設定変更
### cron設定
実行スケジュールは `.docker/prod/crontab` で変更できます。
```cron
50 7 * * * /app/discord_llm_news -- feed
0 8 * * * /app/discord_llm_news -- send
```
変更後は再度デプロイを行ってください。
```bash
make deploy
```

---
### app.toml
モデル選択など、各種設定は
```text
.config/prod/app.toml
```
から変更できます。  
設定ファイルはコンテナへVolumeマウントされるため、多くの場合は再ビルド不要で反映できます。

---
## 設計上の特徴
### 3段階のLLM処理
本プロジェクトでは以下の3段階でLLMを利用します。
```text
1. タイトルのみで候補記事を選定
↓
2. 本文を含めて最終記事を選定
↓
3. 要約と豆知識を生成
```
これにより、
* トークン消費量の削減
* 選定精度の向上
* 要約品質の安定化  
を狙っています。

### Batch型アプリケーション
本システムは常駐型ではなく、
```text
起動
↓
処理
↓
終了
```
を基本とするBatch型です。  
スケジュール管理はcron / supercronicへ委譲しています。

### LLMフェイルオーバー
複数のLLMモデルを優先順位付きで設定できます。
```text
例:
Gemini 2.5 Pro
↓失敗
Gemini 2.5 Flash
↓失敗
Gemini 2.0 Flash
```

### 状態管理
処理状態はJSONファイルで管理し、
```text
running → ready → sent
```
の状態遷移によって
ニュース生成と送信処理を疎結合にしています。


---
## ライセンス
このプロジェクトは MIT License の下で公開されています。

---
## 余談
このREADMEはLLMに書かせ、それをちょっと編集しただけです。
あとちょっとだけこのREADMEに書いてる内容が古いです。

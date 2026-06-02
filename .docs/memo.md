# .docs/memo.md
# メモ
- Arcで全体共有できる。HTTPクライアントはこれをしたい。

- どうしても無理だからAPIキーもベクトルで持つことにした。  
  今後、`.env`で定義するものは配列になるかもしれない。  
  せっかくだしapiキーもフォールバックにする？

- プログラムが開始した段階で、dataフォルダがあるかを確認。dataフォルダがなかったら作成する処理を追加

- kernelは構造体として持ち、次のように定義しようかなと悩んでいる。
```rust
// crates/kernel/src/lib.rs
pub struct Kernel {
    config: Config,
    started_at: DateTime<FixedOffset>,
}

impl Kernel {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            started_at: now_jst(), // インスタンス化時点の時刻
        }
    }

    pub async fn feed(&self) -> AppResult<()> {
        // started_at を infra に渡す
        storage::init_data_dir(self.started_at).await?;
        // ...以降のfeed処理
    }

    pub async fn send(&self) -> AppResult<()> {
        // ファイルがなければStorage エラー
        storage::read_news_summary().await?;
        // ...以降のsend処理
    }
}
```


---
# 開発日記
## 2026年06月02日(朝)
プロジェクト始動！
### やったこと
- Docker関連の整備
- workspaceの整備
- 設計書の作成
- READMEの作成

準備段階が終わり、いつでも開発を進められる状況になった。

## 2026年06月02日終わり
- エラー型の作成
- 共通型の作成
- configクレートと、config設定の作成
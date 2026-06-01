# セットアップ備忘録
## Rust環境
workspaceを採用する

### Workspaceを作成する
1. ルートディレクトリを作成する  
  → 今回はプロジェクトルートをルートとするため、スキップ
2. クレート用ディレクトリを作成する  
```bash
mkdir crates
```
3. ルートにCargo.tomlを作成する  
以下を作成する
```toml
[workspace]
resolver = "3"
members = ["crates/*"]

[workspace.dependencies]
```

### 子クレートを作成する
`--vcs none`を付けると、`.git/`および`gitignore`が生成されなくなる。
```bash
# クレート用ディレクトリに移動
cd crates

# バイナリクレート
cargo new <クレート名> --bin --vcs none

# ライブラリクレート
cargo new <クレート名> --lib --vcs none
```

### 子クレートのCargo.toml
`dependencies`の書き方がworkspace用になる。
```
[dependencies]
# 同じworkspace内の別クレート
aaa = { path = "../aaa" }
bbb = { path = "../bbb" }
ccc= { path = "../ccc" }

# workspace共通クレート
ddd = { workspace = true }
eee = { workspace = true }

# 固有クレート
fff = "0.8"
```

### formatの指定
`rustfmt.toml`を配置する。  
以下を記入する
```toml
tab_spaces = 2
```

---
## Gitリポジトリ
プロジェクトルートをGitリポジトリにする。
```bash
git init
```
初期状態ではブランチ名が`master`であるため、変更する。
```bash
git branch -m main
```
また、gitignoreを配置。必要なものを記入する。

---
## Docker環境
開発環境と本番環境に分けて実装する。
詳しくは`architecture.md`に示す。
また、`.dockerignore`を配置。`target`などを除外するとビルドが速くなる。
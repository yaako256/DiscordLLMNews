


# セットアップ備忘録
## Rust環境
workspaceを採用する

---
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
---
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

---
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

## Gitリポジトリ
プロジェクトルートをGitリポジトリにする。
```bash
git init
```

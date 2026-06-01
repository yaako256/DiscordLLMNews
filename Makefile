# Makefile
# メモ => PHONY: ファイルではないという指定(ファイルは更新されていないと実行されない): 命令である

# ==================================
# 設定・変数定義
# ==================================
.DEFAULT_GOAL := help

# 実行時の引数（未指定時はhelp）
CMD ?= help

# サービス名（compose.yamlと一致させる）
DEV_SERVICE  := yutai_watch_dev
PROD_SERVICE := yutai_watch


# ==================================
### 実行関連(Execution)
# ==================================
.PHONY: run run-prod

## 開発用の引数付き実行(開発コンテナ内) (例: make run CMD=monitor)
run: 
	cargo run -p app -- $(CMD)

## 本番用のバイナリを単発・引数付き実行(本番コンテナ内) (例: make run-prod CMD=monitor)
run-prod:
	/app/yutai_watch $(CMD)


# ==================================
### Docker関連(Docker Management)
# ==================================
.PHONY: dev stop-dev prod stop-prod down deploy build

## 開発用コンテナを起動
dev: 
	docker compose up $(DEV_SERVICE)

## 開発用コンテナを停止
stop-dev: 
	docker compose stop $(DEV_SERVICE)

## 本番用コンテナをバックグラウンド起動
prod:
	docker compose up -d $(PROD_SERVICE)

## 本番用コンテナを停止
stop-prod:
	docker compose stop $(PROD_SERVICE)

# コンテナ・ネットワークを停止・削除(共通)
#down:
#	docker compose down

## 完全本番デプロイ
# - dev停止
# - release build
# - container再作成
deploy:
	docker compose stop $(DEV_SERVICE)
	docker compose rm -f $(DEV_SERVICE)
	docker compose up -d --build --force-recreate $(PROD_SERVICE)

## Dockerイメージのビルドチェック(共通)
build: 
	docker compose build


.PHONY: logs devlogs shell prodshell ps stats stats-once reset
## 本番用コンテナのログをリアルタイム表示
logs: 
	docker compose logs -f $(PROD_SERVICE)

## 開発用コンテナのログをリアルタイム表示
devlogs:
	docker compose logs -f $(DEV_SERVICE)

## 開発用コンテナのシェル（bash）に入る
shell:
	docker compose exec -it $(DEV_SERVICE) bash

## 本番用コンテナのシェル（sh）に入る
prodshell:
	docker compose exec -it $(PROD_SERVICE) sh

## コンテナの起動状態を確認
ps:
	docker compose ps

## 起動中のコンテナのメモリ・CPU使用率をリアルタイム表示
stats:
	docker compose stats

## 【危険】完全リセット（コンテナ、イメージ、ボリューム、ネットワークを全削除）
reset:
	docker compose down --rmi all --volumes --remove-orphans


# ==================================
### Rust品質管理(Rust Quality Control)
# ==================================
.PHONY: test

## ユニットテストの実行
test:
	cargo test


# ==================================
### その他 (Utilities)
# ==================================
.PHONY: tree tree-git chown help

## フォルダツリーを表示 (自作Pythonスクリプト実行)
tree:
	python3 ./generate_tree_ver2.py . 100 target .git

## カレントディレクトリ内の全ファイルに権限の付与
chown:
	sudo chown -R $(shell whoami):$(shell whoami) .

## このMakefileのヘルプメッセージを表示
# `#`が3つのものを検知し、グループ名を表示している
# `#`が2つのものを検知し、そのあとのkeyと組み合わせることでhelpを表示している
help:
	@awk '/^### / {print ""; printf "\033[1;35m%s\033[0m\n", substr($$0, 5); next} /^## / {desc=substr($$0, 4)} /^[a-zA-Z_-]+:/ {if (desc) {sub(/:.*/, "", $$1); printf "  \033[36m%-15s\033[0m %s\n", $$1, desc; desc=""}}' $(MAKEFILE_LIST)

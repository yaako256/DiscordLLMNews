# Makefile
# メモ => PHONY: ファイルではないという指定(ファイルは更新されていないと実行されない): 命令である

# ==================================
# 設定・変数定義
# ==================================
.DEFAULT_GOAL := help

# 実行時の引数（未指定時はhelp）
CMD ?= help

# compose名
DEV_COMPOSE := compose.dev.yaml
PROD_COMPOSE := compose.prod.yaml
SERVICE_NAME := app

# ==================================
### 実行関連(Execution)
# ==================================
.PHONY: run run-prod

## 開発用の引数付き実行(例: make run CMD=monitor)
# コンテナ内で行う
run: 
	cargo run -p app -- $(CMD)
#	cargo run -p app
	
## 本番用のバイナリを単発・引数付き実行(本番コンテナ内) (例: make run-prod CMD=monitor)
run-prod:
	/app/discord_llm_news $(CMD)

# ==================================
### Docker関連(Docker Management)
# ==================================
.PHONY: dev stop-dev prod stop-prod down deploy deploy-release build-dev build-prod

## 開発用コンテナを起動
dev:
	docker compose -f $(DEV_COMPOSE) up

## 開発用コンテナを停止
stop-dev: 
	docker compose -f $(DEV_COMPOSE) stop $(SERVICE_NAME)

## 本番用コンテナをバックグラウンド起動
prod:
	docker compose -f $(PROD_COMPOSE) up -d

## 本番用コンテナを停止
stop-prod:
	docker compose -f $(PROD_COMPOSE) stop $(SERVICE_NAME)

# コンテナ・ネットワークを停止・削除(共通)
#down:
#	docker compose down

## 本番デプロイ
deploy:
	docker compose -f $(PROD_COMPOSE) up -d --build --force-recreate

## 完全本番デプロイ
# - dev停止
# - release build
# - container再作成
deploy-release:
	docker compose -f $(DEV_COMPOSE) stop $(SERVICE_NAME)
	docker compose -f $(DEV_COMPOSE) rm -f $(SERVICE_NAME)
	docker compose -f $(PROD_COMPOSE) up -d --build --force-recreate

## 開発用Dockerイメージのビルドチェック
build-dev:
	docker compose -f $(DEV_COMPOSE) build

## 本番用Dockerイメージのビルドチェック
build-prod:
	docker compose -f $(PROD_COMPOSE) build

.PHONY: logs devlogs shell prodshell ps stats stats-once reset
## 本番用コンテナのログをリアルタイム表示
logs: 
	docker compose -f $(PROD_COMPOSE) logs -f $(SERVICE_NAME)

## 開発用コンテナのログをリアルタイム表示
devlogs:
	docker compose -f $(DEV_COMPOSE) logs -f $(SERVICE_NAME)

## 開発用コンテナのシェル（bash）に入る
shell:
	docker compose -f $(DEV_COMPOSE) exec $(SERVICE_NAME) bash

## 本番用コンテナのシェル（sh）に入る
prodshell:
	docker compose -f $(PROD_COMPOSE) exec $(SERVICE_NAME) sh

## コンテナの起動状態を確認
ps:
	docker compose -f $(DEV_COMPOSE) ps
	docker compose -f $(PROD_COMPOSE) ps

## 起動中のコンテナのメモリ・CPU使用率をリアルタイム表示
stats:
	docker stats

## 【危険】完全リセット（コンテナ、イメージ、ボリューム、ネットワークを全削除）
reset:
	docker compose -f $(DEV_COMPOSE) down --rmi all --volumes --remove-orphans
	docker compose -f $(PROD_COMPOSE) down --rmi all --volumes --remove-orphans

.PHONY: create-at
## 詳細な起動時間を表示
create-at:
	docker compose -f $(DEV_COMPOSE) ps --format "table {{.Name}}\t{{.CreatedAt}}"
	docker compose -f $(PROD_COMPOSE) ps --format "table {{.Name}}\t{{.CreatedAt}}"

# ==================================
### Rust品質管理(Rust Quality Control)
# ==================================
.PHONY: test toml

## ユニットテストの実行
test:
	cargo test

## クレートの依存関係のチェック
toml:
	cargo machete

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

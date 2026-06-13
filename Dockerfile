# Dockerfile

# ========================================
# Base Stage
# ========================================
FROM rust:1.88 AS base

WORKDIR /app


# 開発用コンテナ関連
# ========================================
# Dev Stage
# 開発用コンテナの起動に使用する
# ========================================
FROM base AS dev

RUN cargo install cargo-watch

CMD ["cargo", "watch", "-x", "check"]


# 本運用コンテナ関連
# ========================================
# Builder Stage
# ========================================
FROM base AS builder

# 依存キャッシュ最適化（重要）
COPY Cargo.toml Cargo.lock ./

# workspace構造をコピー
COPY crates ./crates

# release build（ここを実行バイナリに合わせる）
RUN cargo build --release -p app

# ========================================
# Prod Stage
# ========================================
FROM debian:bookworm-slim AS prod

# 必要ライブラリ
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    tzdata \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# タイムゾーン指定
ENV TZ=Asia/Tokyo

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone

WORKDIR /app

# =========================
# supercronic
# =========================
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
      ARCH="amd64"; \
    elif [ "$ARCH" = "aarch64" ]; then \
      ARCH="arm64"; \
    fi && \
    curl -fsSLO https://github.com/aptible/supercronic/releases/latest/download/supercronic-linux-${ARCH} && \
    mv supercronic-linux-${ARCH} /usr/local/bin/supercronic && \
    chmod +x /usr/local/bin/supercronic

# Rust binary
COPY --from=builder /app/target/release/app /app/discord_llm_new

# 周辺ディレクトリのコピー
COPY .docker/crontab /app/crontab
COPY .docker/entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
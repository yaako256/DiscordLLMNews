# .docker/prod/Dockerfile
# 本番用DockerFile

# =========================
# Build Stage
# =========================
FROM rust:1.88 AS builder

WORKDIR /app

# 依存キャッシュ最適化（重要）
COPY Cargo.toml Cargo.lock ./

# workspace構造をコピー
COPY crates ./crates

# release build（ここを実行バイナリに合わせる）
RUN cargo build --release -p app


# =========================
# Runtime Stage
# =========================
FROM debian:bookworm-slim

# 必要ライブラリ
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    tzdata \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# timezone
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

# =========================
# Rust binary
# =========================
COPY --from=builder /app/target/release/app /app/discord_llm_news

# 周辺ディレクトリのコピー
COPY .config /app/.config
COPY .docker/prod/crontab /app/crontab
COPY .docker/prod/entrypoint.sh /entrypoint.sh

RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
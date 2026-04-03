# ---- Stage 1: Builder ----
FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

# ---- Stage 2: Runtime ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/RAG_Server ./RAG_Server
COPY models ./models

EXPOSE 3001

CMD ["./RAG_Server"]

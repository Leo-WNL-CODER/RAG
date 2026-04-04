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
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/RAG_Server ./RAG_Server

# Download quantized BGE model (~110MB) at image build time — fits in 512MB free tier
RUN mkdir -p models && \
    curl -L \
    "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/main/onnx/model_quantized.onnx" \
    -o models/model.onnx

# Copy tokenizer from repo
COPY models/tokenizer.json ./models/tokenizer.json

EXPOSE 3001

CMD ["./RAG_Server"]

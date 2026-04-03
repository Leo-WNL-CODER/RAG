# ---- Stage 1: Builder ----
FROM rust:1.85-slim AS builder

# Install system dependencies needed to compile
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main to pre-compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source and build
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---- Stage 2: Runtime ----
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary
COPY --from=builder /app/target/release/RAG_Server ./RAG_Server

# Copy the ONNX model (required at runtime)
COPY models ./models

EXPOSE 3001

CMD ["./RAG_Server"]

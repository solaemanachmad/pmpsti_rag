# ── Stage 1: Builder ──────────────────────────────────────────────
# ubuntu:24.04 ships glibc 2.39, satisfying ort-sys prebuilt ONNX Runtime
# which requires __isoc23_strtol / __isoc23_strtoll (added in glibc 2.38)
FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    libopenblas-dev \
    cmake \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Install Rust 1.88 via rustup
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain 1.88.0 --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Cache dependencies first
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libopenblas0 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

WORKDIR /app
COPY --from=builder /app/target/release/pmpsti ./pmpsti

# Fastembed model cache directory
RUN mkdir -p /app/.cache && chown appuser:appuser /app/.cache

RUN chown appuser:appuser /app/pmpsti
USER appuser

EXPOSE 7860

ENV HOST=0.0.0.0
ENV PORT=7860
ENV FASTEMBED_CACHE_PATH=/app/.cache

CMD ["./pmpsti"]

# ── Stage 1: Builder ──────────────────────────────────────────────
# ubuntu:24.04 ships glibc 2.39
FROM ubuntu:24.04 AS builder

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
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
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Dependencies untuk Chromium yang di-download Playwright
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    fonts-liberation \
    libnss3 \
    libatk-bridge2.0-0 \
    libgtk-3-0 \
    libx11-xcb1 \
    libxcomposite1 \
    libxdamage1 \
    libxrandr2 \
    libgbm1 \
    libasound2t64 \
    libxshmfence1 \
    libdrm2 \
    libxext6 \
    libxfixes3 \
    libxcb1 \
    libx11-6 \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Playwright dan download Chromium-nya sendiri
RUN npm install -g playwright@1.49.0 && \
    playwright install chromium

WORKDIR /app
COPY --from=builder /app/target/release/pmpsti ./pmpsti

RUN chown -R ubuntu:ubuntu /app
USER ubuntu

EXPOSE 7860

ENV HOST=0.0.0.0
ENV PORT=7860

CMD ["./pmpsti"]

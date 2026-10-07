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

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    chromium-browser \
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
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Install Playwright tanpa download browser (pakai chromium-browser system)
RUN npm install -g playwright@1.49.0 && \
    PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm install -g playwright@1.49.0

WORKDIR /app
COPY --from=builder /app/target/release/pmpsti ./pmpsti

RUN chown -R ubuntu:ubuntu /app
USER ubuntu

ENV CHROMIUM_BIN=/usr/bin/chromium-browser
# Playwright pakai chromium system — tidak perlu download browser sendiri
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1

EXPOSE 7860

ENV HOST=0.0.0.0
ENV PORT=7860

CMD ["./pmpsti"]

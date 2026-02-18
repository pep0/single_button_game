# === Builder stage ===
FROM rust:1.85-bookworm AS builder

# Install Bevy's Linux system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libudev-dev \
    libasound2-dev \
    libx11-dev \
    libxi-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxinerama-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libvulkan-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Dependency caching: copy manifests and build with a stub main.rs
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -rf src

# Copy real source and rebuild (only our code recompiles)
COPY src/ src/
# Touch main.rs so cargo knows it changed
RUN touch src/main.rs && cargo build --release

# === Runtime stage ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libudev1 \
    libasound2 \
    libx11-6 \
    libxi6 \
    libxcursor1 \
    libxrandr2 \
    libxinerama1 \
    libwayland-client0 \
    libxkbcommon0 \
    libvulkan1 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/single_button_game /usr/local/bin/

ENTRYPOINT ["single_button_game"]

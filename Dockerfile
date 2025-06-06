# Use the official Rust image
FROM rust:slim-bookworm AS builder

# Create app directory
WORKDIR /usr/src/app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies for cache
RUN cargo build --release && rm -rf src target/release/deps/feed_crafter*

# Copy the source code
COPY . .

# Build the project
RUN cargo build --release

# Use a minimal image for running
FROM debian:bookworm-slim

# Install required packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        libssl3 \
        ca-certificates \
        curl && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Copy the binary and static files from the builder
COPY --from=builder /usr/src/app/target/release/feed-crafter /app/feed-crafter
COPY --from=builder /usr/src/app/static /app/static

# Set working directory
WORKDIR /app

# Run the binary
CMD ["./feed-crafter"]
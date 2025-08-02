# Multi-stage build for optimal size
FROM rust:1.82-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release --features cli-basic

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/kiren /usr/local/bin/kiren

# Create app user
RUN groupadd -r app && useradd -r -g app app
USER app

WORKDIR /app

# Default entrypoint
ENTRYPOINT ["kiren"]
CMD ["--help"]
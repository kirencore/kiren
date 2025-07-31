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
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/kiren /usr/local/bin/kiren

# Copy test server file
COPY --from=builder /app/test-server.js /app/test-server.js

# Create app user
RUN groupadd -r app && useradd -r -g app app
RUN chown -R app:app /app

USER app
WORKDIR /app

# Set Docker environment variable
ENV DOCKER_ENV=1

# Default port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/healthcheck || exit 1

# Default command
CMD ["kiren", "test-server.js"]
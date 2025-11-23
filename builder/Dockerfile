# Stage 1: Build the Rust application
FROM rust:1.83 as builder

WORKDIR /app

# Copy only cargo files first for better layer caching
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
COPY builder/Cargo.toml builder/
COPY builder/src/ builder/src/

# Build in release mode
RUN cargo build --release --bin builder

# Stage 2: Create the final runtime image
FROM python:3.11-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /app/target/release/builder /app/builder

# Copy RunPod handler (for serverless deployments)
COPY builder/handler.py /app/handler.py

# Install Python dependencies (for serverless deployments)
RUN pip install --no-cache-dir runpod requests

# Set environment variables for production
ENV PRODUCTION=true
ENV HOST=0.0.0.0

# Expose port 3000 for the ingest API
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run the builder directly (for constant pod deployments)
# For serverless, override this with: python3 -u /app/handler.py
CMD ["/app/builder"]

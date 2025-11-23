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

# Copy RunPod handler
COPY builder/handler.py /app/handler.py

# Install Python dependencies
RUN pip install --no-cache-dir runpod requests

# Expose port 3000 for the ingest API
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Run the RunPod handler which will start the builder
CMD ["python3", "-u", "/app/handler.py"]

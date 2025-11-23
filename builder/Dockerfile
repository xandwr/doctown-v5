# Stage 1: Build the Rust application
FROM rust:1.83 as builder

WORKDIR /app

COPY . .

RUN cargo build --release

# Stage 2: Create the final image
FROM python:3.9-slim

WORKDIR /app

COPY --from=builder /app/target/release/builder .
COPY handler.py .

RUN pip install runpod requests psutil

CMD ["python3", "-u", "handler.py"]

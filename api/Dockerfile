FROM --platform=linux/amd64 rust:latest AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM --platform=linux/amd64 debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api .
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data
CMD ["./api"]

FROM rust:1.95 AS builder
WORKDIR /app

# Dependencies separat cachen
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release
RUN rm src/main.rs

# Echter Code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./.sqlx
ARG GIT_SHA=unknown
ENV GIT_SHA=$GIT_SHA
RUN touch src/main.rs && cargo build --release

FROM debian:trixie-slim
WORKDIR /app
COPY --from=builder /app/target/release/api .
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /data
CMD ["./api"]

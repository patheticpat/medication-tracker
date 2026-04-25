FROM --platform=linux/amd64 rust:latest AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM --platform=linux/amd64 debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/medication-tracker-api .
RUN mkdir -p /data
CMD ["./medication-tracker-api"]

FROM rust:1.88 as builder
WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rust_gateway /usr/local/bin/rust_gateway
ENV RUST_LOG=info
EXPOSE 8000
CMD ["rust_gateway"]

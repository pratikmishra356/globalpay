FROM rust:1.78 as builder
WORKDIR /build
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y libpq5

COPY --from=builder /build/target/release/globalpay /app/globalpay
COPY --from=builder /build/Cargo.toml /app/Cargo.toml

# CMD ["/app/globalpay"]
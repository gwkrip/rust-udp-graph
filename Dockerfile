FROM rust:nightly-slim AS builder

WORKDIR /app

COPY Cargo.toml ./

RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && sed -i 's|path = "main.rs"|path = "src/main.rs"|' Cargo.toml \
    && cargo build --release \
    && rm -rf src \
    && sed -i 's|path = "src/main.rs"|path = "main.rs"|' Cargo.toml

COPY main.rs ./
COPY templates ./templates

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1001 appuser

WORKDIR /app

COPY --from=builder /app/target/release/rust_udp_graph ./rust_udp_graph
COPY --from=builder /app/templates ./templates

RUN chown -R appuser:appuser /app

USER appuser

EXPOSE 8080
EXPOSE 8125/udp

ENTRYPOINT ["./rust_udp_graph"]

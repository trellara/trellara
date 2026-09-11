FROM rust:1.98.0-bookworm AS builder

WORKDIR /workspace
COPY . .
RUN cargo build --release -p trellara-cli

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /workspace/target/release/trellara /usr/local/bin/trellara
COPY examples /etc/trellara/examples

ENTRYPOINT ["trellara"]

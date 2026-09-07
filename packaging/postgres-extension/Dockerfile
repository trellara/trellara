ARG RUST_VERSION=1.97.1
FROM rust:${RUST_VERSION}-bookworm AS builder

ARG PG_MAJOR=17
ARG CARGO_PGRX_VERSION=0.19.2
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_TARGET_DIR=/build/target

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates clang cmake curl gnupg libclang-dev libcurl4-openssl-dev make pkg-config \
       postgresql-common rpm ruby-dev build-essential \
    && install -d /usr/share/postgresql-common/pgdg \
    && curl -fsSL https://www.postgresql.org/media/keys/ACCC4CF8.asc \
       -o /usr/share/postgresql-common/pgdg/apt.postgresql.org.asc \
    && echo "deb [signed-by=/usr/share/postgresql-common/pgdg/apt.postgresql.org.asc] https://apt.postgresql.org/pub/repos/apt bookworm-pgdg main" \
       > /etc/apt/sources.list.d/pgdg.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
       "postgresql-${PG_MAJOR}" "postgresql-server-dev-${PG_MAJOR}" \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-pgrx --version "=${CARGO_PGRX_VERSION}" --locked \
    && gem install --no-document fpm -v 1.16.0
RUN cargo pgrx init "--pg${PG_MAJOR}" \
      "/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config"

WORKDIR /src
COPY . .

RUN cargo pgrx package \
      --manifest-path crates/trellara-pg-extension/Cargo.toml \
      --pg-config "/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config" \
      --out-dir /stage
RUN cargo build --release -p trellara-relay --bin trellara-native-relay \
    && install -Dm755 /build/target/release/trellara-native-relay \
       /stage/usr/bin/trellara-native-relay \
    && install -Dm644 packaging/postgres-extension/trellara-native-relay.service \
       /stage/usr/lib/systemd/system/trellara-native-relay.service \
    && install -Dm600 packaging/postgres-extension/native-relay.env \
       /stage/etc/trellara/native-relay.env
RUN TRELLARA_PG_CONFIG="/usr/lib/postgresql/${PG_MAJOR}/bin/pg_config" \
    TRELLARA_PACKAGE_ROOT=/stage \
    scripts/test-native-postgres-extension.sh
RUN packaging/postgres-extension/build-linux-packages.sh \
    "$PG_MAJOR" /stage /artifacts

FROM scratch AS artifacts
COPY --from=builder /artifacts /

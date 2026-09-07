#!/usr/bin/env bash
set -euo pipefail

pg_major=${1:?usage: build-linux-packages.sh PG_MAJOR STAGE_DIR OUT_DIR}
stage_dir=${2:?usage: build-linux-packages.sh PG_MAJOR STAGE_DIR OUT_DIR}
out_dir=${3:?usage: build-linux-packages.sh PG_MAJOR STAGE_DIR OUT_DIR}
workspace_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
version=$(sed -n '/^\[package\]/,/^\[/s/^version = "\([^"]*\)"/\1/p' \
  "$workspace_root/crates/trellara-pg-extension/Cargo.toml")
deb_architecture=$(dpkg --print-architecture)
case "$deb_architecture" in
  amd64) rpm_architecture=x86_64 ;;
  arm64) rpm_architecture=aarch64 ;;
  *) rpm_architecture=$deb_architecture ;;
esac
mkdir -p "$out_dir"

common=(
  --input-type dir
  --name "trellara-pg-extension-$pg_major"
  --version "$version"
  --iteration 1
  --license Apache-2.0
  --vendor Trellara
  --maintainer "Trellara <engineering@trellara.dev>"
  --description "Trellara native logical-decoding extension for PostgreSQL $pg_major"
  --url "https://github.com/trellara/trellara"
  --chdir "$stage_dir"
)

fpm "${common[@]}" --output-type deb --architecture "$deb_architecture" \
  --config-files /etc/trellara/native-relay.env \
  --depends "postgresql-$pg_major" \
  --package "$out_dir/trellara-pg-extension-${version}-pg${pg_major}_${deb_architecture}.deb" .
fpm "${common[@]}" --output-type rpm --architecture "$rpm_architecture" \
  --config-files /etc/trellara/native-relay.env \
  --depends "postgresql${pg_major}-server" \
  --package "$out_dir/trellara-pg-extension-${version}-pg${pg_major}.${rpm_architecture}.rpm" .
tar -C "$stage_dir" -czf \
  "$out_dir/trellara-pg-extension-${version}-pg${pg_major}-${deb_architecture}.tar.gz" .
(
  cd "$out_dir"
  sha256sum ./* >SHA256SUMS
)

# PostgreSQL extension packages

This directory builds installable DEB, RPM, and tar packages for the Trellara native extension and durable relay. Every package is compiled against one PostgreSQL server ABI; PG15, PG16, PG17, and PG18 artifacts are intentionally separate.

The packaged relay defaults to local-spool durability. For the Kafka end-to-end acknowledgement boundary, set `TRELLARA_NATIVE_RELAY_DURABILITY=kafka`, configure `TRELLARA_KAFKA_BOOTSTRAP_SERVERS`, and keep `TRELLARA_KAFKA_PROOF_PATH` on durable storage before enabling the service.

The build image runs the complete live harness before producing an artifact. That harness starts a real server, runs `CREATE EXTENSION`, exercises insert/update/delete/truncate and abort decoding, fills the bounded queue, injects a relay crash after `fsync` but before acknowledgement, proves the replication slot did not advance, restarts the relay, and crashes/restarts the background worker.

Build package sets from the repository root:

```console
docker build -f packaging/postgres-extension/Dockerfile \
  --build-arg PG_MAJOR=15 --output type=local,dest=dist/pg15 .
docker build -f packaging/postgres-extension/Dockerfile \
  --build-arg PG_MAJOR=16 --output type=local,dest=dist/pg16 .
docker build -f packaging/postgres-extension/Dockerfile \
  --build-arg PG_MAJOR=17 --output type=local,dest=dist/pg17 .
docker build -f packaging/postgres-extension/Dockerfile \
  --build-arg PG_MAJOR=18 --output type=local,dest=dist/pg18 .
```

Install exactly the package matching the server major. Set a long random secret in `/etc/trellara/native-relay.env`, keep that file readable only by root, then start the relay:

```console
sudo systemctl daemon-reload
sudo systemctl enable --now trellara-native-relay
```

Configure `shared_preload_libraries`, `output_plugin_libraries`, and the `trellara.*` GUCs documented in the extension README. Use `/run/trellara/native-relay.sock` and the same secret as the relay environment file, then restart PostgreSQL before running `CREATE EXTENSION`. Package installation does not enable the service or edit PostgreSQL configuration.

`SHA256SUMS` covers every artifact. Package publication should sign both the packages and checksum file in the release pipeline.

# Upgrade and rollback workflow

The native library is loaded into the postmaster, so replacing package files is not enough: PostgreSQL must restart before SQL migration is applied. Use the same sequence for an upgrade or an explicitly approved rollback.

1. Pause source writes or otherwise establish a maintenance boundary.
2. Confirm `trellara.runtime_status()` reports `queued_frames = 0` and retain the last durable LSN as release evidence.
3. Back up the database and the currently installed DEB/RPM. Rollback is a package reinstall plus a reverse extension migration; it is not an undo log.
4. Install the package matching the server major. Never install an artifact built for one PostgreSQL major into another.
5. Run `systemctl daemon-reload`, restart `trellara-native-relay`, then restart PostgreSQL. Verify the worker PID changed and `data_plane_ready = true`.
6. Apply the SQL path:

   ```console
   scripts/postgres-extension-version.sh upgrade 0.2.0 postgresql:///app
   ```

7. Resume writes and verify the replication slot advances only after durable relay frames.

For rollback, reinstall the previous package, restart PostgreSQL, and require an explicit operator acknowledgement:

```console
TRELLARA_ALLOW_ROLLBACK=1 \
  scripts/postgres-extension-version.sh rollback 0.1.0 postgresql:///app
```

The command fails closed when the queue is not empty or `pg_extension_update_paths` has no path between the two versions. Every release that changes SQL must ship both the forward migration and a tested reverse migration before rollback can be advertised for that version.

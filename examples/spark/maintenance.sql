-- Trellara Spark template: lake fan-in table maintenance.
-- Required parameters:
--   ${catalog}
--   ${namespace}
--   ${raw_cdc_table}
--   ${epochs_table}
--   ${epoch_partitions_table}
--   ${verification_table}
--   ${target_table}
--   ${epoch_id}
--   ${primary_key_column}
--   ${accept_complete_with_gaps}
--   ${unsafe_allow_non_consumable_epoch}
--   ${unsafe_override_reason}
--
-- Run after `trellara lake fanin verify` reports `match` for the maintained epoch.
-- The target table may be a Spark-derived current-state or SCD2 table. Run this
-- template once per derived table that should be compacted.
--
-- Safety defaults mirror `IcebergMaintenancePolicy::default`:
--   * bin-pack at least five files toward 256 MiB outputs;
--   * retain at least ten snapshots and seven days of history;
--   * remove only orphan files older than one day.
-- Run Trellara's Rust maintenance planner first in production. It binds cleanup
-- to a catalog snapshot, protects pending commit-intent files, and performs a
-- second object HEAD before deletion. These Spark calls are the executable
-- operator-side equivalent after that plan has been reviewed.

WITH verified_epoch AS (
  SELECT e.epoch_id
  FROM ${catalog}.${namespace}.${epochs_table} e
  JOIN ${catalog}.${namespace}.${verification_table} v
    ON e.epoch_id = v.epoch_id
  WHERE e.epoch_id = '${epoch_id}'
    AND (v.checksum_status = 'match' OR ${unsafe_allow_non_consumable_epoch} = true)
    AND (
      e.state = 'complete'
      OR (
        e.state = 'complete_with_gaps'
        AND e.policy = 'publish_with_gaps'
        AND ${accept_complete_with_gaps} = true
      )
      OR ${unsafe_allow_non_consumable_epoch} = true
    )
)
SELECT
  CASE
    WHEN count(*) = 1 THEN 'ready'
    ELSE raise_error('Trellara epoch is not consumable: verification must match and gaps must be explicitly accepted unless an unsafe override is recorded')
  END AS trellara_verified_epoch_ready
FROM verified_epoch;

CALL ${catalog}.system.rewrite_data_files(
  table => '${namespace}.${raw_cdc_table}',
  strategy => 'binpack',
  options => map('target-file-size-bytes', '268435456', 'min-input-files', '5')
);
CALL ${catalog}.system.rewrite_data_files(
  table => '${namespace}.${target_table}',
  strategy => 'binpack',
  options => map('target-file-size-bytes', '268435456', 'min-input-files', '5')
);
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}.${epochs_table}');
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}._trellara_epoch_sources');
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}._trellara_epoch_tables');
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}.${epoch_partitions_table}');
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}._trellara_quarantine');
CALL ${catalog}.system.rewrite_data_files(table => '${namespace}.${verification_table}');

CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}.${raw_cdc_table}',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}.${target_table}',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}.${epochs_table}',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}._trellara_epoch_sources',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}._trellara_epoch_tables',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}.${epoch_partitions_table}',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}._trellara_quarantine',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);
CALL ${catalog}.system.expire_snapshots(
  table => '${namespace}.${verification_table}',
  older_than => current_timestamp() - INTERVAL 7 DAYS,
  retain_last => 10
);

CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}.${raw_cdc_table}',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}.${target_table}',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}.${epochs_table}',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}._trellara_epoch_sources',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}._trellara_epoch_tables',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}.${epoch_partitions_table}',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}._trellara_quarantine',
  older_than => current_timestamp() - INTERVAL 1 DAY
);
CALL ${catalog}.system.remove_orphan_files(
  table => '${namespace}.${verification_table}',
  older_than => current_timestamp() - INTERVAL 1 DAY
);

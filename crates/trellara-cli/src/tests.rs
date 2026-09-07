use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use clap::CommandFactory;
use clap::Parser;
use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, DdlBarrierReleaseBlocker, DdlBarrierReleaseGate,
    DdlBarrierSinkEvidence, DdlBarrierSummary, PartitionWatermarkSummary, ReseedEvent,
    SnapshotHandoffEvent, SnapshotRun, SnapshotRunState, SnapshotTableProgress, TransactionKey,
    ValidationEvent,
};
use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};
#[cfg(feature = "kafka")]
use trellara_protocol::StrictChunkPlanConfig;
use trellara_protocol::{
    Operation, PartitionKeyChangePolicy as ProtocolPartitionKeyChangePolicy,
    PartitionNullKeyPolicy as ProtocolPartitionNullKeyPolicy, PartitionPlanConfig,
    PartitionRebalanceStatus, ReplicaIdentity, TransactionEnvelope,
};
use trellara_relay::RelayMode;
use trellara_sim::{
    FailurePoint, FleetFanInFailurePoint, QualificationFailurePoint, SnapshotFailurePoint,
    StrictChunkFailurePoint,
};
use trellara_stream::StreamPublisher;
use trellara_stream_local::{
    inspect_local_stream, LocalCursorInspection, LocalDurability, LocalPublisher,
    LocalStreamInspection, LocalTopicInspection,
};
use trellara_verify::PostgresRelationInspection;

mod partition_rebalance_render;
mod partition_watermark_render;
mod status_view_render;
mod tests_capture_contract;
mod tests_chaos_report;
mod tests_chaos_report_boundary_contract;
mod tests_chaos_report_pgoutput_contract;
mod tests_chaos_report_source_local_identity_contract;
mod tests_chaos_report_summary_contract;
mod tests_chaos_report_surface;
mod tests_cli_core_commands;
mod tests_cli_lake_commands;
mod tests_cli_proof_loop_commands;
mod tests_cli_run_commands;
mod tests_cli_schema_barrier_commands;
mod tests_cli_schema_commands;
mod tests_cli_status_commands;
mod tests_cli_stream_commands;
mod tests_command_execution;
mod tests_config_mapping;
mod tests_configuration;
mod tests_contract_partition_summary;
mod tests_contract_summary;
mod tests_correctness_report;
mod tests_correctness_report_partition;
mod tests_correctness_report_snapshot;
mod tests_correctness_report_source_safety;
mod tests_correctness_report_validation;
mod tests_ddl;
mod tests_diagnostics_failover_repair;
mod tests_diagnostics_repair;
mod tests_diagnostics_source_slot_repair;
mod tests_diagnostics_validation_repair;
mod tests_direct_source_safety;
mod tests_enterprise_review_surface;
mod tests_fleet_cli_commands;
mod tests_fleet_control_plane_surface;
mod tests_fleet_evidence_plan_surface;
mod tests_fleet_identity_surface;
mod tests_fleet_report_surface;
mod tests_fleet_scorecard_surface;
mod tests_fleet_surface;
mod tests_flow_status_surface;
mod tests_init_command_execution;
mod tests_init_config;
mod tests_lake_command_execution;
mod tests_lake_epoch_fanin_surface;
mod tests_lake_spark_surface;
mod tests_lake_surface;
mod tests_metrics;
mod tests_modularity;
mod tests_mvp_artifact_contracts;
mod tests_mvp_readiness_surface;
mod tests_native_worker_report;
mod tests_operator_control_commands;
mod tests_partitioned_config_mapping;
mod tests_pilot_cli_commands;
mod tests_pilot_command_surface;
mod tests_pilot_evidence_cli_commands;
mod tests_pilot_evidence_surface;
mod tests_pilot_guide_scorecard_surface;
mod tests_pilot_live_evidence_kit;
mod tests_pilot_package_artifact_contract;
mod tests_pilot_package_command_contract;
mod tests_pilot_package_content_contract;
mod tests_pilot_package_surface;
mod tests_quickstart_mvp;
mod tests_quickstart_readiness;
mod tests_run_verify_summary;
mod tests_snapshot;
mod tests_source_capture_parser;
mod tests_source_safety;
mod tests_source_safety_init;
mod tests_status_health_alerts;
mod tests_stream_local_config;
mod tests_stream_local_inspect_surface;
mod tests_stream_local_locate_surface;
mod tests_stream_local_reconstruct_surface;
mod tests_stream_local_redelivery_surface;
mod tests_stream_local_replay;
mod tests_stream_local_seek_surface;
mod tests_stream_recovery_actions;
mod tests_table_verify_summary;
mod tests_target_contract;
mod tests_transaction_inspect;
mod tests_transaction_inspect_commands;

mod fixtures;
use fixtures::*;

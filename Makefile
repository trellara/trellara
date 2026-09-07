TRELLARA_TEST_DATABASE_URL ?= postgresql://trellara:trellara@localhost:55433/trellara_target
CORRECTNESS_REPORT ?= docs/correctness-report.html
CORRECTNESS_SITE_DIR ?= target/correctness-site
SOURCE_REVISION ?= local
SOURCE_REPOSITORY ?= local
WORKFLOW_RUN_URL ?= local
QUICKSTART_CONFIG ?= examples/retail-fleet/local.yml
QUICKSTART_PROOF_OUTPUT ?= target/quickstart-proof
QUICKSTART_LOCAL_CONFIG ?= trellara.yml
QUICKSTART_EVIDENCE_OUTPUT ?= target/trellara-quickstart-evidence

.PHONY: fmt fmt-check clippy test validate-example quickstart-local quickstart-check quickstart-proof-check correctness-report correctness-site verify-correctness-report verify-correctness-site ci integration-test dev-up dev-down dev-logs

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

clippy:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

validate-example:
	cargo run -p trellara-cli -- validate --config examples/retail-fleet/strict.yml

quickstart-local:
	cargo run -p trellara-cli -- dev up
	cargo run -p trellara-cli -- init --source-database-url postgresql://trellara:trellara@localhost:55432/trellara_source --target-database-url postgresql://trellara:trellara@localhost:55433/trellara_target --source-id local-source --dataset-id retail-sales --publication trellara_retail --slot trellara_retail_slot --table public.sales --table public.sale_items --table public.payments --output "$(QUICKSTART_LOCAL_CONFIG)" --evaluate --force
	cargo run -p trellara-cli -- check --config "$(QUICKSTART_LOCAL_CONFIG)" --format text
	cargo run -p trellara-cli -- preflight --config "$(QUICKSTART_LOCAL_CONFIG)"
	cargo run -p trellara-cli -- run --local --verify --format text --config "$(QUICKSTART_LOCAL_CONFIG)" --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100
	cargo run -p trellara-cli -- status --config "$(QUICKSTART_LOCAL_CONFIG)" --view report --format text
	cargo run -p trellara-cli -- pilot-package --config "$(QUICKSTART_LOCAL_CONFIG)" --output "$(QUICKSTART_EVIDENCE_OUTPUT)"
	@printf '%s\n' "evidence bundle: $(QUICKSTART_EVIDENCE_OUTPUT)"
	@printf '%s\n' "recovery path: cargo run -p trellara-cli -- status --config \"$(QUICKSTART_LOCAL_CONFIG)\" --view diagnostics --format text"

quickstart-check:
	cargo run -p trellara-cli -- quickstart --config "$(QUICKSTART_CONFIG)" --check

quickstart-proof-check:
	@set -e; \
	mkdir -p "$(QUICKSTART_PROOF_OUTPUT)"; \
	cargo run -p trellara-cli -- mvp-check --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	cargo run -p trellara-cli -- quickstart --config "$(QUICKSTART_CONFIG)" --check --format text > "$(QUICKSTART_PROOF_OUTPUT)/quickstart-readiness.txt"; \
	cargo run -p trellara-cli -- quickstart --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	cargo run -p trellara-cli -- pilot-guide --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	cargo run -p trellara-cli -- pilot-scorecard --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/pilot-scorecard.txt"; \
	cargo run -p trellara-cli -- pilot-evidence --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/executive-evidence.md"; \
	cargo run -p trellara-cli -- evaluate --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/enterprise-evaluation.txt"; \
	cargo run -p trellara-cli -- schema ddl-plan --config "$(QUICKSTART_CONFIG)" --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe > "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	cargo run -p trellara-cli -- schema ddl-apply-plan --config "$(QUICKSTART_CONFIG)" --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe > "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	cargo run -p trellara-cli -- schema ddl-plan --config examples/retail-fleet/partitioned.yml --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe > "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	cargo run -p trellara-cli -- schema ddl-plan --config examples/retail-fleet/partitioned.yml --change change_partition_key:public.sales.region_id --apply-mode auto-safe > "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	cargo run -p trellara-cli -- consistency --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	cargo run -p trellara-cli -- performance --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	cargo run -p trellara-cli -- identity-audit --config "$(QUICKSTART_CONFIG)" --format text > "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	cargo run -p trellara-cli -- semantics --config "$(QUICKSTART_CONFIG)" > "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
		cargo run -p trellara-cli -- lake ddl --config "$(QUICKSTART_CONFIG)" > "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
		cargo run -p trellara-cli -- lake epoch --config "$(QUICKSTART_CONFIG)" > "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
		cargo run -p trellara-cli -- lake fanin verify --config "$(QUICKSTART_CONFIG)" --stream-epoch "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json" --lake-epoch "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json" --accept-complete-with-gaps > "$(QUICKSTART_PROOF_OUTPUT)/lake-verify.json"; \
		cargo run -p trellara-cli -- lake spark-template current-state --config "$(QUICKSTART_CONFIG)" --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps > "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		cargo run -p trellara-cli -- lake spark-template scd2 --config "$(QUICKSTART_CONFIG)" --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps > "$(QUICKSTART_PROOF_OUTPUT)/spark-scd2.sql"; \
		cargo run -p trellara-cli -- lake spark-template maintenance --config "$(QUICKSTART_CONFIG)" --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps > "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		cargo run -p trellara-cli -- lake spark-template dashboard --config "$(QUICKSTART_CONFIG)" --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps > "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
	cargo run -p trellara-cli -- fleet init --source-database-url postgresql://trellara:trellara@localhost:55432/trellara_source --target-database-url postgresql://trellara:trellara@localhost:55433/trellara_target --source-id local-source --database-id retail --dataset retail-east --dataset retail-west --table public.sales --table public.sale_items --table public.payments --output-dir "$(QUICKSTART_PROOF_OUTPUT)/fleet-init" --force > "$(QUICKSTART_PROOF_OUTPUT)/fleet-init.json"; \
	cargo run -p trellara-cli -- fleet report --config "$(QUICKSTART_CONFIG)" --config examples/retail-fleet/partitioned.yml --format text > "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	cargo run -p trellara-cli -- fleet identity-audit --config "$(QUICKSTART_CONFIG)" --config examples/retail-fleet/partitioned.yml --format text > "$(QUICKSTART_PROOF_OUTPUT)/fleet-identity-audit.txt"; \
	cargo run -p trellara-cli -- fleet evidence-plan --config "$(QUICKSTART_CONFIG)" --config examples/retail-fleet/partitioned.yml --format text > "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	cargo run -p trellara-cli -- fleet scorecard --config "$(QUICKSTART_CONFIG)" --config examples/retail-fleet/partitioned.yml --format text > "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	cargo run -p trellara-cli -- fleet control-plane --config "$(QUICKSTART_CONFIG)" --config examples/retail-fleet/partitioned.yml > "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
	cargo run -p trellara-cli -- pilot-package --config "$(QUICKSTART_CONFIG)" --output "$(QUICKSTART_PROOF_OUTPUT)/package" > "$(QUICKSTART_PROOF_OUTPUT)/pilot-package.json"; \
	cargo run -p trellara-cli -- pilot-package --config examples/retail-fleet/partitioned.yml --output "$(QUICKSTART_PROOF_OUTPUT)/package-partitioned" > "$(QUICKSTART_PROOF_OUTPUT)/pilot-package-partitioned.json"; \
	cp "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-envelope-plan.json"; \
	cp "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json" "$(QUICKSTART_PROOF_OUTPUT)/ddl-barrier-status.json"; \
	cp "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	cargo run -p trellara-cli -- evidence-registry --package "$(QUICKSTART_PROOF_OUTPUT)/package" --format text > "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "Trellara MVP readiness" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "criteria: 14/14 passed" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "no_broker_verified_flow_under_10_minutes" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "large_transactions_bounded" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "partitioned_scale_mode_proven" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "7/7 partitioned scale proof scenarios covered" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "source_failover_readiness_proven" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "3/3 source failover proof scenarios covered" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "schema_change_recovery_scripted" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "3/3 schema-change proof scenarios covered" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "ddl_propagation_contract_packaged" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "target/lake/Spark sink ACKs" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "release blockers" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "protocol_property_tests_cover_invariants" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "modular envelope/idempotency/LSN/strict chunk/partitioned/missing/duplicate proptests current=true" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "design_partner_can_evaluate_without_kafka" "$(QUICKSTART_PROOF_OUTPUT)/mvp-readiness.txt"; \
	grep -q "trellara check --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-readiness.txt"; \
	grep -q "trellara pilot-package --config $(QUICKSTART_CONFIG) --output target/trellara-quickstart-evidence" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-readiness.txt"; \
	grep -q "recovery: trellara status --config $(QUICKSTART_CONFIG) --view diagnostics --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-readiness.txt"; \
	grep -q "trellara dev up" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara init --source-database-url" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara check --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara preflight --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara run --local --verify --format text --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara status --config $(QUICKSTART_CONFIG) --view report --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara pilot-package --config $(QUICKSTART_CONFIG) --output target/trellara-quickstart-evidence" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "operator_reference:" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara evaluate --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara contract-test --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara pilot-scorecard --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "trellara pilot-evidence --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/quickstart-plan.txt"; \
	grep -q "\"strict_language\"" "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
	grep -q "\"partitioned_language\"" "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
	grep -q "\"mode\": \"exact_transaction\"" "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
	grep -q "\"mode\": \"partition_local\"" "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
	grep -q "atomic visibility for cross-partition transactions" "$(QUICKSTART_PROOF_OUTPUT)/consumer-semantics.json"; \
	grep -q "\"table_count\": 9" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "\"checkpoint_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "retail_sales__public__sales__raw_cdc" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "_trellara_epochs" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "_trellara_epoch_sources" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "_trellara_epoch_partitions" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "_trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "retail_sales__spark__derived__current_state" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "retail_sales__spark__derived__scd2_history" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "__trellara_commit_lsn" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "__trellara_epoch_id" "$(QUICKSTART_PROOF_OUTPUT)/lake-ddl.json"; \
	grep -q "\"contract\": \"fleet_fanin_append_only_raw_cdc_with_epoch_completeness\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"state\": \"complete_with_gaps\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"customer_decision\": \"requires_explicit_gap_acceptance_before_spark_consumption\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"missing_source_count\": 3" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"straggler_policy\": \"publish_with_gaps\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"straggler_policy_decision\":" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "accept_complete_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"manifest_digest\":" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"watermark_rollup\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"global_low_watermark_lsn\": \"0/16B6C60\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"source_watermarks\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"table_rollups\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"relation\": \"public.sales\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"quarantine_entries\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"gap_reason\": \"required source missing from published gap epoch\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
	grep -q "\"verification_status\": \"match\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-epoch.json"; \
		grep -q "\"status\": \"match\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-verify.json"; \
		grep -q "\"spark_consumption_allowed\": true" "$(QUICKSTART_PROOF_OUTPUT)/lake-verify.json"; \
		grep -q "\"mismatch_count\": 0" "$(QUICKSTART_PROOF_OUTPUT)/lake-verify.json"; \
		grep -q "\"contract\": \"fleet_fanin_append_only_raw_cdc_with_epoch_completeness\"" "$(QUICKSTART_PROOF_OUTPUT)/lake-verify.json"; \
		grep -q "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__current" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "retail_sales__public__sales__raw_cdc" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_epochs" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "e.state = 'complete_with_gaps'" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "e.policy = 'publish_with_gaps'" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "AND true = true" "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/spark-current-state.sql"; \
		grep -q "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__scd2" "$(QUICKSTART_PROOF_OUTPUT)/spark-scd2.sql"; \
		grep -q "WHEN NOT MATCHED THEN INSERT" "$(QUICKSTART_PROOF_OUTPUT)/spark-scd2.sql"; \
		grep -q "__trellara_valid_from" "$(QUICKSTART_PROOF_OUTPUT)/spark-scd2.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/spark-scd2.sql"; \
		grep -q "CALL spark_catalog.system.rewrite_data_files" "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		grep -q "CALL spark_catalog.system.expire_snapshots" "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/spark-maintenance.sql"; \
		grep -q "epoch_release_gate" "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_epoch_sources" "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
		grep -q "safe_to_publish" "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/spark-completeness-dashboard.sql"; \
	grep -q "large_transaction_evidence:" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "trellara status --config $(QUICKSTART_CONFIG) --view report --format text" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "trellara status --config $(QUICKSTART_CONFIG) --view diagnostics --format text" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "trellara run --local --verify --format text --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "capture_spill_boundary:" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "pilot can run without adopting Kafka" "$(QUICKSTART_PROOF_OUTPUT)/pilot-guide.txt"; \
	grep -q "verdict: ready_for_live_pilot" "$(QUICKSTART_PROOF_OUTPUT)/pilot-scorecard.txt"; \
	grep -q "bounded_memory_capture" "$(QUICKSTART_PROOF_OUTPUT)/pilot-scorecard.txt"; \
	grep -q "Trellara Executive Evidence Brief" "$(QUICKSTART_PROOF_OUTPUT)/executive-evidence.md"; \
	grep -q "transaction-boundary proof" "$(QUICKSTART_PROOF_OUTPUT)/executive-evidence.md"; \
	grep -q "Trellara enterprise evaluation" "$(QUICKSTART_PROOF_OUTPUT)/enterprise-evaluation.txt"; \
	grep -q "recommended_mode: strict_chunked_transaction_order" "$(QUICKSTART_PROOF_OUTPUT)/enterprise-evaluation.txt"; \
	grep -q "manifest boundary_mode" "$(QUICKSTART_PROOF_OUTPUT)/enterprise-evaluation.txt"; \
	grep -q "\"verdict\": \"ready_to_apply\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"decision\": \"auto_apply\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"transaction_boundary_rule\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "schema barrier" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"propagation\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"dml_after_barrier_held\": true" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"release_gates\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"release_blockers\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"blockers\": \\[\\]" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"requires_global_partition_pause\": false" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"kind\": \"target_postgres\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"kind\": \"raw_cdc_lake\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"kind\": \"spark_derived_view\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"required_ack\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"ack_evidence\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"release_dml\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"rejection_code\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-barrier-status.json"; \
	grep -q "\"release_blocker_codes\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-barrier-status.json"; \
	grep -q "\"release_decision\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"cdc_transaction_boundary\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"ddl_dml_replay_proof\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"target_ack_lsn\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"dml_commit_lsn\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"blocker_codes\"" "$(QUICKSTART_PROOF_OUTPUT)/ddl-release-proof.json"; \
	grep -q "\"target_postgres_sql\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "ALTER TABLE" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "ADD COLUMN" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "discount_code" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier record --config" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier ack --config" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--plan-sha256 <target-ddl-plan-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--statement-sha256 <target-ddl-statement-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--epoch-id <lake-epoch-id>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--metadata-table <raw-cdc-epoch-metadata-table>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--partition-metadata-table <raw-cdc-epoch-partitions-table>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--manifest-digest <lake-epoch-manifest-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "trellara lake spark-template current-state --config" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "trellara lake spark-template scd2 --config" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--format json" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--template-digest <spark-template-sha256-hex-from-template_sha256>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--accepted-by <reviewer-or-automation>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q -- "--view-count <derived-view-count>" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier status --config" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-plan.json"; \
	grep -q "\"executable\": true" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	grep -q "\"statement_count\": 1" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	grep -F -q "ALTER TABLE" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	grep -q "\"release_gate_code\": \"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	grep -q "post_ddl_dml_release" "$(QUICKSTART_PROOF_OUTPUT)/schema-ddl-apply-plan.json"; \
	grep -q "\"verdict\": \"ready_to_apply\"" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "\"dml_after_barrier_held\": true" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "\"requires_global_partition_pause\": true" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "\"kind\": \"partition_visibility\"" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "\"verdict\": \"blocked\"" "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	grep -q "\"blockers\"" "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	grep -q "\"kind\": \"change_partition_key\"" "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	grep -q "\"release_impact\"" "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	grep -q "partition-key changes alter partitioned scale mode ordering" "$(QUICKSTART_PROOF_OUTPUT)/blocked-schema-ddl-plan.json"; \
	grep -q "\"partition_visibility_watermark\"" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "partitioned: all partition lanes must acknowledge the schema barrier before global visibility" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "partition-watermarks output showing every partition at or beyond barrier_lsn" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier ack --config" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q -- "--plan-sha256 <target-ddl-plan-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q -- "--statement-sha256 <target-ddl-statement-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q -- "--expected-partition-count <partition-count>" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q -- "--partition-durable-lsn <partition-id>=<durable-lsn>" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -F -q -- "--partition-applied-lsn <partition-id>=<applied-lsn>" "$(QUICKSTART_PROOF_OUTPUT)/partitioned-schema-ddl-plan.json"; \
	grep -q "Trellara consistency contract" "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	grep -q "source_ack_contract:" "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	grep -q "snapshot_handoff_contract:" "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	grep -q "target_checkpoint_contract:" "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	grep -q "strict_chunk_manifest" "$(QUICKSTART_PROOF_OUTPUT)/consistency-contract.txt"; \
	grep -q "Trellara performance envelope" "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	grep -q "quickstart_target: 8 minute estimate within 10 minute budget" "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	grep -q "stream_spill_threshold_changes: 1024" "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	grep -q "strict chunking limits relay memory" "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	grep -q "local fsync durability" "$(QUICKSTART_PROOF_OUTPUT)/performance-envelope.txt"; \
	grep -q "Trellara identity audit" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "ordinary_pk_tables_do_not_require_full: true" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "REPLICA IDENTITY FULL is not required" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "absent non-key columns are treated as unchanged" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "update_omits_absent_non_key_columns_for_unchanged_toast" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "update_omits_explicit_unchanged_toast_marker" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "pgoutput_decoder_rejects_omitted_unchanged_key_column" "$(QUICKSTART_PROOF_OUTPUT)/identity-audit.txt"; \
	grep -q "\"flow_count\": 2" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init.json"; \
	grep -q "retail-east.yml" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init.json"; \
	grep -q "retail-west.yml" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init.json"; \
	grep -q "trellara fleet report --config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init.json"; \
	grep -q "Trellara Fleet Init" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/README.md"; \
	grep -q "trellara fleet report" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/README.md"; \
	grep -q "retail-east.yml" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/fleet-manifest.json"; \
	grep -q "mode: strict_transaction_order" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/configs/retail-east.yml"; \
	grep -q "stream_spill_threshold_changes" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/configs/retail-east.yml"; \
	grep -q "durability: fsync" "$(QUICKSTART_PROOF_OUTPUT)/fleet-init/configs/retail-east.yml"; \
	grep -q "Trellara fleet report" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "flows: 2" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "streams: local=1 kafka=1" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "partitioned=1" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_fanin: verdict=publishable_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_fanin: status=publishable_with_gaps mode=partitioned_scale_epoch_fanin" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "global low watermark" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "publish_with_gaps requires explicit missing-source acceptance" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "convergence_gates:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "target_convergence" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "local_stream_durability" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "partition_watermarks" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "recovery_drills:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_offline_source_gap_acceptance" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_late_source_recompute" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_conflicting_duplicate_quarantine" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "lake_missing_manifest_chunk_replay" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "target_quarantine_replay" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "stream locate-local" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "checksum_reseed" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "schema_handoff_refresh" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "partition-watermarks" "$(QUICKSTART_PROOF_OUTPUT)/fleet-report.txt"; \
	grep -q "Trellara fleet identity audit" "$(QUICKSTART_PROOF_OUTPUT)/fleet-identity-audit.txt"; \
	grep -q "verdict: blocked_by_identity_collision" "$(QUICKSTART_PROOF_OUTPUT)/fleet-identity-audit.txt"; \
	grep -q "duplicate_groups:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-identity-audit.txt"; \
	grep -q "assign unique source.id and dataset.id values" "$(QUICKSTART_PROOF_OUTPUT)/fleet-identity-audit.txt"; \
	grep -q "Trellara fleet evidence plan" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "verdict: ready_to_collect_live_evidence" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "needs_live_evidence" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "required_live_artifacts:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "trellara check --config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "transaction_boundary" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "trellara partition-watermarks --config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "trellara partition-rebalance-plan --config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-evidence-plan.txt"; \
	grep -q "Trellara fleet scorecard" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "verdict: review_required_before_fleet_pilot" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "flow_status: 1 ready, 1 review_required, 0 blocked" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "acceptance_gates:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "convergence_gates: 12 total, 0 blocked_by_config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "recovery_drills: 16" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "analytical_fanin: verdict=publishable_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "analytical_fanin: status=publishable_with_gaps mode=partitioned_scale_epoch_fanin" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "review_sequence:" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "trellara fleet report --config" "$(QUICKSTART_PROOF_OUTPUT)/fleet-scorecard.txt"; \
	grep -q "\"verdict\": \"blocked_by_identity_collision\"" "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
	grep -q "\"code\": \"read_only_fleet_topology\"" "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
	grep -q "\"code\": \"evidence_package_registry\"" "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
	grep -q "\"code\": \"deployment_orchestration\"" "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
	grep -q "\"status\": \"defer\"" "$(QUICKSTART_PROOF_OUTPUT)/fleet-control-plane.json"; \
		grep -q "\"artifact_count\": 51" "$(QUICKSTART_PROOF_OUTPUT)/pilot-package.json"; \
	grep -q "Pgoutput Implementation Design" "docs/DESIGN.md"; \
	grep -q "protocol_version: 2" "docs/DESIGN.md"; \
	grep -q "source checkpoint advances only after" "docs/DESIGN.md"; \
	grep -q "Stream Commit" "docs/DESIGN.md"; \
	grep -q "REPLICA IDENTITY DEFAULT" "docs/DESIGN.md"; \
	grep -q "relation fingerprint changes during a protocol v2 streamed transaction" "docs/DESIGN.md"; \
	grep -q "Initial Snapshot State Machine" "docs/DESIGN.md"; \
	grep -q "stream_handoff_ready" "docs/DESIGN.md"; \
	grep -q "failed_recoverable" "docs/DESIGN.md"; \
	grep -q "idempotent by run and relation" "docs/DESIGN.md"; \
	grep -q "crash after handoff event before relay starts" "docs/DESIGN.md"; \
	grep -q "Embedded Transport Design" "docs/DESIGN.md"; \
	grep -q "TLG2" "docs/DESIGN.md"; \
	grep -q "LocalDurability::Fsync" "docs/DESIGN.md"; \
	grep -q "missing indexes are rebuilt" "docs/DESIGN.md"; \
	grep -q "source acknowledgement after local durability" "docs/DESIGN.md"; \
	grep -q "Trellara evidence registry" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "verified: true" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "verified_artifacts: 50" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "review_surfaces:" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "16 required, 0 missing_required" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "correctness-report.html" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "source_safety" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "identity_audit" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "schema_ddl_plan" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "schema_ddl_apply_plan" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "ddl_release_proof" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "schema_ddl_envelope_plan" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
		grep -q "ddl_barrier_status" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "fleet_evidence_plan" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "live_evidence_template" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "control_plane_pull" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "template_sha256 for current-state DDL ACK evidence" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "template_sha256 for SCD2 DDL ACK evidence" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "template_sha256 for completeness dashboard DDL ACK evidence" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "package_manifest_sha256" "$(QUICKSTART_PROOF_OUTPUT)/evidence-registry.txt"; \
	grep -q "large_transaction_evidence:" "$(QUICKSTART_PROOF_OUTPUT)/package/pilot-guide.txt"; \
	grep -q "bounded_memory_capture" "$(QUICKSTART_PROOF_OUTPUT)/package/pilot-scorecard.txt"; \
	grep -q "Trellara Executive Evidence Brief" "$(QUICKSTART_PROOF_OUTPUT)/package/executive-evidence.md"; \
	grep -q "Trellara enterprise evaluation" "$(QUICKSTART_PROOF_OUTPUT)/package/enterprise-evaluation.txt"; \
	grep -q "recommended_mode:" "$(QUICKSTART_PROOF_OUTPUT)/package/enterprise-evaluation.txt"; \
	grep -q "\"verdict\": \"ready_to_apply\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"decision\": \"auto_apply\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "schema barrier" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"propagation\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"dml_after_barrier_held\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"release_gates\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"release_blockers\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"kind\": \"raw_cdc_lake\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"kind\": \"spark_derived_view\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"ack_evidence\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"target_postgres_sql\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "ALTER TABLE" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "ADD COLUMN" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "discount_code" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier record --config" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier ack --config" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--plan-sha256 <target-ddl-plan-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--statement-sha256 <target-ddl-statement-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--epoch-id <lake-epoch-id>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--metadata-table <raw-cdc-epoch-metadata-table>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--partition-metadata-table <raw-cdc-epoch-partitions-table>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--manifest-digest <lake-epoch-manifest-sha256>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "trellara lake spark-template current-state --config" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "trellara lake spark-template scd2 --config" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--format json" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--template-digest <spark-template-sha256-hex-from-template_sha256>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--accepted-by <reviewer-or-automation>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q -- "--view-count <derived-view-count>" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -F -q "trellara schema ddl-barrier status --config" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-plan.json"; \
	grep -q "\"executable\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-apply-plan.json"; \
	grep -q "\"statement_count\": 1" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-apply-plan.json"; \
	grep -F -q "ALTER TABLE" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-apply-plan.json"; \
	grep -q "\"release_gate_code\": \"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-apply-plan.json"; \
	grep -q "post_ddl_dml_release" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-apply-plan.json"; \
	grep -q "\"ddl_event_count\": 1" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"operation\": \"add_column\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"target_auto_apply\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"executable\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"required_sinks\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"target_sql\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"propagation_boundary\": \"source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"propagation_decisions\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"propagation_policy_sha256\"" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "post_ddl_dml_release" "$(QUICKSTART_PROOF_OUTPUT)/package/schema-ddl-envelope-plan.json"; \
	grep -q "\"release_dml\": false" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"release_blockers\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"release_blocker_codes\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"rejection_code\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"release_gates\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"propagation_boundary\": \"source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"propagation_decisions\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"propagation_policy_sha256\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "pending required sink acknowledgements" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"raw_cdc_lake\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"spark_derived_views\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-barrier-status.json"; \
	grep -q "\"release_gate\": \"post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"cdc_transaction_boundary\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"ack_commands\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"ack_evidence\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"release_decision\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"ddl_dml_replay_proof\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"target_ack_lsn\": \"0/16B8000\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"dml_commit_lsn\": \"0/16B8000\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"target_transaction_boundary\": \"single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"blocker_codes\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"propagation_boundary\": \"source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"propagation_decisions\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"propagation_policy_sha256\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"release_dml\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"pending_sink_count\": 0" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"raw_cdc_lake\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"spark_derived_views\"" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "plan_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "statement_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "partition_metadata_table=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "manifest_digest=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "template_digest=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "accepted_by=" "$(QUICKSTART_PROOF_OUTPUT)/package/ddl-release-proof.json"; \
	grep -q "\"runtime_movement_allowed\": false" "$(QUICKSTART_PROOF_OUTPUT)/package-partitioned/partition-rebalance-plan.json"; \
	grep -q "\"visibility_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package-partitioned/partition-rebalance-plan.json"; \
	grep -q "\"skew_ratio_basis_points\"" "$(QUICKSTART_PROOF_OUTPUT)/package-partitioned/partition-rebalance-plan.json"; \
	grep -q "\"recommended_moves\"" "$(QUICKSTART_PROOF_OUTPUT)/package-partitioned/partition-rebalance-plan.json"; \
	grep -q "Trellara Local Run Proof" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "source_ack_after_local_durability" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "source_ack_lsn" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "every Trellara publish ack is durable" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "source_ack_durability_proof" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "selected_table_count" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "snapshot_handoff_blocker_codes" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "snapshot_handoff_recovery_actions" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "barrier_pending_blockers" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "barrier_pending_blocker_codes" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "barrier_pending_recovery_actions" "$(QUICKSTART_PROOF_OUTPUT)/package/local-run-proof.md"; \
	grep -q "Trellara Source Safety Checklist" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "trellara-check <source-database-url>" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "trellara check --config" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "trellara check --database-url <source-database-url>" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q -- "--write-init" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "wal_status" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "failover slot" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "replica identity" "$(QUICKSTART_PROOF_OUTPUT)/package/source-safety-checklist.md"; \
	grep -q "Trellara Proof Bundle" "$(QUICKSTART_PROOF_OUTPUT)/package/proof-bundle.md"; \
	grep -q "inspect-transaction --file <envelope.pb> --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/proof-bundle.md"; \
	grep -q 'manifest `boundary_mode`' "$(QUICKSTART_PROOF_OUTPUT)/package/proof-bundle.md"; \
	grep -q "Trellara Design-Partner Deployment Guide" "$(QUICKSTART_PROOF_OUTPUT)/package/deployment-guide.md"; \
	grep -q "Go/No-Go Rule" "$(QUICKSTART_PROOF_OUTPUT)/package/deployment-guide.md"; \
	grep -q "Operational Burden Notes" "$(QUICKSTART_PROOF_OUTPUT)/package/operational-burden-notes.md"; \
	grep -q "Before Trellara" "$(QUICKSTART_PROOF_OUTPUT)/package/operational-burden-notes.md"; \
	grep -q "Design-Partner Feature Pull List" "$(QUICKSTART_PROOF_OUTPUT)/package/feature-pull-list.md"; \
	grep -q "Validate Before Building Cloud" "$(QUICKSTART_PROOF_OUTPUT)/package/feature-pull-list.md"; \
	grep -q "Trellara fleet report" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "flows: 1" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "lake_fanin: verdict=publishable_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "lake_fanin: status=publishable_with_gaps mode=strict_chunked_epoch_fanin" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "convergence_gates:" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "target_convergence" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "local_stream_durability" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "recovery_drills:" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "lake_offline_source_gap_acceptance" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "lake_late_source_recompute" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "lake_conflicting_duplicate_quarantine" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "target_quarantine_replay" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "stream locate-local" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "checksum_reseed" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "source_wal_loss_reseed" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "schema_handoff_refresh" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-report.txt"; \
	grep -q "Trellara fleet scorecard" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "verdict: ready_for_design_partner_fleet_review" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "flow_status: 1 ready, 0 review_required, 0 blocked" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "convergence_gates: 6 total, 0 blocked_by_config" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "recovery_drills: 7" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "analytical_fanin: verdict=publishable_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "analytical_fanin: status=publishable_with_gaps mode=strict_chunked_epoch_fanin" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "review_sequence:" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-scorecard.txt"; \
	grep -q "Trellara fleet evidence plan" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-evidence-plan.txt"; \
	grep -q "verdict: ready_to_collect_live_evidence" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-evidence-plan.txt"; \
	grep -q "required_live_artifacts:" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-evidence-plan.txt"; \
	grep -q "trellara check --config" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-evidence-plan.txt"; \
	grep -q "\"source_ack_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consistency-contract.json"; \
	grep -q "\"snapshot_handoff_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consistency-contract.json"; \
	grep -q "\"target_checkpoint_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consistency-contract.json"; \
	grep -q "\"transaction_boundary_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consistency-contract.json"; \
	grep -q "strict_chunk_manifest" "$(QUICKSTART_PROOF_OUTPUT)/package/consistency-contract.json"; \
	grep -q "\"quickstart_time_budget_minutes\": 10" "$(QUICKSTART_PROOF_OUTPUT)/package/performance-envelope.json"; \
	grep -q "\"stream_spill_threshold_changes\": 1024" "$(QUICKSTART_PROOF_OUTPUT)/package/performance-envelope.json"; \
	grep -q "\"transaction_boundary_cost\"" "$(QUICKSTART_PROOF_OUTPUT)/package/performance-envelope.json"; \
	grep -q "strict chunking limits relay memory" "$(QUICKSTART_PROOF_OUTPUT)/package/performance-envelope.json"; \
	grep -q "\"measurement_note\"" "$(QUICKSTART_PROOF_OUTPUT)/package/performance-envelope.json"; \
	grep -q "\"ordinary_pk_tables_do_not_require_full\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/identity-audit.json"; \
	grep -q "\"pk_apply_ready_count\": 3" "$(QUICKSTART_PROOF_OUTPUT)/package/identity-audit.json"; \
	grep -q "REPLICA IDENTITY FULL is not required" "$(QUICKSTART_PROOF_OUTPUT)/package/identity-audit.json"; \
	grep -q "absent non-key columns are treated as unchanged" "$(QUICKSTART_PROOF_OUTPUT)/package/identity-audit.json"; \
	grep -q "plans_delete_with_key_predicate" "$(QUICKSTART_PROOF_OUTPUT)/package/identity-audit.json"; \
	grep -q "\"strict_language\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consumer-semantics.json"; \
	grep -q "\"partitioned_language\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consumer-semantics.json"; \
	grep -q "\"mode\": \"exact_transaction\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consumer-semantics.json"; \
	grep -q "\"mode\": \"partition_local\"" "$(QUICKSTART_PROOF_OUTPUT)/package/consumer-semantics.json"; \
	grep -q "atomic visibility for cross-partition transactions" "$(QUICKSTART_PROOF_OUTPUT)/package/consumer-semantics.json"; \
	grep -q "\"table_count\": 9" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "\"checkpoint_contract\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "retail_sales__public__sales__raw_cdc" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "_trellara_epochs" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "_trellara_epoch_sources" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "_trellara_epoch_partitions" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "_trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "retail_sales__spark__derived__current_state" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "retail_sales__spark__derived__scd2_history" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "__trellara_commit_lsn" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "__trellara_epoch_id" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-ddl.json"; \
	grep -q "\"state\": \"complete_with_gaps\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"customer_decision\": \"requires_explicit_gap_acceptance_before_spark_consumption\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"missing_source_count\": 3" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"straggler_policy\": \"publish_with_gaps\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"straggler_policy_decision\":" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "accept_complete_with_gaps" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"manifest_digest\":" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"watermark_rollup\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"global_low_watermark_lsn\": \"0/16B6C60\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"source_watermarks\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"table_rollups\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"relation\": \"public.sales\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"quarantine_entries\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
	grep -q "\"gap_reason\": \"required source missing from published gap epoch\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-epoch.json"; \
		grep -q "\"status\": \"match\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-verify.json"; \
		grep -q "\"spark_consumption_allowed\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-verify.json"; \
		grep -q "\"mismatch_count\": 0" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-verify.json"; \
		grep -q "\"contract\": \"fleet_fanin_append_only_raw_cdc_with_epoch_completeness\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-verify.json"; \
		grep -q "\"contract\": \"fleet_fanin_append_only_raw_cdc_with_epoch_completeness\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"completeness_state\": \"complete_with_gaps\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"verification_status\": \"match\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"spark_consumption_allowed\": true" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"spark_consumption_gate\": \"released: stream and lake proofs match, complete_with_gaps was explicitly accepted, and verification_status=match\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"committer_strategy\": \"single_table_committer_epoch_batched_append_only_raw_cdc\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "lake-fanin-run.json" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "source acknowledgement advances after durable Trellara stream publish" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"writer_crash_after_epoch_metadata\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"recovery_guidance\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"code\": \"explicit_gap_acceptance_required\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"operator_action\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-completeness.json"; \
		grep -q "\"committer_topology\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"strategy\": \"single_table_committer_epoch_batched_append_only_raw_cdc\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "source acknowledgement advances after durable Trellara stream publish" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "source WAL remains protected" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"commit_steps\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"phase\": \"raw_cdc_data\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"phase\": \"epoch_row_metadata\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"phase\": \"verification_metadata\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"recovery_scenarios\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"code\": \"writer_crash_after_epoch_metadata\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"replay_policy\": \"discover_existing_epoch_metadata_before_publish\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"code\": \"verification_mismatch_after_commit\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "\"replay_policy\": \"hold_spark_consumption_until_fanin_verify_matches\"" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		grep -q "before source acknowledgement" "$(QUICKSTART_PROOF_OUTPUT)/package/lake-writer-plan.json"; \
		test -s "$(QUICKSTART_PROOF_OUTPUT)/package/sample-envelope.pb"; \
		grep -q "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__current" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "retail_sales__public__sales__raw_cdc" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "e.state = 'complete_with_gaps'" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "e.policy = 'publish_with_gaps'" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "AND true = true" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.sql"; \
		grep -q "SparkSession.builder" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		grep -q "spark-current-state.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		grep -q "refusing unresolved Trellara Spark template placeholders" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		grep -q "template_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		grep -q "accept_complete_with_gaps=true" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-current-state.py"; \
		grep -q "MERGE INTO spark_catalog.retail_sales.retail_sales__public__sales__scd2" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.sql"; \
	grep -q "WHEN NOT MATCHED THEN INSERT" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.sql"; \
		grep -q "__trellara_valid_from" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.sql"; \
		grep -q "SparkSession.builder" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		grep -q "spark-scd2.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		grep -q "template=scd2" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		grep -q "template_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		grep -q "accept_complete_with_gaps=true" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-scd2.py"; \
		grep -q "CALL spark_catalog.system.rewrite_data_files" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.sql"; \
		grep -q "CALL spark_catalog.system.expire_snapshots" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.sql"; \
		grep -q "SparkSession.builder" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		grep -q "spark-maintenance.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		grep -q "template=maintenance" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		grep -q "template_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		grep -q "accept_complete_with_gaps=true" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-maintenance.py"; \
		grep -q "epoch_release_gate" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		grep -q "safe_to_publish" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_epoch_sources" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		grep -q "retail_sales__trellara__fanin___trellara_verification" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		grep -q "v.checksum_status = 'match'" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.sql"; \
		grep -q "SparkSession.builder" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
		grep -q "spark-completeness-dashboard.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
		grep -q "template=dashboard" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
		grep -q "template_sha256=" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
		grep -q "accept_complete_with_gaps=true" "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
		! grep -q '$${' "$(QUICKSTART_PROOF_OUTPUT)/package/spark-completeness-dashboard.py"; \
	grep -q "\"verdict\": \"validate_with_design_partners\"" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-control-plane.json"; \
	grep -q "\"code\": \"read_only_fleet_topology\"" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-control-plane.json"; \
	grep -q "\"code\": \"evidence_package_registry\"" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-control-plane.json"; \
	grep -q "\"code\": \"deployment_orchestration\"" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-control-plane.json"; \
	grep -q "\"status\": \"defer\"" "$(QUICKSTART_PROOF_OUTPUT)/package/fleet-control-plane.json"; \
	grep -q "\"sha256\"" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "pilot-scorecard.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "executive-evidence.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "enterprise-evaluation.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "enterprise-evaluation.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "schema-ddl-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "schema-ddl-apply-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "schema-ddl-envelope-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "ddl-barrier-status.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "ddl-release-proof.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "local-run-proof.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "source-safety-checklist.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "proof-bundle.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "deployment-guide.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "operational-burden-notes.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "feature-pull-list.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "fleet-report.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "fleet-scorecard.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "fleet-evidence-plan.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "live-evidence/README.md" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "live-evidence/collect.sh" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "consistency-contract.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "performance-envelope.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "identity-audit.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "consumer-semantics.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-ddl.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-epoch.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-verify.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-completeness.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-writer-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "sample-envelope.pb" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "lake-fanin-run.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-current-state.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-current-state.py" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-scd2.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-scd2.py" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-maintenance.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-maintenance.py" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-completeness-dashboard.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-completeness-dashboard.py" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
		grep -q "spark-golden-fixture.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "fleet-control-plane.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "diagnostics.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "diagnostics.json" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "correctness-report.html" "$(QUICKSTART_PROOF_OUTPUT)/package/manifest.json"; \
	grep -q "Trellara correctness report" "$(QUICKSTART_PROOF_OUTPUT)/package/correctness-report.html"; \
	grep -q "Trellara diagnostics" "$(QUICKSTART_PROOF_OUTPUT)/package/diagnostics.txt"; \
	grep -q "\"repair_plan\"" "$(QUICKSTART_PROOF_OUTPUT)/package/diagnostics.json"; \
	grep -q "trellara pilot-guide --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara pilot-evidence --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara check --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara preflight --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara run --local --verify --format text --config $(QUICKSTART_CONFIG)" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara status --config $(QUICKSTART_CONFIG) --view report --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara evaluate --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara schema ddl-plan --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara schema ddl-apply-plan --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara fleet report --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara fleet scorecard --config $(QUICKSTART_CONFIG) --format text" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "enterprise-evaluation.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "local-run-proof.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "source-safety-checklist.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "proof-bundle.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "deployment-guide.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "operational-burden-notes.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "feature-pull-list.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "fleet-report.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "fleet-scorecard.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "fleet-evidence-plan.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "schema-ddl-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "schema-ddl-apply-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "schema-ddl-envelope-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "ddl-barrier-status.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "ddl-release-proof.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "live-evidence/README.md" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "live-evidence/collect.sh" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara pilot evidence-template --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara pilot evidence-check --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "consistency-contract.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "performance-envelope.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "identity-audit.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "consumer-semantics.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "lake-ddl.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "lake-epoch.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara lake epoch --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "lake-verify.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake fanin verify --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "lake-completeness.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "lake-writer-plan.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "sample-envelope.pb" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "lake-fanin-run.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake writer-plan --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake writer-plan --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-current-state.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-current-state.py" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "template_sha256.*Spark-derived-view DDL ACK evidence" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-scd2.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-scd2.py" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-maintenance.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-maintenance.py" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "template_sha256.*Spark maintenance ACK review" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-completeness-dashboard.sql" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-completeness-dashboard.py" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "template_sha256.*completeness-dashboard ACK review" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "spark-golden-fixture.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake spark-template current-state --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake spark-template scd2 --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
		grep -q "trellara lake spark-template maintenance --config" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "fleet-control-plane.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "diagnostics.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "diagnostics.json" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "correctness-report.html" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "trellara evidence-registry --package" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"; \
	grep -q "Trellara Live Evidence Collection" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "source-safety.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "transaction-boundary.txt" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "ddl-release-proof.json" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "release_dml true" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "ack_commands" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "ack_evidence" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/README.md"; \
	grep -q "trellara check --config" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/collect.sh"; \
	grep -q "trellara status --config" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/collect.sh"; \
	grep -q "trellara pilot-package --config" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/collect.sh"; \
	grep -q "ddl-release-proof-package/ddl-release-proof.json" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/collect.sh"; \
	grep -q "trellara pilot evidence-check --config" "$(QUICKSTART_PROOF_OUTPUT)/package/live-evidence/collect.sh"; \
	grep -q "single-review proof chain" "$(QUICKSTART_PROOF_OUTPUT)/package/README.md"

correctness-report:
	cargo run -p trellara-cli -- chaos report --output "$(CORRECTNESS_REPORT)" --source-revision "$(SOURCE_REVISION)" --source-repository "$(SOURCE_REPOSITORY)" --workflow-run-url "$(WORKFLOW_RUN_URL)"

correctness-site:
	mkdir -p "$(CORRECTNESS_SITE_DIR)"
	cargo run -p trellara-cli -- chaos report --output "$(CORRECTNESS_REPORT)" --source-revision "$(SOURCE_REVISION)" --source-repository "$(SOURCE_REPOSITORY)" --workflow-run-url "$(WORKFLOW_RUN_URL)" > "$(CORRECTNESS_SITE_DIR)/summary.json"
	cp "$(CORRECTNESS_REPORT)" "$(CORRECTNESS_SITE_DIR)/index.html"
	cp "$(CORRECTNESS_REPORT)" "$(CORRECTNESS_SITE_DIR)/correctness-report.html"
	@bytes=$$(wc -c < "$(CORRECTNESS_REPORT)" | tr -d ' '); \
	sha256=$$(shasum -a 256 "$(CORRECTNESS_REPORT)" | awk '{print $$1}'); \
	seed=$$(sed -n 's/.*"deterministic_seed": \([0-9][0-9]*\).*/\1/p' "$(CORRECTNESS_SITE_DIR)/summary.json"); \
	simulations=$$(sed -n 's/.*"simulation_count": \([0-9][0-9]*\).*/\1/p' "$(CORRECTNESS_SITE_DIR)/summary.json"); \
	scenarios=$$(sed -n 's/.*"scenario_count": \([0-9][0-9]*\).*/\1/p' "$(CORRECTNESS_SITE_DIR)/summary.json"); \
	printf '%s\n' "{\"artifact\":\"correctness-report.html\",\"index\":\"index.html\",\"summary\":\"summary.json\",\"status\":\"pass\",\"source_revision\":\"$(SOURCE_REVISION)\",\"source_repository\":\"$(SOURCE_REPOSITORY)\",\"workflow_run_url\":\"$(WORKFLOW_RUN_URL)\",\"deterministic_seed\":$$seed,\"simulation_count\":$$simulations,\"scenario_count\":$$scenarios,\"byte_count\":$$bytes,\"sha256\":\"$$sha256\",\"generated_by\":\"trellara chaos report\",\"freshness_check\":\"make verify-correctness-report\"}" > "$(CORRECTNESS_SITE_DIR)/manifest.json"

verify-correctness-report:
	@set -e; \
	tmp=$$(mktemp); \
	trap 'rm -f "$$tmp"' EXIT; \
	cargo run -p trellara-cli -- chaos report --output "$$tmp" >/dev/null; \
	cmp -s "$(CORRECTNESS_REPORT)" "$$tmp" || { \
		echo "correctness report is stale; run: make correctness-report"; \
		exit 1; \
	}

verify-correctness-site:
	@set -e; \
	tmpdir=$$(mktemp -d); \
	trap 'rm -rf "$$tmpdir"' EXIT; \
	$(MAKE) --no-print-directory correctness-site CORRECTNESS_REPORT="$$tmpdir/report.html" CORRECTNESS_SITE_DIR="$$tmpdir/site" >/dev/null; \
	test -s "$$tmpdir/site/index.html"; \
	test -s "$$tmpdir/site/correctness-report.html"; \
	test -s "$$tmpdir/site/summary.json"; \
	test -s "$$tmpdir/site/manifest.json"; \
	cmp -s "$$tmpdir/site/index.html" "$$tmpdir/site/correctness-report.html"; \
	grep -q '"simulation_count": 28' "$$tmpdir/site/summary.json"; \
	grep -q '"enterprise_review_gate_count": 8' "$$tmpdir/site/summary.json"; \
	grep -q '"code": "transaction_boundary"' "$$tmpdir/site/summary.json"; \
	grep -q '"artifact":"correctness-report.html"' "$$tmpdir/site/manifest.json"; \
	grep -q '"status":"pass"' "$$tmpdir/site/manifest.json"; \
	grep -q '"simulation_count":28' "$$tmpdir/site/manifest.json"; \
	grep -q '"scenario_count":63' "$$tmpdir/site/manifest.json"; \
	grep -q '"sha256":"' "$$tmpdir/site/manifest.json"

ci: fmt-check clippy test validate-example quickstart-check quickstart-proof-check verify-correctness-report verify-correctness-site
	docker compose config --quiet

integration-test:
	TRELLARA_TEST_DATABASE_URL="$(TRELLARA_TEST_DATABASE_URL)" cargo test -p trellara-apply-postgres --test postgres_integration -- --nocapture

dev-up:
	docker compose up -d

dev-down:
	docker compose down -v

dev-logs:
	docker compose logs -f

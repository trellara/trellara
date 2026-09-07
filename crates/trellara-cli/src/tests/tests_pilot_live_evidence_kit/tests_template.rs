use super::*;

#[path = "tests_template/command.rs"]
mod command;
#[path = "tests_template/partition_watermark.rs"]
mod partition_watermark;

#[test]
fn pilot_evidence_template_writes_collection_kit() {
    let root = temp_root("pilot-evidence-template");
    let output_path = root.join("live-evidence");
    fs::create_dir_all(&root).expect("create evidence template temp dir");
    let config_path = write_local_config(&root);
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary =
        write_pilot_evidence_template(&config, &config_path, &output_path).expect("template");

    assert_eq!(summary.artifact_count, 9);
    assert!(summary
        .files
        .iter()
        .any(|file| file.path.ends_with("README.md")));
    assert!(summary
        .files
        .iter()
        .any(|file| file.path.ends_with("collect.sh")));
    assert!(summary
        .artifacts
        .iter()
        .any(|artifact| artifact.gate_code == "source_safety"
            && artifact.artifact.ends_with("source-safety.txt")
            && artifact
                .success_markers
                .contains(&"source dataset identity".to_string())
            && artifact
                .success_markers
                .contains(&"no critical findings".to_string())));
    for gate_code in [
        "source_safety",
        "contract_preflight",
        "transaction_boundary",
        "snapshot_handoff",
        "verified_apply",
        "failure_drill",
        "lake_writer_plan",
        "lake_spark_consumption",
        "ddl_release_proof",
    ] {
        assert!(
            summary
                .artifacts
                .iter()
                .any(|artifact| artifact.gate_code == gate_code
                    && artifact
                        .success_markers
                        .contains(&"source dataset identity".to_string())),
            "expected {gate_code} template artifact to require source dataset identity"
        );
    }
    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "transaction_boundary"
            && artifact.artifact.ends_with("transaction-boundary.txt")
            && artifact
                .success_markers
                .contains(&"transaction_boundary verified".to_string())
            && artifact
                .success_markers
                .contains(&"source_ack_lsn evidence".to_string())
            && artifact
                .success_markers
                .contains(&"source ack publish destinations".to_string())
            && artifact
                .success_markers
                .contains(&"parallel replay contract".to_string())
            && artifact.collection_requirements.iter().any(|requirement| {
                requirement.contains("target applied LSN reaches the source durable LSN")
            })
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("source_ack_durability_proof"))
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("crash_safe_ack=true"))
            && artifact.collection_requirements.iter().any(|requirement| {
                requirement.contains("source_ack_publish_destinations_match=true")
            })
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("partition workers only replay DML-only"))
    }));
    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "lake_writer_plan"
            && artifact.artifact.ends_with("lake-writer-plan.json")
            && artifact
                .success_markers
                .contains(&"DDL boundary metadata".to_string())
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("post_ddl_dml_release"))
    }));
    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "lake_spark_consumption"
            && artifact.artifact.ends_with("lake-completeness.json")
            && artifact
                .success_markers
                .contains(&"spark_consumption_allowed true".to_string())
            && artifact
                .success_markers
                .contains(&"spark consumption contract".to_string())
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("spark_consumption_contract"))
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("released Spark consumption gate"))
    }));
    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "ddl_release_proof"
            && artifact.artifact.ends_with("ddl-release-proof.json")
            && artifact
                .success_markers
                .contains(&"release_dml true".to_string())
            && artifact
                .success_markers
                .contains(&"ack_commands".to_string())
            && artifact
                .success_markers
                .contains(&"cdc_transaction_boundary".to_string())
            && artifact
                .success_markers
                .contains(&"propagation_boundary".to_string())
            && artifact
                .success_markers
                .contains(&"propagation_decisions".to_string())
            && artifact
                .success_markers
                .contains(&"propagation_policy_sha256".to_string())
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("post-DDL DML is held"))
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("ack_evidence"))
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("propagation_policy_sha256"))
    }));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("trellara pilot evidence-check --config")));

    let readme = fs::read_to_string(output_path.join("README.md")).expect("read readme");
    let script = fs::read_to_string(output_path.join("collect.sh")).expect("read script");
    assert!(readme.contains("Trellara Live Evidence Collection"));
    assert!(readme.contains("Identity Binding"));
    assert!(readme.contains(
        "Placeholder values such as `missing`, `none`, `null`, or `unknown` are rejected."
    ));
    assert!(readme.contains("source-safety.txt"));
    assert!(readme.contains("transaction-boundary.txt"));
    assert!(readme.contains("target applied LSN reaches the source durable LSN"));
    assert!(readme.contains("parallel replay contract"));
    assert!(readme.contains("lake-writer-plan.json"));
    assert!(readme.contains("post_ddl_dml_release"));
    assert!(readme.contains("lake-completeness.json"));
    assert!(readme.contains("ddl-release-proof.json"));
    assert!(readme.contains("cdc_transaction_boundary"));
    assert!(readme.contains("post-DDL DML is held"));
    assert!(readme.contains("verified-apply.txt"));
    assert!(readme.contains("requirements:"));
    assert!(readme.contains("structured contract-test JSON artifact"));
    assert!(readme.contains("every selected table must be copy_complete"));
    assert!(readme.contains("repair-plan state plus quarantine list"));
    assert!(script.contains("set -eu"));
    assert!(script.contains("trellara check --config"));
    assert!(script.contains("trellara run --local --verify --format text --config"));
    assert!(script.contains("trellara status --config"));
    assert!(script.contains("trellara lake fanin verify --config"));
    assert!(script.contains("--accept-complete-with-gaps"));
    assert!(script.contains("trellara pilot-package --config"));
    assert!(script.contains("--output"));
    assert!(script.contains("ddl-release-proof-package"));
    assert!(script.contains("cat "));
    assert!(script.contains("ddl-release-proof-package/ddl-release-proof.json"));
    assert!(script.contains(">"));
    assert!(script.contains("trellara pilot evidence-check --config"));

    fs::remove_dir_all(root).expect("remove evidence template temp dir");
}

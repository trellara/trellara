use super::*;

#[test]
fn pilot_evidence_template_includes_partition_watermark_artifact() {
    let root = temp_root("pilot-evidence-template-partitioned");
    let config_path = root.join("partitioned.yml");
    let output_path = root.join("live-evidence");
    fs::create_dir_all(&root).expect("create evidence template temp dir");
    fs::write(&config_path, partitioned_yaml()).expect("write config");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary =
        write_pilot_evidence_template(&config, &config_path, &output_path).expect("template");

    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "partition_watermarks"
            && artifact.artifact.ends_with("partition-watermarks.json")
            && artifact
                .success_markers
                .contains(&"source dataset identity".to_string())
            && artifact
                .success_markers
                .contains(&"complete_partition_set true".to_string())
            && artifact
                .success_markers
                .contains(&"partition_scale_health ready".to_string())
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("global durable/applied LSNs"))
    }));
    assert!(summary.artifacts.iter().any(|artifact| {
        artifact.gate_code == "partition_rebalance_plan"
            && artifact.artifact.ends_with("partition-rebalance-plan.json")
            && artifact
                .success_markers
                .contains(&"runtime movement disabled".to_string())
            && artifact
                .success_markers
                .contains(&"skew metrics present".to_string())
            && artifact
                .success_markers
                .contains(&"move candidates reviewable".to_string())
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("skew_ratio_basis_points"))
            && artifact
                .collection_requirements
                .iter()
                .any(|requirement| requirement.contains("reviewed cutover evidence"))
    }));

    fs::remove_dir_all(root).expect("remove evidence template temp dir");
}

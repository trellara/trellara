use super::*;
use crate::lake_fanin_run_types::LakeFaninRunSummary;

#[test]
fn lake_fanin_run_text_lists_quarantine_evidence() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse config");
    let writer_config = trellara_lake::LakeRawCdcWriterConfig::new("retail-sales", "epoch-1", 1)
        .with_required_sources(["local-source"])
        .with_straggler_policy(trellara_lake::LakeStragglerPolicy::QuarantineOnGap);
    let plan_config =
        trellara_lake::LakePlanConfig::new(vec![trellara_lake::LakeTableConfig::new(
            "public", "sales", "id",
        )]);
    let plan = trellara_lake::plan_raw_cdc_epoch_writes(&writer_config, &plan_config, &[])
        .expect("raw CDC writer plan");
    let summary = LakeFaninRunSummary::from_writer_plan(&config, plan);

    let output = render_lake_fanin_run_summary(&summary, QuickstartOutputFormat::Text)
        .expect("fan-in run text");

    assert!(output.contains("status=blocked_no_data_files"));
    assert!(output.contains("state=quarantined verification=unknown quarantine_rows=1"));
    assert!(output.contains("quarantine_evidence:"));
    assert!(output.contains("source=local-source"));
    assert!(output.contains("commit_lsn=<none>"));
    assert!(output.contains("reason=source_gap_quarantined"));
    assert!(output.contains("required source missing under quarantine-on-gap policy"));
    assert!(output.contains("recovery_command=trellara lake fanin verify"));
}

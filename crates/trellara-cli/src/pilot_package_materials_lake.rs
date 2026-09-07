use std::path::Path;

use crate::{
    lake_fanin_run_types::LakeFaninRunSummary, pilot_package_lake_writer_plan, LakeDdlSummary,
    LakeEpochArgs, LakeEpochScenario, LakeEpochSummary, LakeFaninVerifySummary,
    LakeSparkTemplateArgs, LakeSparkTemplateKind, LakeSparkTemplateSummary, QuickstartOutputFormat,
    Result, SparkGoldenFixture, TrellaraConfig,
};

pub(crate) struct PilotPackageLakeMaterials {
    pub(crate) lake_ddl: LakeDdlSummary,
    pub(crate) lake_epoch: LakeEpochSummary,
    pub(crate) lake_verify: LakeFaninVerifySummary,
    pub(crate) spark_current_state: LakeSparkTemplateSummary,
    pub(crate) spark_scd2: LakeSparkTemplateSummary,
    pub(crate) spark_maintenance: LakeSparkTemplateSummary,
    pub(crate) spark_completeness_dashboard: LakeSparkTemplateSummary,
    pub(crate) spark_golden_fixture: SparkGoldenFixture,
    pub(crate) lake_writer_plan: trellara_lake::LakeRawCdcEpochWritePlan,
    pub(crate) lake_fanin_run: LakeFaninRunSummary,
}

impl PilotPackageLakeMaterials {
    pub(crate) fn from_config(config: &TrellaraConfig, config_path: &Path) -> Result<Self> {
        let lake_ddl = LakeDdlSummary::from_config(config);
        let lake_epoch = LakeEpochSummary::from_config(config, &pilot_lake_epoch_args(config_path));
        let lake_verify = LakeFaninVerifySummary::from_epochs(
            config,
            config_path,
            Path::new("lake-epoch.json"),
            Path::new("lake-epoch.json"),
            lake_epoch.clone(),
            lake_epoch.clone(),
            true,
        );
        let spark_template_args = pilot_spark_template_args(config_path);
        let spark_current_state = LakeSparkTemplateSummary::from_args(
            config,
            &spark_template_args,
            LakeSparkTemplateKind::CurrentState,
        )?;
        let spark_scd2 = LakeSparkTemplateSummary::from_args(
            config,
            &spark_template_args,
            LakeSparkTemplateKind::Scd2,
        )?;
        let spark_maintenance = LakeSparkTemplateSummary::from_args(
            config,
            &spark_template_args,
            LakeSparkTemplateKind::Maintenance,
        )?;
        let spark_completeness_dashboard = LakeSparkTemplateSummary::from_args(
            config,
            &spark_template_args,
            LakeSparkTemplateKind::Dashboard,
        )?;
        let spark_golden_fixture = SparkGoldenFixture::from_config(config);
        let lake_writer_plan = pilot_package_lake_writer_plan(config)?;
        let lake_fanin_run =
            LakeFaninRunSummary::from_writer_plan(config, lake_writer_plan.clone());

        Ok(Self {
            lake_ddl,
            lake_epoch,
            lake_verify,
            spark_current_state,
            spark_scd2,
            spark_maintenance,
            spark_completeness_dashboard,
            spark_golden_fixture,
            lake_writer_plan,
            lake_fanin_run,
        })
    }
}

fn pilot_lake_epoch_args(config_path: &Path) -> LakeEpochArgs {
    LakeEpochArgs {
        config: config_path.to_path_buf(),
        scenario: LakeEpochScenario::OfflineStoresPublishWithGaps,
        required_source_count: 12,
        offline_source_count: 3,
        duplicate_replay_count: 2,
        format: QuickstartOutputFormat::Json,
    }
}

fn pilot_spark_template_args(config_path: &Path) -> LakeSparkTemplateArgs {
    LakeSparkTemplateArgs {
        config: config_path.to_path_buf(),
        catalog: "spark_catalog".to_string(),
        namespace: None,
        table: None,
        epoch_id: "epoch-2026-08-16T00".to_string(),
        target_table: None,
        primary_key_column: None,
        accept_complete_with_gaps: true,
        unsafe_allow_non_consumable_epoch: false,
        unsafe_override_reason: None,
        format: QuickstartOutputFormat::Text,
    }
}

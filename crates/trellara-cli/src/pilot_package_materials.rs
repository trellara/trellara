use std::path::Path;

use crate::lake_fanin_run_types::LakeFaninRunSummary;
use crate::pilot_package_materials_lake::PilotPackageLakeMaterials;
use crate::{
    pilot_package_ddl_barrier_status, pilot_package_ddl_release_proof,
    pilot_package_diagnostics_status, pilot_package_partition_rebalance_plan,
    pilot_package_schema_ddl_envelope_plan, quickstart_readiness, CliError,
    ConsistencyContractSummary, ConsumerSemanticsSummary, DdlApplyPlanSummary,
    DdlEnvelopePlanSummary, DdlPlanApplyMode, DdlPlanArgs, DdlPlanSummary, DdlReleaseProof,
    EnterpriseEvaluationSummary, FleetControlPlaneSummary, FleetEvidencePlanSummary,
    FleetReportSummary, FleetScorecardSummary, FlowStatusSummary, IdentityAuditSummary,
    LakeDdlSummary, LakeEpochSummary, LakeFaninVerifySummary, LakeSparkTemplateSummary,
    PerformanceEnvelopeSummary, PilotExecutiveEvidenceSummary, PilotGuideSummary,
    PilotScorecardSummary, QuickstartArgs, QuickstartOutputFormat, QuickstartSummary, Result,
    SparkGoldenFixture, TrellaraConfig,
};

pub(crate) struct PilotPackageMaterials {
    pub(crate) quickstart_plan: QuickstartSummary,
    pub(crate) quickstart_readiness: crate::QuickstartReadinessSummary,
    pub(crate) pilot_guide: PilotGuideSummary,
    pub(crate) pilot_scorecard: PilotScorecardSummary,
    pub(crate) executive_evidence: PilotExecutiveEvidenceSummary,
    pub(crate) enterprise_evaluation: EnterpriseEvaluationSummary,
    pub(crate) schema_ddl_plan: DdlPlanSummary,
    pub(crate) schema_ddl_apply_plan: DdlApplyPlanSummary,
    pub(crate) schema_ddl_envelope_plan: DdlEnvelopePlanSummary,
    pub(crate) ddl_barrier_status: trellara_checkpoint::DdlBarrierSummary,
    pub(crate) ddl_release_proof: DdlReleaseProof,
    pub(crate) fleet_report: FleetReportSummary,
    pub(crate) fleet_scorecard: FleetScorecardSummary,
    pub(crate) fleet_evidence_plan: FleetEvidencePlanSummary,
    pub(crate) consistency_contract: ConsistencyContractSummary,
    pub(crate) performance_envelope: PerformanceEnvelopeSummary,
    pub(crate) identity_audit: IdentityAuditSummary,
    pub(crate) consumer_semantics: ConsumerSemanticsSummary,
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
    pub(crate) fleet_control_plane: FleetControlPlaneSummary,
    pub(crate) diagnostics_status: FlowStatusSummary,
    pub(crate) partition_rebalance_plan: Option<trellara_protocol::PartitionRebalancePlan>,
}

impl PilotPackageMaterials {
    pub(crate) fn from_config(config: &TrellaraConfig, config_path: &Path) -> Result<Self> {
        let quickstart_args = QuickstartArgs {
            config: config_path.to_path_buf(),
            check: false,
            format: QuickstartOutputFormat::Text,
        };
        let quickstart_plan = QuickstartSummary::from_args(&quickstart_args);
        let quickstart_readiness = quickstart_readiness(&QuickstartArgs {
            config: config_path.to_path_buf(),
            check: true,
            format: QuickstartOutputFormat::Text,
        })?;
        let pilot_guide = PilotGuideSummary::from_config(config, config_path);
        let pilot_scorecard = PilotScorecardSummary::from_config(config, config_path);
        let executive_evidence = PilotExecutiveEvidenceSummary::from_config(config, config_path);
        let enterprise_evaluation = EnterpriseEvaluationSummary::from_config(config, config_path);
        let schema_ddl_plan = DdlPlanSummary::from_args(
            config,
            &DdlPlanArgs {
                config: config_path.to_path_buf(),
                changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Json,
            },
        )?;
        let schema_ddl_apply_plan = DdlApplyPlanSummary::from_args(
            config,
            &DdlPlanArgs {
                config: config_path.to_path_buf(),
                changes: vec!["add_nullable_column:public.sales.discount_code:text".to_string()],
                apply_mode: DdlPlanApplyMode::AutoSafe,
                format: QuickstartOutputFormat::Json,
            },
        )?;
        let schema_ddl_envelope_plan = pilot_package_schema_ddl_envelope_plan(config)?;
        let ddl_barrier_status = pilot_package_ddl_barrier_status(config, &schema_ddl_plan)?;
        let ddl_release_proof =
            pilot_package_ddl_release_proof(config, &schema_ddl_plan, &schema_ddl_apply_plan)?;
        let fleet_report = FleetReportSummary::from_configs(&[(config, config_path)])?;
        let fleet_scorecard =
            FleetScorecardSummary::from_configs(&[(config.clone(), config_path.to_path_buf())])?;
        let fleet_evidence_plan =
            FleetEvidencePlanSummary::from_configs(&[(config.clone(), config_path.to_path_buf())]);
        let consistency_contract = ConsistencyContractSummary::from_config(config, config_path)?;
        let performance_envelope = PerformanceEnvelopeSummary::from_config(config, config_path);
        let identity_audit = IdentityAuditSummary::from_config(config, config_path);
        let consumer_semantics = ConsumerSemanticsSummary::from_config(config, config_path)?;
        let lake_materials = PilotPackageLakeMaterials::from_config(config, config_path)?;
        let fleet_control_plane =
            FleetControlPlaneSummary::from_report_and_scorecard(&fleet_report, &fleet_scorecard);
        let diagnostics_status = pilot_package_diagnostics_status(config);
        let partition_rebalance_plan = pilot_package_partition_rebalance_plan(config)?;

        if config.dataset.tables.is_empty() {
            return Err(CliError::InvalidConfig(
                "dataset.tables must include at least one table".to_string(),
            ));
        }

        Ok(Self {
            quickstart_plan,
            quickstart_readiness,
            pilot_guide,
            pilot_scorecard,
            executive_evidence,
            enterprise_evaluation,
            schema_ddl_plan,
            schema_ddl_apply_plan,
            schema_ddl_envelope_plan,
            ddl_barrier_status,
            ddl_release_proof,
            fleet_report,
            fleet_scorecard,
            fleet_evidence_plan,
            consistency_contract,
            performance_envelope,
            identity_audit,
            consumer_semantics,
            lake_ddl: lake_materials.lake_ddl,
            lake_epoch: lake_materials.lake_epoch,
            lake_verify: lake_materials.lake_verify,
            spark_current_state: lake_materials.spark_current_state,
            spark_scd2: lake_materials.spark_scd2,
            spark_maintenance: lake_materials.spark_maintenance,
            spark_completeness_dashboard: lake_materials.spark_completeness_dashboard,
            spark_golden_fixture: lake_materials.spark_golden_fixture,
            lake_writer_plan: lake_materials.lake_writer_plan,
            lake_fanin_run: lake_materials.lake_fanin_run,
            fleet_control_plane,
            diagnostics_status,
            partition_rebalance_plan,
        })
    }
}

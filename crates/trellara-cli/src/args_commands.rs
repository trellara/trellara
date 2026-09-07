use clap::Subcommand;

use crate::args_recovery::{QuarantineClearArgs, QuarantineListArgs, QuarantineReplayReadyArgs};
use crate::{
    ChaosReportArgs, ConfigArgs, DdlBarrierAckArgs, DdlBarrierRecordArgs, DdlBarrierStatusArgs,
    DdlEnvelopePlanArgs, DdlPlanArgs, DevDownArgs, DevLogsArgs, DevStackArgs,
    FleetControlPlaneArgs, FleetEvidencePlanArgs, FleetIdentityAuditArgs, FleetInitArgs,
    FleetReportArgs, FleetScorecardArgs, LakeEpochArgs, LakeFaninCompletenessArgs,
    LakeFaninRunArgs, LakeFaninVerifyArgs, LakeInspectArgs, LakeSparkTemplateArgs,
    LakeWriterPlanArgs, LocalStreamLocateArgs, LocalStreamReconstructArgs, LocalStreamSeekArgs,
    NativeWorkerReportArgs, PilotEvidenceArgs, PilotEvidenceCheckArgs, PilotEvidenceTemplateArgs,
    PilotGuideArgs, PilotPackageArgs, PilotScorecardArgs,
};

#[derive(Clone, Debug, Subcommand)]
pub enum DevCommand {
    #[command(about = "Start the local source and target Postgres stack")]
    Up(DevStackArgs),
    #[command(about = "Stop and remove the local development stack")]
    Down(DevDownArgs),
    #[command(about = "Show local development stack logs")]
    Logs(DevLogsArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum ConfigurationCommand {
    #[command(about = "Migrate a flow configuration to the current schema version")]
    Migrate(ConfigArgs),
    #[command(about = "Render a structurally redacted current-version configuration")]
    Redact(ConfigArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum FlowCommand {
    Validate(ConfigArgs),
    Create(ConfigArgs),
    Status(ConfigArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum FleetCommand {
    Init(FleetInitArgs),
    Report(FleetReportArgs),
    Scorecard(FleetScorecardArgs),
    ControlPlane(FleetControlPlaneArgs),
    IdentityAudit(FleetIdentityAuditArgs),
    EvidencePlan(FleetEvidencePlanArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum ContractCommand {
    Test(ConfigArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum SchemaCommand {
    Discover(ConfigArgs),
    DdlPlan(DdlPlanArgs),
    DdlApplyPlan(DdlPlanArgs),
    DdlEnvelopePlan(DdlEnvelopePlanArgs),
    DdlBarrier {
        #[command(subcommand)]
        command: Box<DdlBarrierCommand>,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum DdlBarrierCommand {
    Record(DdlBarrierRecordArgs),
    Ack(Box<DdlBarrierAckArgs>),
    Status(DdlBarrierStatusArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum LakeCommand {
    Plan(ConfigArgs),
    Ddl(ConfigArgs),
    WriterPlan(LakeWriterPlanArgs),
    SparkTemplate {
        #[command(subcommand)]
        command: LakeSparkTemplateCommand,
    },
    Inspect(LakeInspectArgs),
    Epoch(LakeEpochArgs),
    Fanin {
        #[command(subcommand)]
        command: LakeFaninCommand,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum LakeFaninCommand {
    Plan(ConfigArgs),
    Ddl(ConfigArgs),
    EpochSpec(LakeEpochArgs),
    Run(LakeFaninRunArgs),
    Verify(LakeFaninVerifyArgs),
    #[command(about = "Build an epoch completeness proof instead of a generic sink report")]
    Completeness(LakeFaninCompletenessArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum LakeSparkTemplateCommand {
    CurrentState(LakeSparkTemplateArgs),
    Scd2(LakeSparkTemplateArgs),
    Maintenance(LakeSparkTemplateArgs),
    Dashboard(LakeSparkTemplateArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum StreamCommand {
    InspectLocal(ConfigArgs),
    LocateLocal(LocalStreamLocateArgs),
    ReconstructLocal(LocalStreamReconstructArgs),
    SeekLocal(LocalStreamSeekArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum NativeCommand {
    WorkerReport(NativeWorkerReportArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum ChaosCommand {
    Run,
    Report(ChaosReportArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum QuarantineCommand {
    List(QuarantineListArgs),
    Clear(QuarantineClearArgs),
    ReplayReady(QuarantineReplayReadyArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum RepairCommand {
    Plan(ConfigArgs),
}

#[derive(Clone, Debug, Subcommand)]
pub enum PilotCommand {
    Guide(PilotGuideArgs),
    Scorecard(PilotScorecardArgs),
    Evidence(PilotEvidenceArgs),
    EvidenceCheck(PilotEvidenceCheckArgs),
    EvidenceTemplate(PilotEvidenceTemplateArgs),
    Package(PilotPackageArgs),
}

use clap::{Parser, Subcommand};

use crate::args::*;
use crate::args_commands::*;
use crate::args_pilot::*;
use crate::args_runtime::*;
use crate::args_source::*;

#[derive(Debug, Parser)]
#[command(name = "trellara")]
#[command(version)]
#[command(about = "Source safety and verified replication for PostgreSQL fleets")]
#[command(
    long_about = "Trellara verifies whether PostgreSQL sources are safe for CDC, moves committed transactions through a durable boundary, applies them downstream, and proves convergence.\n\nPublic loop: trellara init -> trellara check -> trellara preflight -> trellara run -> trellara verify -> trellara status"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(hide = true)]
    #[command(about = "Print the Trellara CLI version")]
    Version,
    #[command(about = "Inspect CDC readiness, slot posture, WAL risk, and failover-slot safety")]
    Check(SourceSafetyArgs),
    #[command(about = "Create a brokerless local Trellara flow config")]
    Init(InitArgs),
    #[command(hide = true)]
    Quickstart(QuickstartArgs),
    #[command(hide = true)]
    MvpCheck(MvpCheckArgs),
    #[command(hide = true)]
    Evaluate(EvaluateArgs),
    #[command(hide = true)]
    Consistency(ConsistencyContractArgs),
    #[command(hide = true)]
    Performance(PerformanceEnvelopeArgs),
    #[command(hide = true)]
    IdentityAudit(IdentityAuditArgs),
    #[command(hide = true)]
    EvidenceRegistry(EvidenceRegistryArgs),
    #[command(hide = true)]
    Semantics(ConfigArgs),
    #[command(hide = true)]
    Demo(RunArgs),
    #[command(about = "Run bootstrap, relay, apply, and optional verify for a configured flow")]
    Run(RunArgs),
    #[command(about = "Summarize verified replication topology across multiple flow configs")]
    Fleet {
        #[command(subcommand)]
        command: FleetCommand,
    },
    #[command(hide = true)]
    Dev {
        #[command(subcommand)]
        command: DevCommand,
    },
    #[command(about = "Migrate and safely inspect versioned configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigurationCommand,
    },
    #[command(hide = true)]
    Flow {
        #[command(subcommand)]
        command: FlowCommand,
    },
    #[command(hide = true)]
    ContractTest(ConfigArgs),
    #[command(hide = true)]
    Contract {
        #[command(subcommand)]
        command: ContractCommand,
    },
    #[command(hide = true)]
    SchemaDiscover(ConfigArgs),
    #[command(hide = true)]
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    #[command(about = "Plan and run lakehouse fan-in from verified transactions")]
    Lake {
        #[command(subcommand)]
        command: LakeCommand,
    },
    #[command(hide = true)]
    Stream {
        #[command(subcommand)]
        command: StreamCommand,
    },
    #[command(hide = true)]
    Native {
        #[command(subcommand)]
        command: NativeCommand,
    },
    #[command(hide = true)]
    Chaos {
        #[command(subcommand)]
        command: ChaosCommand,
    },
    #[command(hide = true)]
    Validate(ConfigArgs),
    #[command(hide = true)]
    Explain(ConfigArgs),
    #[command(about = "Check source and target contracts before CDC starts")]
    Preflight(ConfigArgs),
    #[command(hide = true)]
    Bootstrap(BootstrapArgs),
    #[command(hide = true)]
    Relay(RelayArgs),
    #[command(hide = true)]
    ApplySchema(ConfigArgs),
    #[command(hide = true)]
    Apply(ApplyArgs),
    #[command(hide = true)]
    Snapshot(SnapshotArgs),
    #[command(about = "Show flow health, recovery actions, and proof evidence")]
    Status(StatusArgs),
    #[command(hide = true)]
    Reseed(ReseedArgs),
    #[command(about = "Compare source and target tables and record checksum evidence")]
    Verify(VerifyArgs),
    #[command(hide = true)]
    Report(ConfigArgs),
    #[command(hide = true)]
    Alerts(ConfigArgs),
    #[command(hide = true)]
    Dashboard(ConfigArgs),
    #[command(hide = true)]
    Metrics(ConfigArgs),
    #[command(hide = true)]
    Diagnostics(ConfigArgs),
    #[command(hide = true)]
    SourceSafety(SourceSafetyArgs),
    #[command(hide = true)]
    InspectTransaction(InspectTransactionArgs),
    #[command(hide = true)]
    PartitionLocal(ConfigArgs),
    #[command(hide = true)]
    PartitionWatermarks(PartitionWatermarksArgs),
    #[command(hide = true)]
    PartitionRebalancePlan(PartitionRebalancePlanArgs),
    #[command(hide = true)]
    Quarantine {
        #[command(subcommand)]
        command: QuarantineCommand,
    },
    #[command(hide = true)]
    Repair {
        #[command(subcommand)]
        command: RepairCommand,
    },
    #[command(hide = true)]
    RepairPlan(ConfigArgs),
    #[command(hide = true)]
    PilotGuide(PilotGuideArgs),
    #[command(hide = true)]
    PilotScorecard(PilotScorecardArgs),
    #[command(hide = true)]
    PilotEvidence(PilotEvidenceArgs),
    #[command(hide = true)]
    PilotEvidenceCheck(PilotEvidenceCheckArgs),
    #[command(hide = true)]
    PilotEvidenceTemplate(PilotEvidenceTemplateArgs),
    #[command(hide = true)]
    PilotPackage(PilotPackageArgs),
    #[command(hide = true)]
    Pilot {
        #[command(subcommand)]
        command: PilotCommand,
    },
}

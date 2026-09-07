# Trellara Roadmap

Status: strategic roadmap and domain analysis

Last reviewed: 2026-09-01

Planning horizon: design-partner qualification through an evidence-gated 1.0

## Executive conclusion

Trellara should not compete as another generic CDC connector. Capture and destination-specific sync are crowded, increasingly bundled into clouds and analytical databases, and often purchased for convenience. The defensible problem is narrower and harder:

> Make PostgreSQL fleet replication safe to start, safe to operate, and possible to prove after failure—without forcing the customer into one destination or hosted control plane.

The product wedge is the read-only source-safety diagnostic plus a brokerless verified replication proof. The durable expansion is fleet-wide source risk, transaction-boundary evidence, recovery automation, and completeness-gated analytical fan-in.

The next milestone is not more feature breadth. It is live qualification and design-partner evidence that the implemented contracts solve an expensive operational problem.

## Current baseline

The repository already contains a broader implementation than a typical prototype:

- a no-config read-only PostgreSQL source-safety binary;
- external `pgoutput` capture with protocol-v2 streaming and bounded spill;
- a versioned transaction envelope with strict, strict-chunked, and partitioned barriers;
- a durable brokerless log and a Kafka/Redpanda adapter;
- idempotent PostgreSQL apply with atomic checkpoints and quarantine;
- snapshot/handoff, verification, reseed, DDL barriers, partition watermarks, and recovery plans;
- deterministic failure simulation and reviewable evidence packages;
- an optional native PostgreSQL extension data plane; and
- raw CDC lake planning, Parquet encoding, and a feature-gated Iceberg REST/S3 writer.

What is missing is equally important:

- one automated live qualification matrix across services and supported versions;
- evidence from customer-shaped workloads and failure drills;
- an internally consistent support/release matrix;
- arm64 and non-tarball artifacts matching the runtime vocabulary;
- a supportable upgrade and compatibility policy;
- enough design-partner demand to justify a hosted control plane; and
- proof that fleet fan-in completeness is a buying priority rather than an elegant technical extension.

## Technical domain: what actually makes this hard

Market analysis follows below. This section is the engineering half of the domain: the constraints
PostgreSQL and the lakehouse impose on *any* product in this space, what Trellara does about each,
and what is still unproven. Nothing here is a Trellara opinion — it is the shape of the problem.

### 1. The source is the thing you can break

A logical replication slot is a promise the database makes to a consumer: WAL needed by that slot
will not be removed. That promise is what makes CDC correct, and it is also the single largest
operational hazard the technology carries. A consumer that stalls, crashes, or falls behind converts
correctness into unbounded disk growth on a production primary. PostgreSQL added
`max_slot_wal_keep_size` to bound it, at the cost of *invalidating* the slot — which trades an
outage for a full reseed. PostgreSQL 18 adds `idle_replication_slot_timeout` for the same reason
from a different angle.

Slots also hold back two horizons. `xmin` pins row versions from vacuum; `catalog_xmin` pins
catalog tuples. A long-lived logical slot on a busy database is therefore also a bloat source and,
in the worst case, a transaction-ID wraparound contributor. The failure is slow, then sudden.

*What Trellara does:* the read-only diagnostic is aimed squarely at this. It reports `wal_status`,
`safe_wal_size`, retained WAL bytes, invalidation reason, inactivity, projected headroom from an
observed generation rate, wraparound budget, and which backends or slots pin the xmin horizon —
before creating anything. *What is unproven:* the projection model against real workloads, and
whether the thresholds produce useful alerts rather than noise on a fleet.

### 2. Decoding is a plugin contract, and the plugin choice is a lock-in decision

Logical decoding emits changes through an output plugin. `pgoutput` is the only one guaranteed
present on managed PostgreSQL, which effectively removes the choice for anyone targeting RDS,
Aurora, Cloud SQL, Azure, or Neon. `pgoutput` protocol version 2 adds in-progress transaction
streaming — the difference between buffering a 10 GB transaction in the publisher and receiving it
incrementally — but streaming shifts the reassembly burden onto the consumer, which must now spill,
track subtransactions, and handle `STREAM ABORT`.

*What Trellara does:* `pgoutput` v2 with streaming on by default, bounded spill (1024 changes,
capped at 1M), spill artifacts scoped to transaction identity and validated on reload, and relation
fingerprint checks that reject a streamed transaction whose schema interpretation changed mid-flight.
`test_decoding` is retained for compatibility spikes only. *What is unproven:* spill behaviour under
workload-shaped large transactions with wide TOAST values, and the memory ceiling under concurrent
streamed transactions.

### 3. Replica identity and unchanged TOAST are the correctness cliff

`REPLICA IDENTITY DEFAULT` gives an UPDATE or DELETE only the primary key in the old tuple. A table
with no primary key and no replica identity gives nothing, and PostgreSQL will happily let you
publish it — the failure surfaces downstream as an unapplicable change, not as a source error.
Separately, an UPDATE that does not modify a TOASTed column sends a placeholder rather than the
value. A consumer that treats that placeholder as NULL silently corrupts the target. This is the
most common way CDC pipelines lose data quietly.

*What Trellara does:* replica identity and primary key are preflight checks, not runtime surprises;
unchanged-TOAST markers are preserved through the protocol and merged against existing target
values at apply; and a key column arriving as unchanged TOAST is quarantined rather than guessed.
*What is unproven:* the value-canonicalization corpus against the long tail of PostgreSQL types
(ranges, domains, arrays of composites, custom types, `numeric` edge cases) that verification must
hash identically on both sides.

### 4. The snapshot-to-CDC handoff is where most implementations have a gap

An initial copy must be consistent with the exact LSN where streaming begins, or the pipeline has
either a hole or a duplicate window. The mechanism exists — create the slot, take the exported
snapshot, copy inside a repeatable-read transaction bound to it, start streaming from the slot's
consistent point — but the failure modes are all crash-shaped: crash mid-copy, crash after copy
before handoff, crash after handoff before the relay starts. Orchestrating that in process memory is
where correctness quietly disappears.

*What Trellara does:* nine durable states in `trellara.snapshot_runs`, per-relation progress in
`trellara.snapshot_table_progress`, and the consistent LSN recorded in
`trellara.snapshot_handoff_events` before streaming. A restart after handoff resumes CDC from the
recorded boundary rather than opening a new snapshot. `verified` is a separate terminal state from
`streaming`, because finishing the copy is not proof of convergence. *What is unproven:* interruption
at every transition against real providers and real table sizes.

### 5. DDL is not in the change stream, and no honest product can pretend otherwise

Logical replication carries row changes. It does not carry `ALTER TABLE`. Every CDC product in this
space either stops at "your schemas must match", replays DDL text heuristically (which breaks on
function bodies, permissions, partition administration, and provider-specific syntax), or requires
an out-of-band change process. There is no fourth option that is both general and safe.

The hard part is not detecting the change — relation metadata fingerprints do that. It is
*coordinating release across sinks that evolve at different speeds*: a target PostgreSQL applies in
milliseconds, an Iceberg table needs a schema evolution commit, and a derived Spark view may need a
job redeploy. Releasing post-DDL DML before the slowest sink accepts the change is a correctness
bug that looks like a scheduling bug.

*What Trellara does:* a policy-gated barrier with a digest, per-sink acknowledgements, and a
`dml_replay_after_ddl_barrier` projection so DML releases only once required sinks accept the exact
digest. Three orthogonal axes describe a change — compatibility, decision, and operator apply mode —
rather than one fused category. *What is unproven:* the supported-operation matrix. "Additive column
works" is not a schema story; the matrix of what is refused needs to be published and drilled.

### 6. Failover breaks slots, and providers differ about it

Physical failover historically destroyed logical slots outright, forcing a reseed on every
promotion. PostgreSQL 17 added slot synchronization to standbys, gated on `sync_replication_slots`,
`synchronized_standby_slots`, and slots created with `failover = true`. That is real progress and it
is also exactly the kind of capability where "supports PostgreSQL 17" is too coarse a claim: a
managed provider may expose it, gate it, or implement promotion differently. An operator needs to
know whether the required slots are *actually synchronized* before someone presses promote.

*What Trellara does:* reports failover/sync posture per slot where the server exposes it, and treats
WAL loss or slot invalidation as a fresh snapshot path rather than pretending replay is still
possible. *What is unproven:* per-provider behaviour. This is a fixture problem, and building
RDS/Aurora/Cloud SQL/Azure/Neon fixtures without assuming they are the same system is on the
research backlog for a reason.

### 7. Ordering and atomicity fight throughput, and the trade has to be explicit

A single ordered stream preserves transaction semantics and caps throughput at one consumer.
Partitioning by key scales, and immediately raises three questions most products answer implicitly:
what happens to a row whose partition key is NULL; what happens when an UPDATE moves a row between
partitions; and what a "watermark" means when partitions advance at different rates. The honest
answer to the third is that a global watermark is the *low* watermark across the complete configured
partition set, not the best observed partition — and any dashboard that shows the latter is lying
about freshness.

*What Trellara does:* null-key and key-change policies default to quarantine rather than a silent
choice; the global watermark is the low watermark across the complete set; a manifest names every
required partition and a commit marker binds to the manifest checksum, so global visibility cannot
release on a subset. Partition-local consumers are permitted, but are not allowed to describe their
view as a complete multi-partition transaction. Rebalance plans are evidence-only; nothing moves
live ownership automatically. *What is unproven:* skew behaviour and rebalance ergonomics at real
fleet cardinality.

### 8. "Exactly once" is a property of a boundary, not of a system

End to end, a CDC pipeline is at-least-once with an idempotent consumer. Every claim stronger than
that is either scoped to a named boundary or false. The interesting engineering is in the two
ordering rules that make replay safe:

- **Source side:** durably publish, *then* durably checkpoint, *then* acknowledge the source LSN.
  Acknowledging first means a crash loses committed data with no way to detect it.
- **Target side:** row mutations, the deduplication record, and the target checkpoint commit in one
  database transaction; the stream cursor advances only after that commit returns.

Between them sits the genuinely unavoidable case: an operation that may have succeeded, where the
caller never learned the answer. A broker publish that times out. An object store write with no
response. The only correct handling is reconciliation against durable evidence by stable identity —
never converting an ambiguous result into an assumed success.

*What Trellara does:* both orderings are the crate boundaries themselves, `KafkaPublishIdentity`
exists specifically for the ambiguous-publish case, and Iceberg intents/receipts/object digests
distinguish replay from conflict. *What is unproven:* the failure matrix under real brokers, real
object stores, and real process kills — which is the entire point of the live qualification harness.

### 9. Append-only is the right lakehouse default, and it has a bill

Iceberg commits one table's metadata atomically. There is no standard cross-table atomic commit, so
"the whole fleet landed" is not something the format can express. An append-only raw changelog is
the correct source-of-truth layer — it is replay-safe, it preserves source transaction identity, and
it does not require merge-on-read machinery to be correct. The bill is small files, snapshot growth,
catalog contention, and the fact that current-state and history are now derived jobs someone has to
operate.

*What Trellara does:* a fixed-schema raw changelog with lineage columns, a completeness ledger
committed last as the consumption gate, and Spark-rendered current-state/SCD2 templates instead of a
native equality-delete writer. *What is unproven:* the maintenance operating envelope — compaction,
snapshot expiry, orphan cleanup — and epoch close latency and cost at real source/table/file counts.

### 10. Verification is mostly canonicalization, and canonicalization is mostly types

"Do source and target agree?" sounds like a checksum problem. It is a type problem. Equivalent
PostgreSQL values must hash identically across process, platform, row order, and JSON key order,
while genuinely different identities must not collapse. `numeric` scale, timestamp precision and
timezone handling, `bytea` encoding, array and composite nesting, NULL versus JSON null versus an
absent field — each is a place where a naive implementation reports drift that is not there, or
misses drift that is. And a checksum match proves nothing without first proving the target applied
through the intended source boundary.

*What Trellara does:* canonical JSON with deterministic primary-key ordering, watermark comparison
before checksum comparison, and reports that distinguish missing, extra, mismatched, and
*unknown/incomplete evidence* rather than collapsing the last into "match". *What is unproven:* the
type corpus, and what sampling guidance is defensible when a full compare is too expensive.

### 11. "Supports PostgreSQL" is four or five different products

RDS, Aurora, Cloud SQL, Azure Database for PostgreSQL, and Neon differ in available extensions,
superuser semantics, replication role grants, slot behaviour on failover, parameter groups,
connection handling, and what they let you observe. A pipeline that works on self-managed PostgreSQL
17 tells you almost nothing about Aurora. The support matrix is a product decision with an ongoing
cost, not a compatibility footnote.

*What Trellara does:* the diagnostic uses ordinary TLS connection handling so managed providers take
the same path as self-managed, and the release contract is deliberately narrower than the CI matrix.
*What is unproven:* everything else here. There is currently no provider lane in CI at all — CI runs
no PostgreSQL service — which makes this the single largest gap between what the code does and what
the project can claim.

### Where the technical domain points the roadmap

Reading the eleven items above, the ordering is not a matter of taste. Items 1, 3, 4, and 8 are
where a customer loses data or an availability incident starts, and they are all
implemented-but-unqualified. Items 5, 7, and 9 are where claims outrun evidence. Item 11 is the
multiplier on all of them. That is why Phase 2 is a live qualification harness rather than more
features, and why Phase 5 and Phase 6 are gated behind it.

## Market domain analysis

### 1. PostgreSQL provides primitives, not a fleet operating model

PostgreSQL logical replication provides snapshot-plus-change replication and preserves transactional consistency within a subscription. That validates the core primitive, but production operators still own slot health, WAL retention, failover readiness, schema compatibility, initial-copy handoff, monitoring, and repair. PostgreSQL's model is intentionally database-native rather than a cross-destination fleet evidence system. See the [PostgreSQL logical replication documentation](https://www.postgresql.org/docs/current/logical-replication.html).

Logical slots create a real source-side risk boundary: retained WAL is protected for a consumer, but lagging slots can grow disk usage. AWS documents both the usefulness of native CDC start points and the need for storage alarms because retained changes can increase source disk consumption. This makes source safety and headroom a first-class product problem, not a cosmetic dashboard metric. See [AWS DMS PostgreSQL source guidance](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.PostgreSQL.html).

PostgreSQL failover has improved, including logical-slot synchronization in recent majors, but operators must still know whether the required slots are actually synchronized before promotion. Provider/version differences make “supports PostgreSQL” too coarse a claim.

### 2. Transaction metadata exists elsewhere; mandatory transaction visibility is still a positioning choice

Debezium can emit BEGIN/END transaction metadata and enrich row events with total and per-collection ordering. That means Trellara cannot claim transaction identity itself as unique. The differentiation is that Trellara makes the complete transaction boundary mandatory for its verified consumers, binds large/partitioned barriers with manifests and checksums, couples progress to durable evidence, and provides exact recovery semantics. See [Debezium's PostgreSQL transaction metadata](https://debezium.io/documentation/reference/stable/connectors/postgresql.html#postgresql-transaction-metadata).

The competitive message should therefore be “provable transaction release and recovery,” not “we preserve transactions and others do not.”

### 3. Destination vendors are absorbing PostgreSQL CDC

Destination-specific CDC is becoming a feature of the destination:

- ClickHouse acquired PeerDB and integrated it into ClickPipes; its own retrospective says the connector reached general availability and grew substantially after integration. See [ClickHouse's Postgres CDC retrospective](https://clickhouse.com/blog/postgres-cdc-year-in-review-2025).
- Snowflake's Openflow PostgreSQL connector is generally available and provides both current-state tables and a change log. See [Snowflake Openflow for PostgreSQL](https://docs.snowflake.com/en/user-guide/data-integration/openflow/connectors/postgres/about).
- AWS DMS combines full load and CDC for migration and ongoing replication, with a broad provider surface and a long list of source-specific prerequisites and limitations. See [AWS DMS PostgreSQL source guidance](https://docs.aws.amazon.com/dms/latest/userguide/CHAP_Source.PostgreSQL.html).

This trend compresses the standalone value of “move rows from PostgreSQL to destination X.” It strengthens Trellara's neutral verification position only if the product can demonstrate value before and across destination choice.

### 4. Managed convenience is the default buyer expectation

Buyers compare against a managed connector, not against writing a decoder from scratch. A credible product must minimize first-run infrastructure, explain provider prerequisites, make failure action-oriented, and offer a support path that does not require reading protocol internals.

The brokerless local path is strategically important because forcing Kafka into the first evaluation would reproduce the operational burden many teams are trying to avoid. Kafka remains valuable for scale-out and existing platform teams, but it should be a deployment choice rather than an onboarding tax.

### 5. Database-per-tenant makes fleet correctness more valuable

Database-per-tenant and database-per-user architectures trade stronger isolation for a larger operational fleet. Neon explicitly describes the need for a control plane as database counts grow. This is strong directional evidence for fleet identity, source-safety aggregation, schema-version visibility, and fan-in completeness, although it is not yet proof that those teams will buy Trellara. See [Neon's database-per-user architecture analysis](https://neon.com/blog/multi-tenancy-and-database-per-user-design-in-postgres).

This is also the most coherent AI-platform adjacency: agent/RAG/SaaS products increasingly use isolated Postgres projects or databases per tenant. Trellara should treat AI as a workload overlay on fleet correctness, not as a separate “AI data platform” product category.

### 6. Iceberg is a useful neutral analytical boundary, with explicit semantic limits

Iceberg snapshots atomically replace one table's metadata and track data files through manifests. The specification does not provide a standard cross-table atomic commit. Trellara's decision to publish a completeness ledger last is therefore the correct abstraction: it creates a consumption gate without claiming an Iceberg guarantee that does not exist. See the [Apache Iceberg table specification](https://iceberg.apache.org/spec/).

The current Iceberg Rust ecosystem supports REST catalog operations and append writing. Its capability matrix also evolves quickly; current upstream status shows operations that were unavailable in older versions, including equality-delete writing, while several rewrite/maintenance operations remain incomplete. Trellara should pin behavior to its tested dependency version and use upstream status as an input, not as evidence that a Trellara capability is qualified. See [Apache Iceberg implementation status](https://iceberg.apache.org/status/) and [Iceberg Rust APIs](https://rust.iceberg.apache.org/api.html).

### 7. Correctness is valuable only when it shortens an incident or approval cycle

Protocol rigor by itself is not a market. A buyer values one or more concrete outcomes:

- avoid a source outage caused by unsafe slot/WAL posture;
- approve a replication launch with auditable evidence;
- recover an exact transaction without guessing offsets;
- prove a target is caught up after a migration or incident;
- know which tenant/source is missing from an analytical epoch; or
- give a security/platform review a bounded artifact instead of a collection of screenshots.

Roadmap priority must follow measured time, risk, and approval-cycle reduction—not the elegance of another internal contract.

## Competitive landscape

| Category | Typical strength | Structural limitation Trellara can exploit | Trellara response |
| --- | --- | --- | --- |
| Native PostgreSQL logical replication | Few moving parts, database-native semantics | PostgreSQL-to-PostgreSQL scope; fleet evidence and heterogeneous targets remain operator work | Interoperate with the primitives; add preflight, durable evidence, repair, and neutral consumers. |
| Debezium/Kafka Connect | Mature connector ecosystem and broad adoption | Operational footprint; transaction metadata is optional and downstream semantics vary | Keep Kafka compatibility, but make complete boundaries and verification mandatory for built-in consumers. |
| Cloud migration services | Managed full load + CDC and provider integration | Provider/destination coupling, provider-specific limitations, opaque recovery boundaries | Lead with neutral assessment and proof; do not compete on connector count. |
| Destination-native CDC | Excellent time-to-value into one warehouse/database | Destination lock-in and limited cross-target evidence | Be the safety/evidence layer before and alongside destination choice. |
| ELT connector platforms | Broad catalog and managed operations | Breadth favors row movement and transformations over source-specific correctness | Remain PostgreSQL-deep; integrate/export evidence rather than building a connector marketplace. |
| Distributed/Postgres-derived databases | Integrated replication inside one database platform | Solves a different topology and usually requires database adoption | Stay compatible with ordinary/managed PostgreSQL and external targets. |
| Lakehouse ingestion tools | High-throughput append and table management | Source transaction/fleet completeness may be flattened into file or table progress | Preserve source transaction identity and publish an explicit fleet epoch ledger. |

## Positioning

### Category

**PostgreSQL replication assurance** is a better category than generic CDC or ETL.

### One-sentence promise

Trellara tells a platform team whether PostgreSQL replication is safe, carries only provable committed boundaries, and gives them an exact recovery or convergence proof when something fails.

### Defensible combination

No individual element is unique. The defensible system is the combination of:

- read-only source-risk assessment;
- destination-neutral transaction envelopes;
- durability-before-ack sequencing;
- exact replay/quarantine/reseed semantics;
- source/target convergence proofs;
- fleet identity and completeness evidence; and
- a deployment path that starts without Kafka or a hosted control plane.

### What not to claim

- “Exactly once” without naming the boundary and reconciliation rules.
- “Zero data loss” outside tested failure assumptions.
- “Supports PostgreSQL 15-18” when the advertised release contract is narrower.
- “Production ready” before the live qualification matrix and design-partner gates pass.
- “Atomic Iceberg fan-in” across tables.
- “Automatic DDL” without naming the policy, sinks, digest, and acknowledgement state.

## Target segments

### Priority 1: B2B/vertical SaaS with database-per-customer fleets

Why it fits:

- many structurally similar PostgreSQL sources;
- strong isolation or regional requirements;
- schema/version drift across a fleet;
- need for consolidated analytics or controlled migrations;
- platform teams that feel the operational cost directly.

Entry problem: read-only fleet assessment and one verified tenant flow.

Expansion: fleet risk, proof registry, recovery drills, analytical completeness.

### Priority 2: retail, logistics, and edge fleets

Why it fits:

- intermittent links and independently operating locations;
- explicit “which source is missing?” analytical questions;
- local durable buffering is more useful than a broker dependency at every site;
- recovery and late-arrival evidence matter.

Risk: heterogeneous networking and support burden can overwhelm an early team. Start with centrally reachable PostgreSQL fleets or a small number of representative sites.

### Priority 3: platform teams running online migrations or regional copies

Why it fits:

- source safety, snapshot handoff, target convergence, and cutover evidence are immediately valuable;
- a bounded project can produce a clear before/after outcome.

Risk: migration may be episodic rather than recurring. The recurring expansion must be continuous source-safety/fleet assurance, not perpetual migration tooling.

### Priority 4: AI/agent platforms with isolated PostgreSQL tenants

Why it fits:

- rapidly growing database counts;
- need for cross-tenant operational or analytical views;
- high sensitivity to tenant identity and provenance.

Risk: “AI” can pull the roadmap toward vector pipelines, model telemetry, and application features. Only pursue capabilities that reuse the same source identity, transaction, verification, and completeness contracts.

### Deprioritized segments

- a single small PostgreSQL database with a destination-native connector that already meets its needs;
- buyers primarily seeking hundreds of SaaS/API sources;
- teams requiring active-active writes or automatic conflict resolution;
- workloads whose main requirement is arbitrary transformation rather than replication correctness.

## Buyers, users, and proof moments

| Persona | Primary concern | Product proof |
| --- | --- | --- |
| Database/platform engineer | Will the slot, WAL, schema, and handoff hurt production? | Read-only source-safety report and preflight. |
| Data/platform engineer | Can downstream consumers trust complete transactions and epochs? | Boundary inspection, watermarks, checksums, and epoch ledger. |
| SRE/operator | What failed and what exact action is safe? | Latest failure, ordered recovery action, boundary locator, quarantine/reseed evidence. |
| Security/reliability reviewer | Can the team demonstrate controls without exposing data/secrets? | Redacted configuration, digested evidence package, explicit compatibility/support matrix. |
| Engineering leader | Does this reduce risk and operating time enough to adopt? | Time-to-proof, incident drill results, support burden, and go/no-go scorecard. |

## Strategic risks

### Demand risk

The repository demonstrates technical depth, not product demand. The most dangerous outcome is building a complete hosted fleet platform before learning whether teams will pay for source safety, verified recovery, or lake completeness.

Mitigation: require design-partner evidence gates before each control-plane investment.

### Surface-area risk

PostgreSQL replication, Kafka, local storage, pgrx, Iceberg, Spark templates, fleet analysis, and a CLI create a very large qualification surface.

Mitigation: define one golden external-relay/PostgreSQL-target path, then add qualification lanes in a deliberate order. Treat native and Iceberg as optional tracks until the core lane is supportable.

### Trust-claim risk

The product's value is correctness, so inconsistent support claims are unusually damaging. The current native matrix mismatch and artifact/runtime mismatch must be resolved before broad promotion.

Mitigation: generate a support manifest from tested release inputs and fail CI when docs, runtime constants, workflows, and package metadata diverge.

### Source-impact risk

Logical slots retain WAL; long/large transactions, stalled consumers, failover, and provider settings can threaten source availability.

Mitigation: make headroom, horizon pinners, slot invalidation, failover readiness, and bounded capture part of acceptance—not optional dashboards.

### Recovery-complexity risk

Replay, snapshot, DDL, partition barriers, and multi-sink release can produce recovery states operators cannot reason about.

Mitigation: every blocked state needs one named boundary, one evidence bundle, one safe next command, and a tested drill.

### Lake semantics risk

Per-table Iceberg atomicity can be mistaken for cross-table fleet atomicity. Small-file pressure, catalog contention, schema evolution, and ambiguous object writes compound the issue.

Mitigation: keep the append-only raw layer, publish the epoch ledger last, require consumers to gate on it, and qualify maintenance separately from ingestion.

### Support economics risk

Provider/version/network combinations can create a support matrix larger than the team can sustain.

Mitigation: sell a narrow qualified matrix, report unknown combinations honestly, and expand only from repeated partner demand plus automated coverage.

## Roadmap principles

1. Qualification precedes breadth.
2. Every phase ends in externally reviewable evidence.
3. Support claims are generated from what is tested and shipped.
4. A hosted feature requires repeated user pull, not architectural possibility.
5. The external relay/PostgreSQL target is the golden lane.
6. Native and Iceberg tracks must not destabilize the golden lane.
7. Performance goals come from customer workloads; synthetic numbers are baselines, not market proof.
8. Recovery time and source safety matter more than peak benchmark throughput at this stage.

## Phased execution plan

### Phase 0 — One source of truth

Status: complete. Nine `docs/` memos were folded into one design document, the crate READMEs and
`skills.md` files were reconciled against the source, and the drift found in that pass was fixed:
the native queue-capacity bound, two undocumented GUCs, the raw-CDC schema (which is a fixed 29-column
changelog, not source business columns plus `_trellara_*` fields), and the boundary-mode vocabulary
(`dataset.mode` has two values; there is no `strict_chunked` mode).

Deliverables:

- one root onboarding README;
- one authoritative design document;
- one roadmap and domain analysis;
- crate-by-crate responsibility and contributor guidance;
- executable tests redirected from superseded filenames to the consolidated design;
- explicit current-vs-planned status and support mismatches.

Exit gate:

- all internal Markdown links resolve;
- docs tests and workspace unit tests pass;
- no deleted historical document remains a build/runtime dependency.

Carried forward: the doc-freshness gates in `quickstart_artifacts.rs`,
`quickstart_schema_artifacts/`, and `quickstart_source_safety_artifacts.rs`, plus the
`include_str!` assertions in `trellara-protocol`, `trellara-stream-local`, and
`trellara-checkpoint`, are the mechanism that keeps this phase from decaying. They assert exact
phrases in `docs/DESIGN.md`. That makes prose a build dependency, which is the point, but it also
means a heading rename is a code change — run `make quickstart-proof-check` after editing the design
document, not only `cargo test`.

### Phase 1 — Golden-path release integrity

Objective: make the external relay from PostgreSQL to PostgreSQL installable, reproducible, and supportable.

Deliverables:

- generate a machine-readable support manifest from runtime constants and release jobs;
- align external PostgreSQL majors, OS/architecture, package claims, README, and artifacts;
- publish signed checksums, SBOM/provenance, and reproducible build metadata;
- define protocol/config/checkpoint upgrade and rollback policy;
- test installation and upgrade from a clean host, not only `cargo run`;
- make the ten-minute quickstart time and failure output measurable in CI;
- reduce hidden-command dependencies in customer-facing evidence packages.

Exit gate:

- a clean Linux x86_64 host can install, run the golden flow, restart every service, verify convergence, and uninstall using published artifacts;
- every public compatibility claim has a matching automated lane;
- support-manifest drift fails CI.

### Phase 2 — Live qualification harness

Objective: convert deterministic contracts into cross-service failure evidence.

Harness dimensions:

- supported PostgreSQL majors and at least one managed provider per declared class;
- PK updates, unchanged TOAST, truncate, logical messages, schema drift, and long/large transactions;
- snapshot interruption at every durable state transition;
- relay crash before publish, after publish, after checkpoint, and before source feedback;
- local torn tail/index loss/cursor crash;
- Kafka leader loss, ISR shrink, ambiguous delivery, rebalance, and consumer crash;
- target deadlock, schema mismatch, quarantine, replay, and reseed;
- slot invalidation, WAL loss, promotion/failover posture, and network partitions;
- rolling binary/config/checkpoint upgrade; and
- secret redaction and evidence integrity.

Outputs:

- one immutable run manifest;
- environment/provider/version inventory;
- timings and resource ceilings;
- expected vs observed recovery state;
- evidence-package digests; and
- a qualification verdict that cannot pass with missing mandatory lanes.

Exit gate:

- all golden-lane failures recover without silent loss, partial visibility, or unbounded source risk;
- recovery point and recovery time are measured for every scenario;
- failures produce one actionable next step.

### Phase 3 — Design-partner validation

Objective: determine which correctness capability has pull and who buys it.

Minimum discovery program:

- 15 structured interviews across at least three target segments;
- 5 read-only source-safety assessments against real environments;
- 3 bounded replication pilots;
- at least 2 rehearsed failure/recovery drills;
- at least 1 fleet or analytical fan-in evaluation before expanding that track.

Questions to answer:

- What incident, migration, audit, or fleet problem triggered the evaluation?
- What is the cost of the current failure mode?
- Which evidence is required for approval?
- Does “no Kafka for evaluation” materially shorten time-to-proof?
- Is the buyer willing to operate the data plane, or is managed operation mandatory?
- Which provider/version combinations are non-negotiable?
- Is fleet completeness valuable enough to pay for independently of row movement?

Exit gate:

- three partners complete the golden proof;
- two independently describe the same recurring problem in their own language;
- at least one commits budget or a concrete procurement path;
- roadmap language is rewritten from observed objections and outcomes.

### Phase 4 — Production hardening and supportability

Objective: support a small number of production design partners without heroics.

Deliverables:

- tested upgrades and rollback across protocol/config/checkpoint versions;
- capacity model for WAL headroom, spill, local segments, Kafka inflight, target lag, and evidence retention;
- SLOs for freshness, recovery, verification, and source-risk alerting;
- backup/restore and disaster-recovery runbooks for Trellara state;
- alert routing and incident artifact collection;
- least-privilege PostgreSQL role templates and provider-specific setup guides;
- retention and deletion policy for row-bearing artifacts;
- compatibility deprecation windows;
- optional arm64 artifacts only after a complete qualification lane exists.

Exit gate:

- production pilots sustain their workload window and pass a scheduled recovery drill;
- on-call can diagnose and execute the safe path using generated evidence without source-code access;
- no support claim depends on an untested manual branch.

### Phase 5 — Fleet fan-in and Iceberg qualification

Objective: prove that the second pillar solves a distinct, repeated customer problem.

Deliverables:

- live REST catalog/S3-compatible object-store matrix;
- exact object/catalog ambiguity and replay tests;
- schema evolution and DDL acknowledgement qualification;
- small-file/compaction/expiration/orphan-cleanup operating envelope;
- Spark current-state/SCD2 golden datasets and late-recovery tests;
- completeness dashboard tied to expected source inventory;
- consumer conformance tests proving `_trellara_epochs` gating;
- measured epoch close latency and cost by source/table/file count.

Exit gate:

- a real fleet publishes raw CDC and an epoch ledger, survives writer/catalog failure, reconciles replay, and produces a correct derived view;
- consumers cannot accidentally read an unreleased epoch through the supported path;
- at least two partners ask for the same fan-in/completeness workflow.

### Phase 6 — Optional native-extension qualification

Objective: turn the implemented native path into a supportable product only where it creates measurable value.

Deliverables:

- decide whether the supported native matrix is 17-18 or 15-18;
- align runtime constants, CI, packages, docs, and live server tests;
- upgrade/uninstall/restart/promotion procedures;
- ABI/pgrx/PostgreSQL patch-release policy;
- queue sizing and backpressure benchmarks;
- proof that the native path materially improves cost, latency, or deployability over the external relay.

Exit gate:

- every advertised major passes install, preload, capture, crash, feedback, upgrade, and uninstall qualification;
- at least one design partner chooses native for a measured reason rather than novelty.

### Phase 7 — Pull-driven hosted control plane

Objective: centralize recurring fleet work without moving correctness out of the data plane.

Build only after the same requests recur across partners. Candidate first capabilities:

- read-only fleet inventory and source-risk aggregation;
- evidence package registry and sharing workflow;
- compatibility/upgrade posture;
- alert routing and drill scheduling;
- identity collision prevention; and
- deployment orchestration only after read-only surfaces prove insufficient.

Before implementation, design:

- tenant authorization and row-level access boundaries;
- regional/data-residency policy;
- secret ownership and rotation;
- audit log and evidence retention;
- control-plane outage behavior;
- billing/usage boundaries; and
- guarantees that data-plane replication continues safely when control-plane services are unavailable.

Exit gate:

- three partners independently request the same hosted workflow;
- two agree to use or pay for it;
- the capability can fail without violating data-plane acknowledgement or apply invariants.

## Cross-cutting workstreams

| Workstream | Near-term outcome | 1.0-quality outcome |
| --- | --- | --- |
| Source safety | Provider-aware actionable assessment | Continuous fleet risk with tested alert thresholds and failover posture. |
| Protocol | Version-1 compatibility corpus | Documented evolution/deprecation policy and independent consumer conformance kit. |
| Local transport | Crash/recovery qualification | Capacity/retention tooling and operational SLOs for supported single-node use. |
| Kafka | Failure/rebalance qualification | Broker-version compatibility, upgrade runbooks, and sustained-load envelope. |
| PostgreSQL apply | Golden failure matrix | Provider/version matrix, upgrade policy, and bounded recovery SLO. |
| Verification | Customer-shaped canonicalization corpus | Scheduled validation policy, sampling/full-compare guidance, and accepted evidence thresholds. |
| Schema/DDL | Additive-change qualification | Explicit supported-operation matrix and multi-sink release drills. |
| Native extension | Decide supported majors | Fully aligned qualified matrix or explicit experimental status. |
| Iceberg | Live append/reconcile matrix | Completeness-gated fan-in with maintenance and derived-view qualification. |
| Security | Secret redaction and least privilege | Threat model, supply-chain controls, audit/retention, and incident playbooks. |
| DX | Reliable ten-minute proof | Install-to-first-proof SLO, generated reference, diagnostics, and upgrade UX. |

## Metrics and decision gates

### Activation

- time from download to first source-safety report;
- percentage of assessments that produce an actionable finding;
- time from install to first verified transaction;
- percentage of evaluators completing without Kafka;
- setup failures by provider and prerequisite.

### Correctness and operations

- source WAL headroom at detection and at recovery;
- maximum unacknowledged/replay window;
- recovery point actually observed in each failure drill;
- median and p95 time to identify the blocking boundary;
- median and p95 time to recover/reseed;
- quarantined transactions by root cause;
- verification mismatch and unknown-evidence rates;
- false-positive/false-negative source-safety findings.

### Product evidence

- design-partner assessments, pilots, and completed drills;
- recurring problems stated independently by multiple partners;
- evidence artifacts used in an approval or incident review;
- conversion from free assessment to continuous pilot;
- willingness to pay and procurement status;
- support hours per active flow/source.

### Fleet/lake

- expected vs observed sources per epoch;
- epoch close latency;
- complete, complete-with-gaps, late-recovery, and blocked rates;
- duplicate/conflicting replay counts;
- object/catalog reconciliation time;
- file counts/size distribution and maintenance cost;
- time until derived current-state/SCD2 views become releasable.

### Stop/go rules

- Do not build hosted orchestration without three repeated requests and two committed users.
- Do not expand PostgreSQL support without automated install/failure/upgrade qualification.
- Do not add a new destination unless it validates the neutral evidence layer or is paid design-partner work.
- Do not replace Spark-derived current state until append-only fan-in has customer pull and a native mutation writer has a demonstrably better operating model.
- Do not publish performance claims that cannot be reproduced from a retained run manifest.

## Immediate 30/60/90-day sequence

The dates are sequencing guidance, not externally committed release dates.

### First 30 days

- finish docs/link/test consolidation;
- generate and enforce the support manifest;
- create the live harness skeleton and golden PostgreSQL-to-PostgreSQL lane;
- automate crash points around relay publication/checkpoint/ack and target apply/cursor commit;
- validate install-from-artifact on clean Linux x86_64;
- recruit the first five assessment candidates;
- define interview and pilot evidence templates.

### Days 31-60

- add the declared external PostgreSQL-major matrix and one managed provider lane;
- qualify snapshot interruption, large transactions, TOAST, schema drift, slot risk, quarantine, and reseed;
- run at least three real source-safety assessments and one bounded pilot;
- publish reproducible release/SBOM/provenance metadata;
- decide whether native qualification is active or explicitly experimental;
- measure actual time-to-proof and operator recovery time.

### Days 61-90

- complete three bounded pilots and two failure drills if partner access permits;
- convert findings into provider-specific setup and recovery guidance;
- establish initial SLOs from observed workloads;
- run one Iceberg fan-in qualification only if a partner has the need;
- make a go/no-go decision on continuous fleet monitoring and evidence registry;
- cut a qualification milestone only if every published claim maps to retained evidence.

## Explicitly deferred

Unless a paid partner changes the evidence, defer:

- non-PostgreSQL sources;
- a generic transform/stream-processing language;
- NATS/Pulsar/additional brokers;
- multi-primary conflict resolution;
- a broad catalog of destination connectors;
- native Iceberg current-state/equality-delete mutation;
- a general schema-migration engine;
- embedded dashboards that duplicate existing observability stacks;
- automatic live partition movement;
- a hosted control plane with write access to customer infrastructure;
- AI-specific vector, prompt, or model-observability features.

## Research backlog

### Customer research

- Interview database/platform owners after an actual CDC, migration, WAL, or convergence incident.
- Collect redacted examples of approval artifacts and failed runbooks.
- Quantify current tools, operator hours, incident cost, and acceptable recovery time.
- Test whether “source-safety score” or “go/no-go replication proof” is the stronger initial framing.
- Identify who owns budget: database platform, data platform, infrastructure, or application engineering.

### Technical research

- Build provider fixtures for RDS/Aurora, Cloud SQL, Azure Database for PostgreSQL, and Neon without treating them as identical.
- Track PostgreSQL logical failover-slot behavior by major/provider.
- Maintain a pgoutput/protobuf compatibility corpus across releases.
- Benchmark spill and replay using workload-shaped large transactions and TOAST values.
- Compare Iceberg REST catalogs and S3-compatible stores for idempotency, versioning, and ambiguous commits.
- Evaluate upstream Iceberg Rust changes against pinned `0.10.1` before adopting them.
- Model source/fleet cardinality against checkpoint/evidence retention and control-plane cost.

### Security and compliance research

- Threat-model source credentials, local segments, spill files, evidence packages, relay sockets, catalog credentials, and hosted metadata.
- Decide whether payload encryption at rest is required in addition to filesystem/object-store controls.
- Define evidence retention/deletion and customer-controlled keys.
- Add SBOM, dependency provenance, release signing, and secret-scanning gates.
- Document which artifacts may contain row values and how redaction/sampling changes the proof.

## 1.0 definition

Trellara should call itself 1.0 only when:

1. the external PostgreSQL-to-PostgreSQL golden lane has a declared and automated compatibility matrix;
2. installation, upgrade, rollback, restart, failover posture, replay, quarantine, reseed, and verification are qualified from published artifacts;
3. support claims are generated from tests and release metadata;
4. at least three external design partners have completed pilots and two have completed failure drills;
5. operational SLOs and capacity guidance come from retained workloads;
6. protocol/config/checkpoint compatibility and deprecation policies are published;
7. security, supply-chain, secret, evidence-retention, and incident practices are reviewed;
8. the CLI's public surfaces and machine schemas have compatibility coverage; and
9. optional native/Iceberg capabilities are either separately qualified or labeled experimental without ambiguity.

That threshold deliberately values a narrow trusted product over a broad unqualified platform.

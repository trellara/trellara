create schema if not exists trellara;

create table if not exists trellara.flow_checkpoints (
    source_id text not null,
    dataset_id text not null,
    last_seen_lsn text not null default '',
    last_durable_lsn text not null default '',
    last_applied_lsn text not null default '',
    updated_at timestamptz not null default now(),
    primary key (source_id, dataset_id),
    constraint flow_checkpoints_durable_not_ahead_of_seen_check check (
        last_seen_lsn = ''
        or last_durable_lsn = ''
        or last_durable_lsn::pg_lsn <= last_seen_lsn::pg_lsn
    ),
    constraint flow_checkpoints_applied_not_ahead_of_durable_check check (
        last_durable_lsn = ''
        or last_applied_lsn = ''
        or last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn
    )
);

do $$
begin
    if not exists (
        select 1
         from pg_constraint
         where conname = 'flow_checkpoints_durable_not_ahead_of_seen_check'
           and conrelid = 'trellara.flow_checkpoints'::regclass
    ) then
        alter table trellara.flow_checkpoints
            add constraint flow_checkpoints_durable_not_ahead_of_seen_check
            check (
                last_seen_lsn = ''
                or last_durable_lsn = ''
                or last_durable_lsn::pg_lsn <= last_seen_lsn::pg_lsn
            );
    end if;

    if not exists (
        select 1
         from pg_constraint
         where conname = 'flow_checkpoints_applied_not_ahead_of_durable_check'
           and conrelid = 'trellara.flow_checkpoints'::regclass
    ) then
        alter table trellara.flow_checkpoints
            add constraint flow_checkpoints_applied_not_ahead_of_durable_check
            check (
                last_durable_lsn = ''
                or last_applied_lsn = ''
                or last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn
            );
    end if;
end $$;

create table if not exists trellara.applied_transactions (
    source_id text not null,
    database_id text not null,
    dataset_id text not null,
    transaction_id text not null,
    commit_lsn text not null,
    applied_at timestamptz not null default now(),
    primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn)
);

create table if not exists trellara.apply_quarantine (
    source_id text not null,
    database_id text not null,
    dataset_id text not null,
    transaction_id text not null,
    commit_lsn text not null,
    reason text not null,
    detail text not null,
    attempt_count bigint not null default 1,
    first_seen_at timestamptz not null default now(),
    last_seen_at timestamptz not null default now(),
    primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn)
);

alter table trellara.applied_transactions
    add column if not exists database_id text not null default '';

alter table trellara.apply_quarantine
    add column if not exists database_id text not null default '';

do $$
begin
    if not exists (
        select 1
          from pg_constraint
         where conname = 'applied_transactions_pkey'
           and conrelid = 'trellara.applied_transactions'::regclass
           and pg_get_constraintdef(oid) like '%database_id%'
    ) then
        alter table trellara.applied_transactions
            drop constraint if exists applied_transactions_pkey;
        alter table trellara.applied_transactions
            add constraint applied_transactions_pkey
            primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn);
    end if;

    if not exists (
        select 1
          from pg_constraint
         where conname = 'apply_quarantine_pkey'
           and conrelid = 'trellara.apply_quarantine'::regclass
           and pg_get_constraintdef(oid) like '%database_id%'
    ) then
        alter table trellara.apply_quarantine
            drop constraint if exists apply_quarantine_pkey;
        alter table trellara.apply_quarantine
            add constraint apply_quarantine_pkey
            primary key (source_id, database_id, dataset_id, transaction_id, commit_lsn);
    end if;
end $$;

create table if not exists trellara.reseed_events (
    id bigserial primary key,
    source_id text not null,
    dataset_id text not null,
    watermark_lsn text not null,
    table_count bigint not null,
    copied_rows bigint not null,
    completed_at timestamptz not null default now()
);

create index if not exists reseed_events_flow_completed_at_idx
    on trellara.reseed_events (source_id, dataset_id, completed_at desc);

create table if not exists trellara.snapshot_handoff_events (
    id bigserial primary key,
    source_id text not null,
    dataset_id text not null,
    relation text not null,
    watermark_lsn text not null,
    copied_rows bigint not null,
    completed_at timestamptz not null default now()
);

create index if not exists snapshot_handoff_events_flow_completed_at_idx
    on trellara.snapshot_handoff_events (source_id, dataset_id, completed_at desc);

create table if not exists trellara.snapshot_runs (
    source_id text not null,
    dataset_id text not null,
    run_id text not null,
    state text not null,
    slot_name text not null,
    consistent_lsn text,
    current_relation text,
    copied_rows bigint not null default 0,
    failure_reason text,
    started_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    primary key (source_id, dataset_id, run_id),
    check (state in (
        'planned',
        'slot_created',
        'snapshot_exported',
        'copying_table',
        'copy_complete',
        'stream_handoff_ready',
        'streaming',
        'verified',
        'failed_recoverable'
    )),
    check (copied_rows >= 0)
);

create index if not exists snapshot_runs_flow_updated_at_idx
    on trellara.snapshot_runs (source_id, dataset_id, updated_at desc);

create table if not exists trellara.snapshot_table_progress (
    source_id text not null,
    dataset_id text not null,
    run_id text not null,
    relation text not null,
    state text not null,
    copied_rows bigint not null default 0,
    watermark_lsn text,
    updated_at timestamptz not null default now(),
    primary key (source_id, dataset_id, run_id, relation),
    check (state in (
        'planned',
        'slot_created',
        'snapshot_exported',
        'copying_table',
        'copy_complete',
        'stream_handoff_ready',
        'streaming',
        'verified',
        'failed_recoverable'
    )),
    check (copied_rows >= 0)
);

create index if not exists snapshot_table_progress_run_idx
    on trellara.snapshot_table_progress (source_id, dataset_id, run_id, relation);

create table if not exists trellara.validation_events (
    id bigserial primary key,
    source_id text not null,
    dataset_id text not null,
    source_watermark_lsn text not null,
    target_watermark_lsn text not null,
    converged boolean not null,
    table_count bigint not null,
    drift_count bigint not null,
    drift_relations text[] not null default '{}',
    evidence_sha256 text,
    completed_at timestamptz not null default now()
);

alter table trellara.validation_events
    add column if not exists drift_relations text[] not null default '{}';

alter table trellara.validation_events
    add column if not exists evidence_sha256 text;

create index if not exists validation_events_flow_completed_at_idx
    on trellara.validation_events (source_id, dataset_id, completed_at desc);

create table if not exists trellara.partition_checkpoints (
    source_id text not null,
    dataset_id text not null,
    partition_id integer not null,
    last_durable_lsn text not null default '',
    last_applied_lsn text not null default '',
    updated_at timestamptz not null default now(),
    primary key (source_id, dataset_id, partition_id),
    check (partition_id >= 0),
    constraint partition_checkpoints_applied_not_ahead_of_durable_check check (
        last_durable_lsn = ''
        or last_applied_lsn = ''
        or last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn
    )
);

do $$
begin
    if not exists (
        select 1
         from pg_constraint
         where conname = 'partition_checkpoints_applied_not_ahead_of_durable_check'
           and conrelid = 'trellara.partition_checkpoints'::regclass
    ) then
        alter table trellara.partition_checkpoints
            add constraint partition_checkpoints_applied_not_ahead_of_durable_check
            check (
                last_durable_lsn = ''
                or last_applied_lsn = ''
                or last_applied_lsn::pg_lsn <= last_durable_lsn::pg_lsn
            );
    end if;
end $$;

create index if not exists partition_checkpoints_flow_partition_idx
    on trellara.partition_checkpoints (source_id, dataset_id, partition_id);

create table if not exists trellara.ddl_barriers (
    source_id text not null,
    database_id text not null,
    dataset_id text not null,
    barrier_id text not null,
    barrier_lsn text not null,
    schema_version text not null,
    cdc_transaction_boundary text not null default 'source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn',
    required_sinks text[] not null,
    requires_global_partition_pause boolean not null default false,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now(),
    primary key (source_id, database_id, dataset_id, barrier_id),
    check (array_length(required_sinks, 1) > 0),
    check (cdc_transaction_boundary = btrim(cdc_transaction_boundary) and cdc_transaction_boundary <> ''),
    check (
        not requires_global_partition_pause
        or 'partition_visibility' = any(required_sinks)
    )
);

alter table trellara.ddl_barriers
    add column if not exists database_id text not null default '';

alter table trellara.ddl_barriers
    add column if not exists cdc_transaction_boundary text not null
    default 'source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn';

do $$
begin
    if exists (
        select 1
          from pg_constraint
         where conrelid = 'trellara.ddl_barriers'::regclass
           and conname = 'ddl_barriers_pkey'
    ) then
        alter table trellara.ddl_barriers
            drop constraint if exists ddl_barriers_pkey;
        alter table trellara.ddl_barriers
            add constraint ddl_barriers_pkey
            primary key (source_id, database_id, dataset_id, barrier_id);
    end if;
end $$;

create index if not exists ddl_barriers_flow_updated_at_idx
    on trellara.ddl_barriers (source_id, database_id, dataset_id, updated_at desc);

create table if not exists trellara.ddl_barrier_acks (
    source_id text not null,
    database_id text not null,
    dataset_id text not null,
    barrier_id text not null,
    sink text not null,
    ack_lsn text not null,
    schema_version text not null,
    accepted boolean not null,
    detail text not null,
    acked_at timestamptz not null default now(),
    primary key (source_id, database_id, dataset_id, barrier_id, sink),
    check (sink = btrim(sink) and sink <> ''),
    check (schema_version = btrim(schema_version) and schema_version <> ''),
    check (btrim(detail) <> '')
);

alter table trellara.ddl_barrier_acks
    add column if not exists database_id text not null default '';

do $$
begin
    if exists (
        select 1
          from pg_constraint
         where conrelid = 'trellara.ddl_barrier_acks'::regclass
           and conname = 'ddl_barrier_acks_pkey'
    ) then
        alter table trellara.ddl_barrier_acks
            drop constraint if exists ddl_barrier_acks_pkey;
        alter table trellara.ddl_barrier_acks
            add constraint ddl_barrier_acks_pkey
            primary key (source_id, database_id, dataset_id, barrier_id, sink);
    end if;
end $$;

create index if not exists ddl_barrier_acks_flow_barrier_idx
    on trellara.ddl_barrier_acks (source_id, database_id, dataset_id, barrier_id);

create table if not exists trellara.iceberg_commit_intents (
    dataset_id text not null,
    epoch_id text not null,
    epoch_commit_id text not null,
    lake_table_name text not null,
    relation text not null,
    target text not null,
    table_commit_id text not null,
    manifest_digest text not null,
    file_count bigint not null,
    record_count bigint not null,
    planned_at text not null,
    recorded_at timestamptz not null default now(),
    primary key (dataset_id, epoch_id, target, table_commit_id),
    check (file_count > 0),
    check (record_count > 0)
);

create index if not exists iceberg_commit_intents_epoch_idx
    on trellara.iceberg_commit_intents (dataset_id, epoch_id, target);

create table if not exists trellara.iceberg_commit_receipts (
    dataset_id text not null,
    epoch_id text not null,
    epoch_commit_id text not null,
    target text not null,
    table_commit_id text not null,
    snapshot_id bigint not null,
    file_count bigint not null,
    record_count bigint not null,
    status text not null,
    committed_at text not null,
    recorded_at timestamptz not null default now(),
    primary key (dataset_id, epoch_id, target, table_commit_id),
    foreign key (dataset_id, epoch_id, target, table_commit_id)
        references trellara.iceberg_commit_intents
            (dataset_id, epoch_id, target, table_commit_id),
    check (snapshot_id > 0),
    check (file_count > 0),
    check (record_count > 0),
    check (status in ('committed', 'already_committed'))
);

create index if not exists iceberg_commit_receipts_epoch_idx
    on trellara.iceberg_commit_receipts (dataset_id, epoch_id, target);

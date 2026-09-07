use trellara_protocol::{DdlEvent, DdlOperation, TransactionBoundaryKind};

pub(crate) fn protocol_transaction_boundary_kind_label(
    kind: TransactionBoundaryKind,
) -> &'static str {
    match kind {
        TransactionBoundaryKind::Empty => "empty",
        TransactionBoundaryKind::DmlOnly => "dml_only",
        TransactionBoundaryKind::DdlOnly => "ddl_only",
        TransactionBoundaryKind::MixedDdlAndDml => "mixed_ddl_and_dml",
    }
}

pub(crate) fn protocol_ddl_operation_label(event: &DdlEvent) -> &'static str {
    match DdlOperation::try_from(event.operation).unwrap_or(DdlOperation::Unspecified) {
        DdlOperation::Unspecified => "unspecified",
        DdlOperation::AddColumn => "add_column",
        DdlOperation::AddTable => "add_table",
        DdlOperation::WidenColumnType => "widen_column_type",
        DdlOperation::RenameColumn => "rename_column",
        DdlOperation::RenameTable => "rename_table",
        DdlOperation::DropColumn => "drop_column",
        DdlOperation::AddNotNullColumn => "add_not_null_column",
        DdlOperation::ChangePrimaryKey => "change_primary_key",
        DdlOperation::ChangePartitionKey => "change_partition_key",
        DdlOperation::Other => "other",
    }
}

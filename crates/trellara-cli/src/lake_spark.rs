#[path = "lake_spark_command_plan.rs"]
mod lake_spark_command_plan;
#[path = "lake_spark_table.rs"]
mod lake_spark_table;

use crate::{
    lake_epoch_partitions_table_name, lake_epochs_table_name, lake_identifier_segment,
    lake_quarantine_table_name, lake_spark_idempotency_rule, lake_spark_template_kind_label,
    lake_spark_template_sha256, lake_spark_template_sql_file_name,
    lake_spark_unsafe_override_reason, lake_spark_visibility_rule, lake_table_prefix,
    lake_verification_table_name, render_lake_spark_pyspark_runner, render_lake_spark_template_sql,
    template_requires_primary_key, CliError, LakeSparkTemplateArgs, LakeSparkTemplateKind,
    LakeSparkTemplateRenderParams, LakeSparkTemplateSummary, Result, TableConfig, TrellaraConfig,
};

use lake_spark_command_plan::lake_spark_template_next_commands;
use lake_spark_table::{
    lake_spark_raw_cdc_table, lake_spark_target_table, selected_lake_spark_table,
};

impl LakeSparkTemplateSummary {
    pub(crate) fn from_args(
        config: &TrellaraConfig,
        args: &LakeSparkTemplateArgs,
        template: LakeSparkTemplateKind,
    ) -> Result<Self> {
        let table = selected_lake_spark_table(config, args.table.as_deref())?;
        Self::from_table(config, args, table, template)
    }

    fn from_table(
        config: &TrellaraConfig,
        args: &LakeSparkTemplateArgs,
        table: &TableConfig,
        template: LakeSparkTemplateKind,
    ) -> Result<Self> {
        let verify = table.verify.clone().unwrap_or_default();
        let primary_key_column = args
            .primary_key_column
            .clone()
            .unwrap_or(verify.primary_key)
            .trim()
            .to_string();
        if template_requires_primary_key(template) && primary_key_column.is_empty() {
            return Err(CliError::InvalidConfig(format!(
                "lake spark-template {} requires dataset.tables[].verify.primary_key or --primary-key-column for {}",
                lake_spark_template_kind_label(template),
                table.relation_id().display_name()
            )));
        }
        let unsafe_override_reason = lake_spark_unsafe_override_reason(args)?;

        let table_prefix = lake_table_prefix(&config.dataset.id, &table.schema, &table.name);
        let namespace = args
            .namespace
            .clone()
            .unwrap_or_else(|| lake_identifier_segment(&config.dataset.id));
        let target_table = args.target_table.clone().unwrap_or_else(|| {
            lake_spark_target_table(&config.dataset.id, &table_prefix, template)
        });
        let raw_cdc_table = lake_spark_raw_cdc_table(&table_prefix);
        let epochs_table = lake_epochs_table_name(&config.dataset.id);
        let epoch_partitions_table = lake_epoch_partitions_table_name(&config.dataset.id);
        let verification_table = lake_verification_table_name(&config.dataset.id);
        let quarantine_table = lake_quarantine_table_name(&config.dataset.id);
        let render_params = LakeSparkTemplateRenderParams {
            catalog: &args.catalog,
            namespace: &namespace,
            raw_cdc_table: &raw_cdc_table,
            epochs_table: &epochs_table,
            epoch_partitions_table: &epoch_partitions_table,
            verification_table: &verification_table,
            quarantine_table: &quarantine_table,
            target_table: &target_table,
            epoch_id: &args.epoch_id,
            primary_key_column: &primary_key_column,
            accept_complete_with_gaps: args.accept_complete_with_gaps,
            unsafe_allow_non_consumable_epoch: args.unsafe_allow_non_consumable_epoch,
            unsafe_override_reason: &unsafe_override_reason,
        };
        let sql = render_lake_spark_template_sql(template, render_params);
        let template_sha256 = lake_spark_template_sha256(&sql);
        let pyspark_runner = render_lake_spark_pyspark_runner(
            template,
            lake_spark_template_sql_file_name(template),
            LakeSparkTemplateRenderParams {
                catalog: &args.catalog,
                namespace: &namespace,
                raw_cdc_table: &raw_cdc_table,
                epochs_table: &epochs_table,
                epoch_partitions_table: &epoch_partitions_table,
                verification_table: &verification_table,
                quarantine_table: &quarantine_table,
                target_table: &target_table,
                epoch_id: &args.epoch_id,
                primary_key_column: &primary_key_column,
                accept_complete_with_gaps: args.accept_complete_with_gaps,
                unsafe_allow_non_consumable_epoch: args.unsafe_allow_non_consumable_epoch,
                unsafe_override_reason: &unsafe_override_reason,
            },
        );

        Ok(Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            template,
            relation: table.relation_id().display_name(),
            catalog: args.catalog.clone(),
            namespace,
            raw_cdc_table,
            epochs_table,
            epoch_partitions_table,
            verification_table,
            quarantine_table,
            target_table,
            epoch_id: args.epoch_id.clone(),
            primary_key_column,
            accept_complete_with_gaps: args.accept_complete_with_gaps,
            unsafe_allow_non_consumable_epoch: args.unsafe_allow_non_consumable_epoch,
            unsafe_override_reason: args
                .unsafe_allow_non_consumable_epoch
                .then_some(unsafe_override_reason.clone()),
            visibility_rule: lake_spark_visibility_rule(args, &unsafe_override_reason),
            idempotency_rule: lake_spark_idempotency_rule(template),
            template_sha256,
            sql,
            pyspark_runner,
            next_commands: lake_spark_template_next_commands(
                args,
                template,
                &unsafe_override_reason,
            ),
        })
    }
}

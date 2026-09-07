use tokio_postgres::NoTls;

use crate::{
    PostgresColumnInspection, PostgresRelationInspection, PostgresRelationInspectionConfig, Result,
};

pub async fn inspect_postgres_relation(
    config: PostgresRelationInspectionConfig,
) -> Result<PostgresRelationInspection> {
    let (client, connection) = tokio_postgres::connect(&config.database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("trellara inspect postgres connection failed: {error}");
        }
    });

    let columns = client
        .query(
            r#"
            select ordinal_position::int,
                   column_name,
                   udt_name,
                   is_nullable = 'YES'
              from information_schema.columns
             where table_schema = $1
               and table_name = $2
             order by ordinal_position
            "#,
            &[&config.relation.schema, &config.relation.table],
        )
        .await?
        .into_iter()
        .map(|row| PostgresColumnInspection {
            ordinal_position: row.get(0),
            name: row.get(1),
            type_name: row.get(2),
            nullable: row.get(3),
        })
        .collect::<Vec<_>>();

    Ok(PostgresRelationInspection {
        relation: config.relation,
        exists: !columns.is_empty(),
        columns,
    })
}

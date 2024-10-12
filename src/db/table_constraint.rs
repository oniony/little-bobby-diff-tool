use sqlx::{Error, FromRow, PgConnection};

const QUERY: &str = r#"
SELECT
    constraint_catalog,
    constraint_schema,
    constraint_name,
    constraint_type,
    table_catalog,
    table_schema,
    table_name,
    is_deferrable,
    initially_deferred,
    enforced,
    nulls_distinct
FROM
    information_schema.table_constraints
WHERE
    table_schema = ANY($1)
  AND
    constraint_type != 'CHECK'
ORDER BY
    table_catalog,
    table_schema,
    table_name,
    constraint_name,
    constraint_type;"#;

pub async fn query(connection: &mut PgConnection, schema_names: &[String]) -> Result<Vec<TableConstraint>, Error> {
    sqlx::query_as(QUERY)
        .bind(&schema_names[..])
        .fetch_all(connection).await
}

#[derive(Debug, Clone, PartialEq, FromRow)]
pub struct TableConstraint {
    pub constraint_catalog: String,
    pub constraint_schema: String,
    pub constraint_name: String,
    pub table_catalog: String,
    pub table_schema: String,
    pub table_name: String,
    pub constraint_type: String,
    pub is_deferrable: String,
    pub initially_deferred: String,
    pub enforced: String,
    pub nulls_distinct: Option<String>,
}
